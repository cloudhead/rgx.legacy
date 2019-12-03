#version 450

layout(set = 0, binding = 0) uniform Globals {
	mat4 ortho;
	mat4 transform;
} global;

layout(location = 0) in vec3 position;
layout(location = 1) in float angle;
layout(location = 2) in vec2 center;
layout(location = 3) in vec4 color;

layout(location = 0) out vec4 f_color;

// Convert an sRGB color to linear space.
vec3 linearize(vec3 srgb) {
	bvec3 cutoff = lessThan(srgb, vec3(0.04045));
	vec3 higher = pow((srgb + vec3(0.055)) / vec3(1.055), vec3(2.4));
	vec3 lower = srgb / vec3(12.92);

	return mix(higher, lower, cutoff);
}

mat2 rotation2d(float angle) {
	float s = sin(angle);
	float c = cos(angle);
	return mat2(c, -s, s, c);
}

vec2 rotate(vec2 position, vec2 around, float angle) {
	mat2 m = rotation2d(angle);
	vec2 rotated = m * (position - around);
	return rotated + around;
}

void main() {
	vec2 r = rotate(position.xy, center, angle);

	f_color = vec4(linearize(color.rgb), color.a);
	gl_Position = global.ortho * global.transform * vec4(r, position.z, 1.0);
}
