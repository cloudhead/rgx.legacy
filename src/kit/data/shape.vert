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
layout(location = 2) in float angle;

layout(location = 0) out vec4 f_color;

// vec2 new_pos(vec2 oldpos, float angle) {
//   float x = oldpos.x * cos(angle) + oldpos.y * sin(angle);
//   float y = oldpos.x * -sin(angle) + oldpos.y * cos(angle);
//   return vec2(x, y);
// }
// 
// vec4 rotate(vec2 p, vec3 axis, float angle) {
//   mat4 m = rotationMatrix(axis, angle);
//   return m * vec4(p, 0.0, 1.0);
// }
// 
// mat2 rotate2d(float _angle) {
//   return mat2(cos(_angle), -sin(_angle),
//               sin(_angle),  cos(_angle));
// }

mat4 rotationMatrix(vec3 axis, float angle)
{
    axis = normalize(axis);
    float s = sin(angle);
    float c = cos(angle);
    float oc = 1.0 - c;

    return mat4(oc * axis.x * axis.x + c,           oc * axis.x * axis.y - axis.z * s,  oc * axis.z * axis.x + axis.y * s,  0.0,
                oc * axis.x * axis.y + axis.z * s,  oc * axis.y * axis.y + c,           oc * axis.y * axis.z - axis.x * s,  0.0,
                oc * axis.z * axis.x - axis.y * s,  oc * axis.y * axis.z + axis.x * s,  oc * axis.z * axis.z + c,           0.0,
                0.0,                                0.0,                                0.0,                                1.0);
}

void main() {
	f_color = color;

  mat4 rotated = rotationMatrix(vec3(0.0, 0.0, 1.0), angle);

  gl_Position = global.ortho * global.transform * model.transform * rotated * vec4(position, 0.0, 1.0);

  // float ct = cos(angle);
  // float st = sin(angle);
	// vec4 p = vec4(
	//     position.x * ct - position.y * st,
	//     position.x * st + position.y * ct,
	//     0.0, 1.0
  //   );
  // vec2 pos = rotate2d(angle) * position;

  // gl_Position = global.ortho * global.transform * model.transform * vec4(pos, 0.0, 1.0);
  // gl_Position = global.ortho * global.transform * model.transform * vec4(new_pos(position, angle), 0.0, 1.0);
  // gl_Position = global.ortho * global.transform * model.transform * rotate(position, vec3(0.0, 0.0, 1.0), angle);

  // gl_Position = global.ortho * global.transform * model.transform * rotate(position, vec3(0.0, 0.0, 1.0), angle);
  // gl_Position *= rotated;

	// gl_Position = global.ortho * global.transform * model.transform * vec4(new_pos(position, angle), 0.0, 1.0);
}
