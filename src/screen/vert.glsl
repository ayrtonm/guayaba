#version 330 core

in ivec2 vert_pos;
in uvec3 vert_color;

out vec3 color;

void main() {
  float xpos = (float(vert_pos.x) / 512) - 1.0;
  float ypos = 1.0 - (float(vert_pos.y) / 256);
  gl_Position.xyzw = vec4(xpos, ypos, 0.0, 1.0);
  color = vec3(float(vert_color.r) / 255,
               float(vert_color.g) / 255,
               float(vert_color.b) / 255);
}
