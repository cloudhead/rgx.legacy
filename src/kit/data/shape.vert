#version 450

layout(set = 0, binding = 0) uniform Globals {
	mat4 ortho;
	mat4 transform;
} global;

layout(set = 1, binding = 0) uniform Model {
	mat4 transform;
} model;

layout(location = 0) in vec2 position;
layout(location = 1) in float angle;
layout(location = 2) in vec2 center;
layout(location = 3) in vec4 color;

layout(location = 0) out vec4 f_color;

mat4 rotationZ(float angle ) {
	return mat4(
		cos(angle), -sin(angle),  0, 0,
		sin(angle),  cos(angle),  0, 0,
		0,          0,            1, 0,
		0,          0,            0, 1
	);
}

mat2 rotate2d(float angle) {
	float s = sin(a);
	float c = cos(a);
	return mat2(c, -s, s, c);
}

vec2 rotate(vec2 position, vec2 around, float angle) {
	mat2 m = rotate2d(angle);
	vec2 rotated = m * (position - around);
	return rotated + around;
}

void main() {
	f_color = color;

	vec2 r = rotate(position, center, angle);

	gl_Position = global.ortho * global.transform * model.transform * vec4(r, 0.0, 1.0);
}
