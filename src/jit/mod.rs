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
mod jumps;
mod handle_dma;

type Stub = Vec<Box<dyn Fn(&mut State)>>;

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
  next_pc: Register,
  delayed_writes: VecDeque<DelayedWrite>,
  modified_register: Option<Name>,
}

impl State {
  fn resolve_memresponse(&mut self, response: MemResponse) -> Register {
    match response {
      MemResponse::Value(value) => value,
      MemResponse::GPUREAD => self.gpu.gpuread(),
      MemResponse::GPUSTAT => self.gpu.gpustat(),
      MemResponse::CDResponse => self.cd.read_response(),
    }
  }
  fn resolve_memactions(&mut self, maybe_action: Option<Vec<MemAction>>) {
    maybe_action.map(
      |actions| {
        actions.into_iter().for_each(
          |action| {
            match action {
              MemAction::DMA(transfer) => {
                self.handle_dma(transfer);
              },
              MemAction::GpuGp0(value) => self.gpu.write_to_gp0(value),
              MemAction::GpuGp1(value) => self.gpu.write_to_gp1(value),
              MemAction::CDCmd(value) => {
                self.cd.send_command(value);
              },
              MemAction::CDParam(value) => {
                self.cd.send_parameter(value);
              },
              MemAction::Interrupt(irq) => {
                self.cop0.request_interrupt(irq);
              },
            }
          })
      }
    );
  }
}

pub struct JIT {
  state: State,
  stubs: HashMap<Register, Stub>,

  i: u32,
}

impl Runnable for JIT {
  fn run(&mut self, n: Option<u32>, logging: bool) {
    println!("running in JIT mode");
    loop {
      let maybe_stub = self.stubs.get(&self.state.r3000.pc());
      match maybe_stub {
        Some(stub) => {
          println!("running block {:#x}", self.state.r3000.pc());
          for f in stub {
            self.state.r3000.flush_write_cache(&mut self.state.delayed_writes,
                                               &mut self.state.modified_register);
            f(&mut self.state);
            match self.state.gpu.exec_next_gp0_command() {
              Some(object) => self.state.screen.draw(object),
              None => (),
            };
            self.state.cd.exec_command();
          }
          *self.state.r3000.pc_mut() = self.state.next_pc;
        },
        None => {
          let mut operations = vec![];
          let start = self.state.r3000.pc();
          let op = self.state.resolve_memresponse(self.state.memory.read_word(start));
          let mut address = start;
          let mut compiled = self.compile_opcode(op);
          //add all instructions before the next jump to the stub
          while compiled.is_some() {
            operations.push(compiled.take().expect(""));
            address += 4;
            let op = self.state.resolve_memresponse(self.state.memory.read_word(address));
            compiled = self.compile_opcode(op);
          }
          //get the jump instruction that ended the block
          let compiled_jump = self.compile_jump(op);
          operations.push(compiled_jump);

          //add the branch delay slot to the stub
          address += 4;
          let op = self.state.resolve_memresponse(self.state.memory.read_word(address));
          compiled = self.compile_opcode(op);
          operations.push(compiled.take().expect(""));

          self.stubs.insert(start, operations);
          println!("compiled a block for {:#x}", start);
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

        next_pc: 0,
        delayed_writes,
        modified_register: None,
      },

      stubs: Default::default(),

      i: 0,
    })
  }
}
