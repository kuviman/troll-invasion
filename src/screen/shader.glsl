#ifdef VERTEX
attribute vec2 a_pos;
uniform mat4 u_matrix;
uniform vec2 u_pos;
uniform float u_radius;
void main() {
    gl_Position = u_matrix * (vec4(a_pos * u_radius + u_pos, 0.0, 1.0));
}
#endif

#ifdef FRAGMENT
uniform vec4 u_color;
void main() {
    gl_FragColor = u_color;
}
#endif