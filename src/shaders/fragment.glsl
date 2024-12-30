#version 140

out vec4 color;
uniform sampler2D tex;

in vec2 v_tex_coords;




void main() {

    color = texture(tex[0], v_tex_coords);
    //color = vec4(0.4,0.25,1.0, color.a);
    
    //color.rgb *= v_brightness;
    //color = vec4(0.5,0.5,0.0,1.0);

    if (color.a == 0.0) { discard; }
    //color = vec4(1.0,1.0,1.0,1.0);

}