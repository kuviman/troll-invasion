varying vec4 v_color;
varying vec2 v_vt;
#ifdef VERTEX
attribute vec2 a_vt;
attribute vec2 a_pos;
attribute vec4 a_color;
void main() {
    v_vt = a_vt;
    v_color = a_color;
    vec2 pos = 2.0 * a_pos / u_framebuffer_size;
    if (a_vt.x > -0.5) {
        pos = vec2(pos.x - 1.0, 1.0 - pos.y);
    }
    gl_Position = vec4(pos, 0.0, 1.0);
}
#endif
#ifdef FRAGMENT
uniform sampler2D u_glyph_cache;
void main() {
    if (v_vt.x < -0.5) {
        gl_FragColor = v_color;
    } else {
        gl_FragColor = texture2D(u_glyph_cache, v_vt) * v_color;
    }
}
#endif