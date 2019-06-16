#version 450

layout(set = 0, binding = 0) uniform Globals {
	mat4 ortho;
	mat4 transform;
} global;

layout(set = 1, binding = 0) uniform Model {
	mat4 transform;
} model;

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 f_color;

void main() {
	f_color = color;

	gl_Position = global.ortho * global.transform * model.transform * vec4(position, 0.0, 1.0);
}
