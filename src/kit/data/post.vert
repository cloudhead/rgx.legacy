#version 450

layout(set = 0, binding = 0) uniform Globals {
	vec4 color;
} global;

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 f_uv;
layout(location = 1) out vec4 f_color;

void main() {
	f_uv = uv;
	f_color = global.color;

	gl_Position = vec4(position, 0.0, 1.0);
}
