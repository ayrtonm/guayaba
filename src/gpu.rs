use crate::memory::MB;
use crate::register::Register;

struct Command {
}

pub struct GPU {
  vram: Box<[u8]>,
  command_buffer: Vec<Command>,
}

impl GPU {
  pub fn new() -> Self {
    let command_buffer = Vec::new();
    GPU {
      vram: vec![0; MB].into_boxed_slice(),
      command_buffer,
    }
  }
}
