use std::ffi::CString;
use std::ffi::CStr;
use gl::types::GLuint;
extern crate sdl2;
extern crate gl;

mod shader;
use shader::Shader;

pub enum Drawable {
  Line,
  Rectangle,
  Polygon,
}

pub struct Screen {
  sdl: sdl2::Sdl,
  video_subsystem: sdl2::VideoSubsystem,
  window: sdl2::video::Window,
  gl_context: sdl2::video::GLContext,
  event_pump: sdl2::EventPump,

  vertex_shader: Shader,
  fragment_shader: Shader,
  program_id: GLuint,
}

impl Screen {
  pub fn new(wx: u32, wy: u32) -> Self {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let window = video_subsystem.window("RSX", wx, wy)
                                .opengl()
                                .resizable()
                                .build()
                                .unwrap();
    let mut event_pump = sdl.event_pump().unwrap();
    let gl_context = window.gl_create_context().unwrap();
    let gl = gl::load_with(
      |s| {
        video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
      });
    let vertex_source = CString::new(include_str!("vert.glsl"))
                                .expect("Could not turn vertex shader into a CString");
    let fragment_source = CString::new(include_str!("triangle.frag"))
                                  .expect("Could not turn fragment shader into a CString");
    let vertex_shader = Shader::new_vertex_shader(&vertex_source);
    let fragment_shader = Shader::new_fragment_shader(&fragment_source);
    let program_id = unsafe {
      gl::CreateProgram()
    };
    unsafe {
      gl::AttachShader(program_id, vertex_shader.id());
      gl::AttachShader(program_id, fragment_shader.id());
      gl::LinkProgram(program_id);
      gl::DetachShader(program_id, vertex_shader.id());
      gl::DetachShader(program_id, fragment_shader.id());
      gl::UseProgram(program_id);
    }
    unsafe {
      gl::ClearColor(0.3, 0.3, 0.5, 1.0);
      gl::Clear(gl::COLOR_BUFFER_BIT);
    }
    ////////////////////////////////////////////////////////////
    //let vertices: Vec<u32> = vec![256, 128,  255, 0, 0,
    //                              768, 128,  0, 255, 0,
    //                              512, 256,  0, 0, 255];
    let vertices: Vec<f32> = vec![256.0, 384.0,     255.0, 0.0,   0.0,
                                  768.0, 384.0,    0.0,   255.0, 0.0,
                                  512.0, 256.0,    0.0,   0.0, 255.0];
    let mut vbo: gl::types::GLuint = 0;
    unsafe {
      gl::GenBuffers(1, &mut vbo);
      gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
      gl::BufferData(
        gl::ARRAY_BUFFER,
        (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
        vertices.as_ptr() as *const gl::types::GLvoid,
        gl::STATIC_DRAW);
      gl::BindBuffer(gl::ARRAY_BUFFER, 0);

    }
    let mut vao: gl::types::GLuint = 0;
    unsafe {
      gl::GenVertexArrays(1, &mut vao);
      gl::BindVertexArray(vao);
      gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
      gl::EnableVertexAttribArray(0);
      gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE,
        (5 * std::mem::size_of::<f32>()) as gl::types::GLint,
        std::ptr::null());
      gl::EnableVertexAttribArray(1);
      gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE,
        (5 * std::mem::size_of::<f32>()) as gl::types::GLint,
        (2 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid);
      gl::BindBuffer(gl::ARRAY_BUFFER, 0);
      gl::BindVertexArray(0);
    }
    unsafe {
      gl::BindVertexArray(vao);
      gl::DrawArrays(gl::TRIANGLES, 0, 3);
    }
    ////////////////////////////////////////////////////////////
    window.gl_swap_window();
    Screen {
      sdl,
      video_subsystem,
      window,
      gl_context,
      event_pump,
      vertex_shader,
      fragment_shader,
      program_id,
    }
     
  }
  pub fn draw(&mut self, object: Drawable) {
  }
  pub fn event_pump(&mut self) -> &mut sdl2::EventPump {
    &mut self.event_pump
  }
}

impl Drop for Screen {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteProgram(self.program_id)
    }
  }
}
