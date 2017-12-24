varying vec2 v_vt;

#ifdef VERTEX
attribute vec2 a_pos;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform vec2 u_pos;
uniform vec2 u_size;
uniform float u_up;
void main() {
    v_vt = (a_pos - vec2(-1.0, 0.0)) / 2.0;
    v_vt.y = 1.0 - v_vt.y;
    if (u_up > 0.5) {
        gl_Position = u_projection_matrix * u_view_matrix * (vec4(u_pos, 0.0, 1.0) + vec4(a_pos.x * u_size.x, 0.0, a_pos.y * u_size.y, 0.0));
    } else {
        gl_Position = u_projection_matrix * (u_view_matrix * vec4(u_pos, 0.0, 1.0) + vec4(a_pos * 0.03 * u_size, 0.0, 0.0));
    }
}
#endif

#ifdef FRAGMENT
uniform sampler2D u_texture;
uniform vec4 u_color;
void main() {
    gl_FragColor = texture2D(u_texture, v_vt) * u_color;
    if (gl_FragColor.w < 0.5) {
        discard;
    }
}
#endif