#version 450

layout(set = 1, binding = 0) uniform texture2D tex;
layout(set = 1, binding = 1) uniform sampler   sam;

layout(location = 0) in  vec2 f_uv;
layout(location = 1) in  vec4 f_color;

layout(location = 0) out vec4 fragColor;

void main() {
	vec4 texel = texture(sampler2D(tex, sam), vec2(f_uv.s, 1 - f_uv.t));

	fragColor = vec4(
		mix(texel.rgb, f_color.rgb, f_color.a),
		texel.a
	);
}
