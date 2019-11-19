#version 450

layout(set = 0, binding = 0) uniform Globals {
	mat4 ortho;
	mat4 transform;
} global;

layout(set = 1, binding = 0) uniform Model {
	mat4 transform;
} model;

layout(location = 0) in vec3  position;
layout(location = 1) in vec2  uv;
layout(location = 2) in vec4  color;
layout(location = 3) in float opacity;

layout(location = 0) out vec2  f_uv;
layout(location = 1) out vec4  f_color;
layout(location = 2) out float f_opacity;

void main() {
	f_color = color;
	f_uv = uv;
	f_opacity = opacity;

	gl_Position = global.ortho * global.transform * model.transform * vec4(position, 1.0);
}
