#version 460

layout(location = 0) in vec2 texCoord;
layout(location = 0) out vec4 FragColor;

layout(set = 2, binding = 0) uniform sampler2D texSampler;

// i have no idea why set 3 works, but others dont
layout(set = 3, binding = 0) uniform inputColourUniform {
		vec4 incolor;
};


void main(){
	//FragColor = incolor;
	FragColor = texture(texSampler, texCoord);
}
