#version 460

layout(location = 0) out vec4 FragColor;

// i have no idea why set 3 works, but others dont
layout(set = 3, binding = 0) uniform inputColourUniform {
		vec4 incolor;
		//float hello;
};

void main(){
	FragColor = incolor;
	//FragColor = vec4(0.5, 0.0, 0.0, 1.0);
	//FragColor = vec4(hello, 0.0, 0.0, 1.0);
}
