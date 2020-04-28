use std::ffi::CString;
use std::ffi::CStr;
use gl::types::GLuint;
use gl::types::GLenum;

pub struct Shader {
  id: GLuint,
}

impl Shader {
  pub fn id(&self) -> GLuint {
    self.id
  }
  pub fn new_vertex_shader(source: &CStr) -> Shader {
    Shader {
      id: Shader::from_source(source, gl::VERTEX_SHADER).expect("Expected a shader ID")
    }
  }
  pub fn new_fragment_shader(source: &CStr) -> Shader {
    Shader {
      id: Shader::from_source(source, gl::FRAGMENT_SHADER).expect("Expected a shader ID")
    }
  }
  fn from_source(source: &CStr, kind: GLenum) -> Option<GLuint> {
    unsafe {
      let id = gl::CreateShader(kind);
      gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
      gl::CompileShader(id);
      let mut success = 1;
      gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
      if success == 0 {
        None
      } else {
        Some(id)
      }
    }
  }
}
