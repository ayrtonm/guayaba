extern crate sdl2;
extern crate gl;

pub struct Screen {
  sdl: sdl2::Sdl,
  video_subsystem: sdl2::VideoSubsystem,
  window: sdl2::video::Window,
  gl_context: sdl2::video::GLContext,
  event_pump: sdl2::EventPump,
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
    unsafe {
      gl::ClearColor(0.3, 0.3, 0.5, 1.0);
      gl::Clear(gl::COLOR_BUFFER_BIT);
    }
    window.gl_swap_window();
    Screen {
      sdl,
      video_subsystem,
      window,
      gl_context,
      event_pump,
    }
     
  }
  pub fn event_pump(&mut self) -> &mut sdl2::EventPump {
    &mut self.event_pump
  }
}



