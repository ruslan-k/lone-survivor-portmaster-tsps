//! Destination-reading Overlay for bitmap draws (scanlines and film grain).
//! Resolve MSAA first, never sample the currently attached framebuffer.
use super::*;
use std::num::NonZeroU32;
pub(super) struct Backdrop {
    gl: Arc<glow::Context>, texture: glow::Texture, framebuffer: glow::Framebuffer, width:i32,height:i32,
}
impl Drop for Backdrop {fn drop(&mut self){unsafe{self.gl.delete_framebuffer(self.framebuffer);self.gl.delete_texture(self.texture);}}}
pub(super) struct Snapshot {
    gl: Arc<glow::Context>,
    previous_texture: Option<glow::Texture>,
    previous_sampler: Option<glow::Sampler>,
}
impl Snapshot {
 pub unsafe fn capture(gl: Arc<glow::Context>, width:i32,height:i32, cache: &mut Option<Backdrop>)->Result<Self,String>{
  unsafe {
   let read=NonZeroU32::new(gl.get_parameter_i32(glow::READ_FRAMEBUFFER_BINDING) as u32).map(glow::NativeFramebuffer);
   let draw=NonZeroU32::new(gl.get_parameter_i32(glow::DRAW_FRAMEBUFFER_BINDING) as u32).map(glow::NativeFramebuffer);
   let active=gl.get_parameter_i32(glow::ACTIVE_TEXTURE);
   gl.active_texture(glow::TEXTURE1);
   let previous_texture=NonZeroU32::new(gl.get_parameter_i32(glow::TEXTURE_BINDING_2D) as u32).map(glow::NativeTexture);
   let previous_sampler=NonZeroU32::new(gl.get_parameter_i32(glow::SAMPLER_BINDING) as u32).map(glow::NativeSampler);
   let resources = if let Some(b)=cache.as_ref().filter(|b|b.width==width && b.height==height){Ok((b.texture,b.framebuffer))}else{
    gl.create_texture().and_then(|texture| match gl.create_framebuffer(){Ok(framebuffer)=>{*cache=Some(Backdrop{gl:gl.clone(),texture,framebuffer,width,height});gl.bind_texture(glow::TEXTURE_2D,Some(texture));gl.tex_image_2d(glow::TEXTURE_2D,0,glow::RGBA8 as i32,width,height,0,glow::RGBA,glow::UNSIGNED_BYTE,glow::PixelUnpackData::Slice(None));Ok((texture,framebuffer))},Err(e)=>{gl.delete_texture(texture);Err(e)}})
   };
   let (texture,framebuffer)=match resources {Ok(r)=>r,Err(e)=>{gl.active_texture(active as u32);return Err(e)}};
   gl.bind_sampler(1,None);gl.bind_texture(glow::TEXTURE_2D,Some(texture));
   for (p,v) in [(glow::TEXTURE_MIN_FILTER,glow::NEAREST),(glow::TEXTURE_MAG_FILTER,glow::NEAREST),(glow::TEXTURE_WRAP_S,glow::CLAMP_TO_EDGE),(glow::TEXTURE_WRAP_T,glow::CLAMP_TO_EDGE)]{gl.tex_parameter_i32(glow::TEXTURE_2D,p,v as i32);}
   gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER,Some(framebuffer));gl.framebuffer_texture_2d(glow::DRAW_FRAMEBUFFER,glow::COLOR_ATTACHMENT0,glow::TEXTURE_2D,Some(texture),0);
   let complete=gl.check_framebuffer_status(glow::DRAW_FRAMEBUFFER)==glow::FRAMEBUFFER_COMPLETE;
   let scissor=gl.is_enabled(glow::SCISSOR_TEST);gl.disable(glow::SCISSOR_TEST);
   if complete {gl.bind_framebuffer(glow::READ_FRAMEBUFFER,draw);gl.blit_framebuffer(0,0,width,height,0,0,width,height,glow::COLOR_BUFFER_BIT,glow::NEAREST);}
   if scissor {gl.enable(glow::SCISSOR_TEST);}
   gl.bind_framebuffer(glow::READ_FRAMEBUFFER,read);gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER,draw);gl.active_texture(active as u32);
   let s=Self{gl,previous_texture,previous_sampler};
   if !complete {return Err("Overlay framebuffer incomplete".into());}
   Ok(s)
  }
 }
}
impl Drop for Snapshot {
 fn drop(&mut self){unsafe{
  let active=self.gl.get_parameter_i32(glow::ACTIVE_TEXTURE);self.gl.active_texture(glow::TEXTURE1);self.gl.bind_texture(glow::TEXTURE_2D,self.previous_texture);self.gl.bind_sampler(1,self.previous_sampler);self.gl.active_texture(active as u32);
 }}
}
impl GlowRenderBackend {
 pub fn overlay_self_test(&mut self)->Result<(),String>{
  // Real shader/draw/resolve/readback, not just a CPU formula test.
  self.set_viewport_dimensions(ViewportDimensions{width:2,height:2,scale_factor:1.0});
  let cases=[([40u8,100,200,255],[180u8,70,120,255],0.85f32),([0,0,0,255],[255,255,255,255],1.0),([255,255,255,255],[0,0,0,255],1.0),([120,150,80,255],[40,80,120,128],0.4)];
  for (n,(d,s,alpha)) in cases.into_iter().enumerate(){
   let bitmap=self.register_bitmap(Bitmap::new(1,1,BitmapFormat::Rgba,s.to_vec())).map_err(|e|e.to_string())?;
   self.begin_frame(Color{r:d[0],g:d[1],b:d[2],a:d[3]});
   self.push_blend_mode(RenderBlendMode::Builtin(BlendMode::Overlay));
   let mut transform=Transform::default();transform.matrix=Matrix::scale(2.0,2.0);transform.color_transform.a_multiply=swf::Fixed8::from_f32(alpha);
   self.render_bitmap(bitmap,transform,false,PixelSnapping::Never);self.pop_blend_mode();self.end_frame();
   let mut got=[0u8;16];unsafe{self.gl.read_pixels(0,0,2,2,glow::RGBA,glow::UNSIGNED_BYTE,glow::PixelPackData::Slice(Some(&mut got)));}
   let sa=s[3] as f32/255.0*alpha;
   let mut expected=[0u8;4];for c in 0..3{let cd=d[c] as f32/255.0;let cs=if s[3]>0 {s[c] as f32/s[3] as f32}else{0.0};let blend=if cd<0.5{2.0*cs*cd}else{1.0-2.0*(1.0-cs)*(1.0-cd)};expected[c]=((cd*(1.0-sa)+sa*blend)*255.0).round() as u8;}expected[3]=255;
   eprintln!("overlay_selftest case={n} got={:?} expected={expected:?}",&got[..4]);
   if got.chunks(4).any(|pixel|pixel.iter().zip(expected).any(|(a,b)|(*a as i32-b as i32).abs()>2)){return Err(format!("Overlay case {n} mismatch {got:?} vs {expected:?}"));}
   unsafe{let e=self.gl.get_error();if e!=glow::NO_ERROR{return Err(format!("GL error {e:#x}"));}}
  }
  // Regression: a 1x1 dirty region must not resize/truncate its 4x4 backing texture.
  let mut expected=vec![20u8,80,120,255].repeat(16);
  let bitmap=self.register_bitmap(Bitmap::new(4,4,BitmapFormat::Rgba,expected.clone())).map_err(|e|e.to_string())?;
  let _=as_registry_data(&bitmap).texture();
  expected[20..24].copy_from_slice(&[180,40,90,255]);
  self.update_texture(&bitmap,Bitmap::new(4,4,BitmapFormat::Rgba,expected.clone()),PixelRegion{x_min:1,y_min:1,x_max:2,y_max:2}).map_err(|e|e.to_string())?;
  unsafe {
   let f=self.gl.create_framebuffer()?;self.gl.bind_framebuffer(glow::FRAMEBUFFER,Some(f));self.gl.framebuffer_texture_2d(glow::FRAMEBUFFER,glow::COLOR_ATTACHMENT0,glow::TEXTURE_2D,Some(as_registry_data(&bitmap).texture()),0);
   let mut got=vec![0u8;64];self.gl.read_pixels(0,0,4,4,glow::RGBA,glow::UNSIGNED_BYTE,glow::PixelPackData::Slice(Some(&mut got)));self.gl.bind_framebuffer(glow::FRAMEBUFFER,None);self.gl.delete_framebuffer(f);
   if got!=expected {eprintln!("partial_texture_selftest got={got:?} expected={expected:?}");return Err("partial texture upload truncated the bitmap".into());}
  }
  eprintln!("partial_texture_selftest PASS backing=4x4 dirty=1x1 unchanged_pixels=15");
  eprintln!("overlay_selftest PASS cases=4 tolerance=2");Ok(())
 }
}
