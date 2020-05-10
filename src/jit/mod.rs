use std::io;
use std::collections::VecDeque;
use std::collections::HashMap;
use crate::register::Register;
use crate::r3000::R3000;
use crate::r3000::DelayedWrite;
use crate::r3000::Name;
use crate::cop0::Cop0;
use crate::memory::Memory;
use crate::memory::MemAction;
use crate::memory::MemResponse;
use crate::cd::CD;
use crate::gpu::GPU;
use crate::gte::GTE;
use crate::screen::Screen;
use crate::runnable::Runnable;

mod opcodes;

struct State {
  //these correspond to physical components
  r3000: R3000,
  cop0: Cop0,
  memory: Memory,
  gpu: GPU,
  gte: GTE,
  cd: CD,
  screen: Screen,

  //these are register writes due to memory loads which happen after one cycle
  delayed_writes: VecDeque<DelayedWrite>,
  modified_register: Option<Name>,
}

impl State {
  fn resolve_memresponse(&mut self, response: MemResponse) -> Register {
    todo!("")
  }
  fn resolve_memactions(&mut self, maybe_action: Option<Vec<MemAction>>) {
    todo!("")
  }
}

pub struct JIT {
  state: State,
  stubs: HashMap<Register, Vec<Box<dyn Fn(&mut State)>>>,

  //other members of interpreter
  next_pc: Option<Register>,
  i: u32,
}

impl Runnable for JIT {
  fn run(&mut self, n: Option<u32>, logging: bool) {
    let f = self.compile_opcode(0x00000000).unwrap();
    let g = self.compile_opcode(0x00000000).unwrap();
    f(&mut self.state);
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
    let delayed_writes = VecDeque::new();
    Ok(Self {
      state: State {
        r3000,
        cop0,
        memory,
        gpu,
        gte,
        cd,
        screen,

        delayed_writes,
        modified_register: None,
      },

      stubs: Default::default(),

      next_pc: None,
      i: 0,
    })
  }
  fn execute_stub(&mut self, stub: &Vec<Box<dyn Fn(&mut State)>>) {
    for f in stub {
      f(&mut self.state)
    }
  }
}
