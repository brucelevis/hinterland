#version 150 core

in vec3 a_Pos;
in vec2 a_BufPos;
out vec2 v_BufPos;

uniform b_VsLocals {
  mat4 u_Model;
  mat4 u_View;
  mat4 u_Proj;
};

void main() {
  v_BufPos = a_BufPos;
  gl_Position = u_Proj * u_View * u_Model * vec4(a_Pos, 1.0);
}
