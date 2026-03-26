#version 460 

layout(location = 0) in vec3 in_pos;
//layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform myUniform {
		mat4 matrix;
};



void main(){
	gl_Position = vec4(in_pos, 1.0) * matrix;
}
