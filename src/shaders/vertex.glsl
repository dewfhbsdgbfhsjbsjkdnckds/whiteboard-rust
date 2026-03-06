#version 460 

layout(location = 0) in vec3 in_pos;
layout(location = 0) out vec4 out_color;

void main(){
	out_color = vec4(0.5, 0.0, 0.0, 1.0);
	gl_Position = vec4(in_pos, 1.0);
}
