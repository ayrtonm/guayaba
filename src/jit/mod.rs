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

struct Stub {
  operations: Vec<Box<dyn Fn(&mut State)>>,
  final_pc: Register,
}

impl Stub {
  fn operations(&self) -> &Vec<Box<dyn Fn(&mut State)>> {
    &self.operations
  }
  fn final_pc(&self) -> Register {
    self.final_pc
  }
}

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
  stubs: HashMap<Register, Stub>,

  //other members of interpreter
  next_pc: Option<Register>,
  i: u32,
}

impl Runnable for JIT {
  fn run(&mut self, n: Option<u32>, logging: bool) {
    loop {
      let maybe_stub = self.stubs.get(&self.state.r3000.pc());
      match maybe_stub {
        Some(stub) => {
          let operations = stub.operations();
          for f in operations {
            self.state.r3000.flush_write_cache(&mut self.state.delayed_writes,
                                               &mut self.state.modified_register);
            f(&mut self.state);
            match self.state.gpu.exec_next_gp0_command() {
              Some(object) => self.state.screen.draw(object),
              None => (),
            };
            self.state.cd.exec_command();
          }
          *self.state.r3000.pc_mut() = stub.final_pc();
        },
        None => {
          let mut new_operations = vec![];
          let start = self.state.r3000.pc();
          let op = self.state.resolve_memresponse(self.state.memory.read_word(start));
          let mut compiled = self.compile_opcode(op);
          while compiled.is_some() {
            new_operations.push(compiled.take().expect(""));
            *self.state.r3000.pc_mut() += 4;
            let op = self.state.resolve_memresponse(self.state.memory.read_word(self.state.r3000.pc()));
            compiled = self.compile_opcode(op);
          }
          let stub = Stub { operations: new_operations, final_pc: self.state.r3000.pc() };
          self.stubs.insert(start, stub);
        },
      }
    }
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
}
