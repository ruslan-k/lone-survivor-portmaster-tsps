#version 100

#ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
#else
    precision mediump float;
#endif

uniform mat4 view_matrix;
uniform mat4 world_matrix;
uniform vec4 mult_color;
uniform vec4 add_color;
uniform mat3 u_matrix;

uniform sampler2D u_texture;
uniform sampler2D u_backdrop;
uniform vec2 u_backdrop_size;
uniform int u_overlay;

// W3C/Flash separable Overlay, in straight RGB, then premultiplied source-over.
vec4 composite_overlay(vec4 s, vec4 d) {
    vec3 cs = s.a > 0.0 ? s.rgb / s.a : vec3(0.0);
    vec3 cd = d.a > 0.0 ? d.rgb / d.a : vec3(0.0);
    vec3 low = 2.0 * cs * cd;
    vec3 high = 1.0 - 2.0 * (1.0 - cs) * (1.0 - cd);
    vec3 blended = mix(low, high, step(vec3(0.5), cd));
    return vec4((1.0-s.a)*d.rgb + (1.0-d.a)*s.rgb + s.a*d.a*blended,
                s.a + d.a*(1.0-s.a));
}

varying vec2 frag_uv;

void main() {
    vec4 color = texture2D(u_texture, frag_uv);

    // Unmultiply alpha before apply color transform.
    if (color.a > 0.0) {
        color.rgb /= color.a;
        color = clamp(mult_color * color + add_color, 0.0, 1.0);
        float alpha = clamp(color.a, 0.0, 1.0);
        color = vec4(color.rgb * alpha, alpha);
    }

    gl_FragColor = u_overlay != 0
        ? composite_overlay(color, texture2D(u_backdrop, gl_FragCoord.xy / u_backdrop_size))
        : color;
}
