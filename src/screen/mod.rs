use std::ffi::CString;
use gl::types::GLuint;
extern crate sdl2;
extern crate gl;

mod shader;
use shader::Shader;

#[derive(Debug)]
pub struct Drawable {
  positions: Vec<i16>,
  colors: Vec<i16>,
}

impl Drawable {
  pub fn new(positions: Vec<i16>, colors: Vec<i16>) -> Self {
    Drawable {
      positions,
      colors,
    }
  }
  pub fn n_points(&self) -> i32 {
    (self.positions.len() as i32)/2
  }
  pub fn positions(&self) -> &Vec<i16> {
    &self.positions
  }
  pub fn colors(&self) -> &Vec<i16> {
    &self.colors
  }
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
    let event_pump = sdl.event_pump().unwrap();
    let gl_context = window.gl_create_context().unwrap();
    let gl = gl::load_with(
      |s| {
        video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
      });
    let vertex_source = CString::new(include_str!("vert.glsl"))
                                .expect("Could not turn vertex shader into a CString");
    let fragment_source = CString::new(include_str!("frag.glsl"))
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
      gl::ClearColor(0.0, 0.0, 0.0, 1.0);
      gl::Clear(gl::COLOR_BUFFER_BIT);
    }
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
    let vertices: Vec<i16> = match object.n_points() {
      3 => {
        vec![object.positions().clone(), object.colors().clone()].into_iter().flatten().collect()
      },
      4 => {
        let pos_t1 = object.positions().clone().into_iter().skip(0).cycle().take(3 * 2);
        let pos_t2 = object.positions().clone().into_iter().skip(2).cycle().take(3 * 2);
        let col_t1 = object.colors().clone().into_iter().skip(0).cycle().take(3 * 3);
        let col_t2 = object.colors().clone().into_iter().skip(3).cycle().take(3 * 3);
        vec![pos_t1, pos_t2, col_t1, col_t2].into_iter().flatten().collect()
      },
      _ => {
        panic!("drawing this object is not implemented {:?}", object);
      },
    };
    let n_vertices = vertices.len()/5;
    let mut vbo: gl::types::GLuint = 0;
    unsafe {
      gl::GenBuffers(1, &mut vbo);
      gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
      gl::BufferData(
        gl::ARRAY_BUFFER,
        (vertices.len() * std::mem::size_of::<i16>()) as gl::types::GLsizeiptr,
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
      gl::VertexAttribPointer(0, 2, gl::SHORT, gl::FALSE,
        (2 * std::mem::size_of::<i16>()) as gl::types::GLint,
        std::ptr::null());
      gl::EnableVertexAttribArray(1);
      gl::VertexAttribPointer(1, 3, gl::SHORT, gl::FALSE,
        (3 * std::mem::size_of::<i16>()) as gl::types::GLint,
        (2 * n_vertices * std::mem::size_of::<i16>()) as *const gl::types::GLvoid);
      gl::BindBuffer(gl::ARRAY_BUFFER, 0);
      gl::BindVertexArray(0);
    }
    unsafe {
      gl::BindVertexArray(vao);
      gl::DrawArrays(gl::TRIANGLES, 0, n_vertices as i32);
    }
    self.window.gl_swap_window();
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
