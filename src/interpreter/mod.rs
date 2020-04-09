use std::io;
use std::collections::VecDeque;
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

mod opcodes;
mod handle_dma;

pub struct Interpreter {
  //these correspond to physical components
  r3000: R3000,
  cop0: Cop0,
  memory: Memory,
  gpu: GPU,
  gte: GTE,
  cd: Option<CD>,

  //regular members of interpreter
  next_pc: Option<Register>,
  //these are register writes due to memory loads which happen after one cycle
  delayed_writes: VecDeque<DelayedWrite>,
  modified_register: Option<Name>,
  i: u32,
}

impl Interpreter {
  pub fn new(bios_filename: &String, infile: Option<&String>) -> io::Result<Self> {
    let r3000 = R3000::new();
    let cop0 = Default::default();
    let memory = Memory::new(bios_filename)?;
    let gpu = GPU::new();
    let gte = Default::default();
    let cd = infile.and_then(|f| CD::new(f).ok());
    let delayed_writes = VecDeque::new();
    Ok(Interpreter {
      r3000,
      cop0,
      memory,
      gpu,
      gte,
      cd,
      next_pc: None,
      delayed_writes,
      modified_register: None,
      i: 0,
    })
  }
  pub fn run(&mut self, n: Option<u32>, logging: bool, event_pump: &mut sdl2::EventPump) {
    let event_rate: u32 = 1_00_000;
    n.map(
      |n| {
        println!("started in test mode");
        for i in 1..=n {
          if logging {
            println!("  ");
            println!("{} ----------------------", i);
          }
          self.step(logging);
          if i % event_rate == 0 {
            for event in event_pump.poll_iter() {
              match event {
                sdl2::event::Event::Quit {..} => panic!(""),
                _ => {},
              }
            }
          }
        }
    }).or_else(
      || {
        println!("started in free-running mode");
        loop {
          if logging {
            println!("  ");
            println!("{} ----------------------", self.i);
          }
          self.step(logging);
          self.i += 1;
          if self.i % event_rate == 0 {
            for event in event_pump.poll_iter() {
              match event {
                sdl2::event::Event::Quit {..} => panic!(""),
                _ => {},
              }
            }
          }
        }
      });
  }
  fn resolve_memresponse(&mut self, response: MemResponse) -> Register {
    match response {
      MemResponse::Value(value) => value,
      MemResponse::GPUREAD => 0,
      MemResponse::GPUSTAT => 0x1c00_0000,
    }
  }
  fn resolve_memaction(&mut self, maybe_action: Option<MemAction>) {
    maybe_action.map(
      |action| {
        match action {
          MemAction::DMA(transfers) => self.handle_dma(transfers),
          MemAction::GpuGp0(value) => self.gpu.write_to_gp0(value),
          MemAction::GpuGp1(value) => self.gpu.write_to_gp1(value),
          MemAction::Debug => {println!("{:#x?} {}", self.r3000, self.i); panic!("")},
        }
      }
    );
  }
  fn step(&mut self, logging: bool) {
    //get opcode from memory at program counter
    let op = self.resolve_memresponse(self.memory.read_word(self.r3000.pc()));
    if logging {
      println!("read opcode {:#x} from [{:#x}]", op, self.r3000.pc());
    }
    //the instruction following each jump is always executed before updating the pc
    //increment the program counter
    *self.r3000.pc_mut() = self.next_pc
                           .take()
                           .map_or_else(|| self.r3000.pc().wrapping_add(4),
                                        |next_pc| next_pc);
    self.next_pc = self.execute_opcode(op, logging);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn dummy_bios() {
    //this is the entry point in case we want to test some dummy instructions
    const BIOS: Register = 0x1fc0_0000;
    let mut vm = Interpreter::new(&"/home/ayrton/dev/rspsx/scph1001.bin".to_string(),
                                  None).unwrap();
    vm.memory.write_word(BIOS, 0x0000_0002);
    let dest = 0x0bf0_0000;
    let instr = (2 << 26) | (dest & 0x03ff_ffff);
    vm.memory.write_word(BIOS + 4, 0x0000_0003);
    vm.memory.write_word(BIOS + 8, 0x0000_0004);
    vm.memory.write_word(BIOS + 12, instr);
    vm.memory.write_word(BIOS + 16, 0x0000_0006);
    vm.run(Some(10),false);
  }
}
