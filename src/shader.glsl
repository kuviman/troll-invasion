#ifdef VERTEX
attribute vec2 a_quad_pos;
uniform vec2 p1, p2;
void main() {
    gl_Position = vec4(p1 + (p2 - p1) * a_quad_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT
uniform vec4 u_color;
void main() {
    gl_FragColor = u_color;
}
#endif