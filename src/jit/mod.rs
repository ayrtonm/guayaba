use std::io;
use crate::r3000::R3000;
use crate::cop0::Cop0;
use crate::memory::Memory;
use crate::cd::CD;
use crate::gpu::GPU;
use crate::gte::GTE;
use crate::screen::Screen;
use crate::runnable::Runnable;

pub struct JIT {
  //these correspond to physical components
  r3000: R3000,
  cop0: Cop0,
  memory: Memory,
  gpu: GPU,
  gte: GTE,
  cd: CD,
  screen: Screen,
}

impl Runnable for JIT {
  fn run(&mut self, n: Option<u32>, logging: bool) {
    println!("ran the JIT");
  }
}

impl JIT {
  pub fn new(bios_filename: &String, infile: Option<&String>, gpu_logging: bool,
             wx: u32, wy: u32) -> io::Result<Self> {
    let r3000 = R3000::new();
    let cop0 = Default::default();
    let memory = Memory::new(bios_filename)?;
    let gpu = GPU::new(gpu_logging);
    let gte = Default::default();
    let cd = CD::new(infile);
    let screen = Screen::new(wx, wy);
    Ok(Self {
      r3000,
      cop0,
      memory,
      gpu,
      gte,
      cd,
      screen,
    })
  }
}
