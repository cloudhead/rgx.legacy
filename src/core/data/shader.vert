#version 450

layout(set = 0, binding = 0) uniform Globals {
	mat4 ortho;
	mat4 transform;
} global;

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 color;

layout(location = 0) out vec2 f_uv;
layout(location = 1) out vec4 f_color;

void main() {
	f_color = color;
	f_uv = uv;

	gl_Position = global.ortho * global.transform * vec4(position, 0.0, 1.0);
}
