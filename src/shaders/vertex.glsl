#version 140

in vec3 pos;
in vec2 tex_coords;
in float rotation;

uniform mat3 ortho_mat;
uniform vec2 translation;


out vec2 v_tex_coords;


void main() {
    v_tex_coords = tex_coords;



    vec3 translated = vec3(pos.xy - translation, pos.z);
    vec3 clipped = ortho_mat * translated;

    gl_Position = vec4(clipped.xy,pos.z, 1.0);
    //gl_Position = vec4(pos, 1.0);
}