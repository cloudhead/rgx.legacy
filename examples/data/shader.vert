#version 450

layout(set = 0, binding = 0) uniform Locals {
	mat4 ortho;
	mat4 transform;
} mvp;

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec2 texcoord;

layout(location = 0) out vec2 f_texcoord;
layout(location = 1) out vec4 f_color;

void main() {
	f_color = color;
	f_texcoord = texcoord;

	gl_Position = mvp.ortho * mvp.transform * vec4(position, 0.0, 1.0);
}
