#ifdef VERTEX
attribute vec2 a_pos;
void main() {
    gl_Position = vec4(a_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT
uniform vec4 u_color;
void main() {
    gl_FragColor = u_color;
}
#endif