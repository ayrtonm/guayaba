#version 330 core

layout (location = 0) in vec2 vert_position;
layout (location = 1) in vec3 vert_color;

out VS_OUTPUT {
  vec3 Color;
} OUT;

void main() {
  float xpos = (float(vert_position.x) / 320) - 1.0;
  float ypos = 1.0 - (float(vert_position.y) / 240);
  gl_Position = vec4(xpos, ypos, 0.0, 1.0);
  OUT.Color = vec3(float(vert_color.r) / 255,
               float(vert_color.g) / 255,
               float(vert_color.b) / 255);
}
