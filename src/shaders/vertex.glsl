#version 460 

layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec2 tex_coord;

layout(location = 0) out vec2 out_tex_coord;

layout(set = 1, binding = 0) uniform myUniform {
		mat4 matrix;
};



void main(){
	out_tex_coord = tex_coord;
	gl_Position = vec4(in_pos, 1.0) * matrix;
}
