varying vec2 v_pos;

#ifdef VERTEX
attribute vec2 a_pos;
uniform mat4 u_matrix;
uniform vec2 u_pos;
uniform float u_radius;
void main() {
    v_pos = a_pos * u_radius + u_pos;
    gl_Position = u_matrix * (vec4(a_pos * u_radius + u_pos, 0.0, 1.0));
}
#endif

#ifdef FRAGMENT
uniform float use_texture;
uniform sampler2D u_texture;
uniform vec4 u_color;
void main() {
    if (use_texture > 0.5) {
        gl_FragColor = texture2D(u_texture, v_pos / 3.0) * u_color;
    } else {
        gl_FragColor = u_color;
    }
}
#endif