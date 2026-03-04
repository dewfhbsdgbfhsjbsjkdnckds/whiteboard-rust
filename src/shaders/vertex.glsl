#version 460 

layout(location = 0) out vec4 out_color;

void main(){
	const vec2 pos[] = {
		vec2(-0.5, -0.5),
		vec2( 0.5, -0.5),
		vec2( 0.0,  0.5),
	};

	out_color = vec4(0.5, 0.0, 0.0, 1.0);
	gl_Position = vec4(pos[gl_VertexIndex], 0.0, 1.0);
}
