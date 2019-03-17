#version 450

layout(set = 0, binding = 1) uniform texture2D tex;
layout(set = 0, binding = 2) uniform sampler   sam;

layout(location = 0) in  vec2 f_texcoord;
layout(location = 1) in  vec4 f_color;

layout(location = 0) out vec4 fragColor;

void main() {
	fragColor = mix(
		texture(sampler2D(tex, sam), f_texcoord),
		f_color,
		0.5
	);
}
