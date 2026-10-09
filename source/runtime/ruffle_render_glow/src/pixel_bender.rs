//! Pixel Bender -> upstream Naga IR -> GLES 3.0. No shader/effect bypass.
use super::*;
use ruffle_render::pixel_bender::{PixelBenderShader, PixelBenderShaderHandle, PixelBenderShaderImpl, PixelBenderParam, PixelBenderType, OUT_COORD_NAME};
use ruffle_render::pixel_bender_support::{PixelBenderShaderArgument, ImageInputTexture};
use naga::back::glsl;

#[derive(Debug)]
pub struct Compiled {
    shader: PixelBenderShader,
    program: glow::Program,
    floats: usize,
    ints: usize,
    blocks: Vec<(u32, u32)>,
    textures: Vec<(String, u8, bool)>,
}
impl PixelBenderShaderImpl for Compiled { fn parsed_shader(&self) -> &PixelBenderShader { &self.shader } }
fn error(e: impl std::fmt::Display) -> BitmapError { BitmapError::Unimplemented(format!("PixelBender GLES: {e}").into()) }

pub fn translate(module: &naga::Module, stage: naga::ShaderStage, entry: &str) -> Result<(String, glsl::ReflectionInfo), BitmapError> {
    let info=naga::valid::Validator::new(naga::valid::ValidationFlags::all(),naga::valid::Capabilities::all()).validate(module).map_err(error)?;
    let options=glsl::Options { version: glsl::Version::new_gles(300), ..Default::default() };
    let pipeline=glsl::PipelineOptions { shader_stage:stage, entry_point:entry.into(), multiview:None };
    let mut text=String::new();
    let reflection=glsl::Writer::new(&mut text,module,&info,&options,&pipeline,naga::proc::BoundsCheckPolicies::default()).map_err(error)?.write().map_err(error)?;
    Ok((text,reflection))
}
impl GlowRenderBackend {
 pub fn compile_pb(&mut self, shader: PixelBenderShader) -> Result<PixelBenderShaderHandle,BitmapError> {
    let modules=naga_pixelbender::ShaderBuilder::build(&shader).map_err(error)?;
    let (vs,_)=translate(&modules.vertex,naga::ShaderStage::Vertex,naga_pixelbender::VERTEX_SHADER_ENTRYPOINT)?;
    let (fs,reflection)=translate(&modules.fragment,naga::ShaderStage::Fragment,naga_pixelbender::FRAGMENT_SHADER_ENTRYPOINT)?;
    unsafe {
     let p=self.gl.create_program().map_err(error)?;
     for (kind,text) in [(glow::VERTEX_SHADER,&vs),(glow::FRAGMENT_SHADER,&fs)] {
      let s=self.gl.create_shader(kind).map_err(error)?; self.gl.shader_source(s,text); self.gl.compile_shader(s);
      if !self.gl.get_shader_compile_status(s) { let msg=self.gl.get_shader_info_log(s); self.gl.delete_shader(s);self.gl.delete_program(p);return Err(error(msg)); }
      self.gl.attach_shader(p,s);self.gl.delete_shader(s);
     }
     self.gl.link_program(p);
     if !self.gl.get_program_link_status(p) {let msg=self.gl.get_program_info_log(p);self.gl.delete_program(p);return Err(error(msg));}
     let mut blocks=Vec::new();
     for (h,name) in &reflection.uniforms {
      let binding=modules.fragment.global_variables[*h].binding.as_ref().unwrap().binding;
      if let Some(index)=self.gl.get_uniform_block_index(p,name) {self.gl.uniform_block_binding(p,index,binding);blocks.push((binding,index));}
     }
     let textures=reflection.texture_mapping.iter().map(|(name,m)| {
      let index=(modules.fragment.global_variables[m.texture].binding.as_ref().unwrap().binding-naga_pixelbender::TEXTURE_START_BIND_INDEX) as u8;
      let linear=m.sampler.map(|s| modules.fragment.global_variables[s].binding.as_ref().unwrap().binding != naga_pixelbender::SAMPLER_CLAMP_NEAREST).unwrap_or(false);
      (name.clone(),index,linear)
     }).collect();
     log::info!("pixel_bender_compiled name={} uniforms={} textures={}",shader.name,blocks.len(),reflection.texture_mapping.len());
     Ok(PixelBenderShaderHandle(Arc::new(Compiled { shader, program:p, floats:modules.float_parameters_buffer_size as usize, ints:modules.int_parameters_buffer_size as usize, blocks, textures })))
    }
 }
 /// Always render to a scratch texture to avoid sampling the destination attachment.
 fn execute_pb(&mut self, handle:&PixelBenderShaderHandle,args:&[PixelBenderShaderArgument],width:u32,height:u32,filter:bool, uv:[f32;4]) -> Result<Vec<u8>,BitmapError> {
    let shader=<dyn Any>::downcast_ref::<Compiled>(&*handle.0).ok_or_else(||error("foreign shader handle"))?;
    let mut floats=Vec::<f32>::new(); let mut ints=Vec::<i32>::new();
    for arg in args {
     if let PixelBenderShaderArgument::ValueInput {index,value}=arg {
      if matches!(&shader.shader.params[*index as usize],PixelBenderParam::Normal {name,..} if name==OUT_COORD_NAME) {continue;}
      pack(value,&mut floats,&mut ints)?;
     }
    }
    floats.resize(shader.floats/4,0.0);ints.resize(shader.ints/4,0);
    unsafe {
     // All temporary GLES bindings are restored; the main renderer caches its program.
     let previous_program=self.gl.get_parameter_i32(glow::CURRENT_PROGRAM);
     let previous_fbo=self.gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING);
     let previous_vao=self.gl.get_parameter_i32(glow::VERTEX_ARRAY_BINDING);
     let previous_buffer=self.gl.get_parameter_i32(glow::ARRAY_BUFFER_BINDING);
     let active=self.gl.get_parameter_i32(glow::ACTIVE_TEXTURE);
     let mut viewport=[0;4];self.gl.get_parameter_i32_slice(glow::VIEWPORT,&mut viewport);
     let blend=self.gl.is_enabled(glow::BLEND);let stencil=self.gl.is_enabled(glow::STENCIL_TEST);let scissor=self.gl.is_enabled(glow::SCISSOR_TEST);
     let mut color_mask=[0;4];self.gl.get_parameter_i32_slice(glow::COLOR_WRITEMASK,&mut color_mask);
     let fbo=self.gl.create_framebuffer().map_err(error)?;let target=self.gl.create_texture().map_err(error)?;
     self.gl.bind_texture(glow::TEXTURE_2D,Some(target));
     self.gl.tex_image_2d(glow::TEXTURE_2D,0,glow::RGBA8 as i32,width as i32,height as i32,0,glow::RGBA,glow::UNSIGNED_BYTE,glow::PixelUnpackData::Slice(None));
     self.gl.bind_framebuffer(glow::FRAMEBUFFER,Some(fbo));self.gl.framebuffer_texture_2d(glow::FRAMEBUFFER,glow::COLOR_ATTACHMENT0,glow::TEXTURE_2D,Some(target),0);
     if self.gl.check_framebuffer_status(glow::FRAMEBUFFER)!=glow::FRAMEBUFFER_COMPLETE {return Err(error("incomplete filter framebuffer"));}
     self.gl.use_program(Some(shader.program));
     self.gl.disable(glow::BLEND);self.gl.disable(glow::STENCIL_TEST);self.gl.disable(glow::SCISSOR_TEST);self.gl.color_mask(true,true,true,true);
     self.gl.viewport(0,0,width as i32,height as i32);
     let zero=[if filter {1.0f32}else{0.0};4];
     let mut buffers=Vec::new();
     for (binding,_) in &shader.blocks {
      let data=match *binding {3=>bytemuck::cast_slice(floats.as_slice()),4=>bytemuck::cast_slice(ints.as_slice()),5=>bytemuck::cast_slice(&zero),_=>return Err(error("unknown PB binding"))};
      let b=self.gl.create_buffer().map_err(error)?; self.gl.bind_buffer(glow::UNIFORM_BUFFER,Some(b));self.gl.buffer_data_u8_slice(glow::UNIFORM_BUFFER,data,glow::STREAM_DRAW);self.gl.bind_buffer_base(glow::UNIFORM_BUFFER,*binding,Some(b));buffers.push(b);
     }
     let mut samplers=Vec::new(); let mut temps=Vec::new();
     for (unit,(name,index,linear)) in shader.textures.iter().enumerate() {
      let input=args.iter().find_map(|arg|if let PixelBenderShaderArgument::ImageInput {index:i,texture,..}=arg {if i==index {texture.as_ref()}else{None}}else{None});
      let texture=match input {
       Some(ImageInputTexture::Bitmap(b))=>as_registry_data(b).texture(),
       Some(ImageInputTexture::Bytes {width,height,channels,bytes})=>{
        let t=self.gl.create_texture().map_err(error)?;self.gl.bind_texture(glow::TEXTURE_2D,Some(t));
        let (fmt,internal)=match channels {1=>(glow::RED,glow::R32F),2=>(glow::RG,glow::RG32F),3=>(glow::RGB,glow::RGB32F),4=>(glow::RGBA,glow::RGBA32F),_=>return Err(error("invalid image channels"))};
        self.gl.tex_image_2d(glow::TEXTURE_2D,0,internal as i32,*width as i32,*height as i32,0,fmt,glow::FLOAT,glow::PixelUnpackData::Slice(Some(bytes)));temps.push(t);t
       }
       None=>{let t=self.gl.create_texture().map_err(error)?;self.gl.bind_texture(glow::TEXTURE_2D,Some(t));self.gl.tex_image_2d(glow::TEXTURE_2D,0,glow::RGBA8 as i32,1,1,0,glow::RGBA,glow::UNSIGNED_BYTE,glow::PixelUnpackData::Slice(Some(&[0,0,0,0])));temps.push(t);t}
       _=>return Err(error("unsupported RawTexture input")),
      };
      self.gl.active_texture(glow::TEXTURE0+unit as u32);self.gl.bind_texture(glow::TEXTURE_2D,Some(texture));
      let sampler=self.gl.create_sampler().map_err(error)?;let mode=if *linear {glow::LINEAR}else{glow::NEAREST} as i32;
      self.gl.sampler_parameter_i32(sampler,glow::TEXTURE_MIN_FILTER,mode);self.gl.sampler_parameter_i32(sampler,glow::TEXTURE_MAG_FILTER,mode);
      self.gl.sampler_parameter_i32(sampler,glow::TEXTURE_WRAP_S,glow::CLAMP_TO_EDGE as i32);self.gl.sampler_parameter_i32(sampler,glow::TEXTURE_WRAP_T,glow::CLAMP_TO_EDGE as i32);
      self.gl.bind_sampler(unit as u32,Some(sampler));samplers.push(sampler);
      self.gl.uniform_1_i32(self.gl.get_uniform_location(shader.program,name).as_ref(),unit as i32);
     }
     let vao=self.gl.create_vertex_array().map_err(error)?;self.gl.bind_vertex_array(Some(vao));let vbo=self.gl.create_buffer().map_err(error)?;self.gl.bind_buffer(glow::ARRAY_BUFFER,Some(vbo));
     let [u0,v0,u1,v1]=uv;
     let vertices:[f32;24]=[0.,0.,u0,v0, 1.,0.,u1,v0, 0.,1.,u0,v1, 0.,1.,u0,v1, 1.,0.,u1,v0, 1.,1.,u1,v1];
     self.gl.buffer_data_u8_slice(glow::ARRAY_BUFFER,bytemuck::cast_slice(&vertices),glow::STREAM_DRAW);
     for i in 0..2 {self.gl.enable_vertex_attrib_array(i);self.gl.vertex_attrib_pointer_f32(i,2,glow::FLOAT,false,16,(i*8) as i32);}
     self.gl.draw_arrays(glow::TRIANGLES,0,6);
     let mut pixels=vec![0u8;(width*height*4) as usize];self.gl.read_pixels(0,0,width as i32,height as i32,glow::RGBA,glow::UNSIGNED_BYTE,glow::PixelPackData::Slice(Some(&mut pixels)));
     let gl_error=self.gl.get_error();
     for (unit,s) in samplers.into_iter().enumerate(){self.gl.bind_sampler(unit as u32,None);self.gl.delete_sampler(s);}
     for t in temps {self.gl.delete_texture(t);}for b in buffers {self.gl.delete_buffer(b);}
     self.gl.delete_buffer(vbo);self.gl.delete_vertex_array(vao);self.gl.delete_framebuffer(fbo);self.gl.delete_texture(target);
     fn program(x:i32)->Option<glow::Program>{std::num::NonZeroU32::new(x as u32).map(glow::NativeProgram)}
     self.gl.use_program(program(previous_program));self.gl.bind_framebuffer(glow::FRAMEBUFFER,std::num::NonZeroU32::new(previous_fbo as u32).map(glow::NativeFramebuffer));self.gl.bind_vertex_array(std::num::NonZeroU32::new(previous_vao as u32).map(glow::NativeVertexArray));self.gl.bind_buffer(glow::ARRAY_BUFFER,std::num::NonZeroU32::new(previous_buffer as u32).map(glow::NativeBuffer));self.gl.active_texture(active as u32);
     self.gl.viewport(viewport[0],viewport[1],viewport[2],viewport[3]);if blend {self.gl.enable(glow::BLEND);}if stencil {self.gl.enable(glow::STENCIL_TEST);}if scissor {self.gl.enable(glow::SCISSOR_TEST);}self.gl.color_mask(color_mask[0]!=0,color_mask[1]!=0,color_mask[2]!=0,color_mask[3]!=0);
     if gl_error!=glow::NO_ERROR {return Err(error(format!("GL error {gl_error:#x}")));}
     Ok(pixels)
    }
 }
 pub fn filter_pb(&mut self, source:BitmapHandle, point:(u32,u32), size:(u32,u32), destination:BitmapHandle,dest:(u32,u32),filter:ruffle_render::filters::ShaderFilter<'static>) -> Result<Box<dyn SyncHandle>,BitmapError> {
    let s=as_registry_data(&source);let uv=[point.0 as f32/s.width as f32,point.1 as f32/s.height as f32,(point.0+size.0) as f32/s.width as f32,(point.1+size.1) as f32/s.height as f32];
    let mut args=filter.shader_args;
    // Flash ShaderFilter supplies its source as the first image parameter.
    let mut replaced=false;
    for a in &mut args {if let PixelBenderShaderArgument::ImageInput {texture,..}=a {if !replaced {*texture=Some(source.clone().into());replaced=true;}}}
    let pixels=self.execute_pb(&filter.shader,&args,size.0,size.1,true,uv)?;
    self.upload_pb(&destination,dest,size,&pixels);
    Ok(Box::new(QueueSyncHandle {texture:destination,bounds:PixelRegion {x_min:dest.0,y_min:dest.1,x_max:dest.0+size.0,y_max:dest.1+size.1}}))
 }
 fn upload_pb(&self,dest:&BitmapHandle,point:(u32,u32),size:(u32,u32),pixels:&[u8]) {
    unsafe {self.gl.bind_texture(glow::TEXTURE_2D,Some(as_registry_data(dest).texture()));self.gl.tex_sub_image_2d(glow::TEXTURE_2D,0,point.0 as i32,point.1 as i32,size.0 as i32,size.1 as i32,glow::RGBA,glow::UNSIGNED_BYTE,glow::PixelUnpackData::Slice(Some(pixels)));}
 }
 pub fn job_pb(&mut self,handle:PixelBenderShaderHandle,args:&[PixelBenderShaderArgument],target:&PixelBenderTarget)->Result<PixelBenderOutput,BitmapError>{
    match target {
     PixelBenderTarget::Bitmap(b)=>{let d=as_registry_data(b);let size=(d.width,d.height);let p=self.execute_pb(&handle,args,size.0,size.1,false,[0.,0.,1.,1.])?;self.upload_pb(b,(0,0),size,&p);Ok(PixelBenderOutput::Bitmap(Box::new(QueueSyncHandle{texture:b.clone(),bounds:PixelRegion{x_min:0,y_min:0,x_max:size.0,y_max:size.1}})))}
     PixelBenderTarget::Bytes{..}=>Err(error("float ByteArray output not yet implemented")),
    }
 }
}
fn pack(v:&PixelBenderType,f:&mut Vec<f32>,i:&mut Vec<i32>)->Result<(),BitmapError>{
 use PixelBenderType::*;
 match v {
 TFloat(a)=>f.extend([*a,0.,0.,0.]),TFloat2(a,b)=>f.extend([*a,*b,0.,0.]),TFloat3(a,b,c)=>f.extend([*a,*b,*c,0.]),TFloat4(a,b,c,d)=>f.extend([*a,*b,*c,*d]),
 TInt(a)|TBool(a)=>i.extend([*a as i32,0,0,0]),TInt2(a,b)|TBool2(a,b)=>i.extend([*a as i32,*b as i32,0,0]),TInt3(a,b,c)|TBool3(a,b,c)=>i.extend([*a as i32,*b as i32,*c as i32,0]),TInt4(a,b,c,d)|TBool4(a,b,c,d)=>i.extend([*a as i32,*b as i32,*c as i32,*d as i32]),
 TFloat2x2(a)=>f.extend(a),TFloat3x3(a)=>{for c in a.chunks(3){f.extend(c);f.push(0.);}},TFloat4x4(a)=>f.extend(a),_=>return Err(error("unsupported parameter type")),
 };Ok(())
}
