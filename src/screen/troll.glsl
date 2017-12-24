varying vec2 v_vt;

#ifdef VERTEX
attribute vec2 a_pos;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform vec2 u_pos;
void main() {
    v_vt = (a_pos - vec2(-1.0, 0.0)) / 2.0;
    v_vt.y = 1.0 - v_vt.y;
    gl_Position = u_projection_matrix * (u_view_matrix * vec4(u_pos, 0.0, 1.0) + vec4(a_pos * 0.03, 0.0, 0.0));
}
#endif

#ifdef FRAGMENT
uniform sampler2D u_texture;
uniform vec4 u_color;
void main() {
    gl_FragColor = texture2D(u_texture, v_vt) * u_color;
}
#endif