#version 460

layout(location = 0) out vec4 FragColor;

layout(location = 0) in vec4 vertexColor;

void main(){
	FragColor = vertexColor;
}
