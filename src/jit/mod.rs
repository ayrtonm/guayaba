use std::io;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;
use std::time::Duration;
use crate::register::Register;
use crate::register::BitBang;
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
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod opcodes;
mod jumps;
mod handle_dma;

const PHYS_MASK: [u32; 8] = [0xffff_ffff, 0xffff_ffff, 0xffff_ffff, 0xffff_ffff,
                             0x7fff_ffff, 0x1fff_ffff, 0xffff_ffff, 0xffff_ffff];
fn physical(address: Register) -> Register {
  let idx = address.upper_bits(3) as usize;
  address & PHYS_MASK[idx]
}

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
  next_pc: Register,
  delayed_writes: VecDeque<DelayedWrite>,
  modified_register: Option<Name>,
  overwritten: HashSet<Register>,
}

impl State {
  fn write_byte(&mut self, address: Register, value: Register) -> Option<Vec<MemAction>> {
    self.overwritten.insert(physical(address));
    self.memory.write_byte(address, value)
  }
  fn write_half(&mut self, address: Register, value: Register) -> Option<Vec<MemAction>> {
    self.overwritten.insert(physical(address));
    self.memory.write_half(address, value)
  }
  fn write_word(&mut self, address: Register, value: Register) -> Option<Vec<MemAction>> {
    self.overwritten.insert(physical(address));
    self.memory.write_word(address, value)
  }
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
  //maps start addresses to stubs for efficient execution
  stubs: HashMap<Register, Stub>,
  //maps end addresses to start addresses for efficient cache invalidation
  ranges_compiled: HashMap<Register, Vec<Register>>,
  i: u32,
}

impl Runnable for JIT {
  fn run(&mut self, n: Option<u32>, logging: bool) {
    let start_time = Instant::now();
    let mut down_time = start_time - start_time;
    let mut compile_time = start_time - start_time;
    println!("running in JIT mode");
    loop {
      let address = physical(self.state.r3000.pc());
      let t0 = Instant::now();
      if self.state.overwritten.len() >= 1000 {
        self.cache_invalidation();
      }
      let maybe_invalidated_stub = self.stubs.get(&address);
      match maybe_invalidated_stub {
        Some(stub) => {
          let mut intersection = self.state.overwritten.clone();
          //these are the executable addresses that have been overwritten
          //this will be no bigger than the size of the stub
          intersection.retain(|&t| address <= t && t <= physical(stub.final_pc()));

          if !intersection.is_empty() {
            self.cache_invalidation();
          };
        },
        None => {
        },
      }
      let t1 = Instant::now();
      down_time += t1 - t0;
      let maybe_stub = self.stubs.get(&address);
      match maybe_stub {
        Some(stub) => {
          //println!("running block {:#x}", self.state.r3000.pc());
          let operations = stub.operations();
          *self.state.r3000.pc_mut() = stub.final_pc();
          for f in operations {
            self.state.r3000.flush_write_cache(&mut self.state.delayed_writes,
                                               &mut self.state.modified_register);
            f(&mut self.state);
            match self.state.gpu.exec_next_gp0_command() {
              Some(object) => self.state.screen.draw(object),
              None => (),
            };
            self.state.cd.exec_command();
            self.i += 1;
            n.map(|n| {
              if self.i == n {
                let end_time = Instant::now();
                panic!("Executed {} steps in {:?}\nwith {:?} of down time and {:?} of compile time",
                       self.i, end_time - start_time, down_time, compile_time);
              };
            });
            //println!("on step {} of block", self.i);
          }
          *self.state.r3000.pc_mut() = self.state.next_pc;
          if !self.handle_events() {
            return
          }
        },
        None => {
          //if the stub was invalidated, compile another one
          compile_time += self.compile_stub(logging);
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
        overwritten: Default::default(),
      },

      stubs: Default::default(),
      ranges_compiled: Default::default(),

      i: 0,
    })
  }
  fn compile_stub(&mut self, logging: bool) -> Duration {
    let t0 = Instant::now();
    let mut operations = Vec::with_capacity(50);
    let start = self.state.r3000.pc();
    let mut op = self.state.resolve_memresponse(self.state.memory.read_word(start));
    let mut address = start;
    let mut compiled = self.compile_opcode(op, logging);
    //add all instructions before the next jump to the stub
    while compiled.is_some() {
      operations.push(compiled.take().expect(""));
      address = address.wrapping_add(4);
      op = self.state.resolve_memresponse(self.state.memory.read_word(address));
      //println!("{:#x}", op);
      compiled = self.compile_opcode(op, logging);
    }
    //println!("jump {:#x} at {:#x}", op, address);
    //get the jump instruction that ended the block
    let jump_op = op;
    let compiled_jump = self.compile_jump(op, logging);
    operations.push(compiled_jump);

    //if the jump was not a SYSCALL
    if jump_op != 0xc {
      //add the branch delay slot to the stub
      address = address.wrapping_add(4);
      let op = self.state.resolve_memresponse(self.state.memory.read_word(address));
      //println!("branch delay slot contained {:#x}", op);
      compiled = self.compile_opcode(op, logging);
      //println!("{:#x} followed by {:#x}", jump_op, op);
      operations.push(compiled.expect("Consecutive jumps are not allowed in the MIPS ISA"));
    }

    //println!("compiled a block with {} operations for {:#x}", operations.len(), start);
    //let's try limiting the size of the cache
    //if self.stubs.len() >= 128 {
    //  self.stubs.clear();
    //  self.ranges_compiled.clear();
    //};
    self.stubs.insert(physical(start), Stub { operations, final_pc: address });
    let end = physical(address);
    self.ranges_compiled.get_mut(&end)
                        .map(|v| {
                          v.push(physical(start));
                        })
                        .or_else(|| {
                          self.ranges_compiled.insert(end, vec![physical(start)]);
                          None
                        });
    //self.ranges_compiled.push((physical(start), physical(address));
    let t1 = Instant::now();
    t1 - t0
  }
  fn cache_invalidation(&mut self) {
    let mut invalidated = Vec::new();
    self.state.overwritten.iter()
    .for_each(|&addr| {
      for &e in self.ranges_compiled.keys().filter(|&e| addr <= *e) {
        self.ranges_compiled.get(&e)
                            .unwrap()
                            .iter()
                            .filter(|&s| *s <= addr)
                            .for_each(|&s| {
                              println!("removed a stub");
                              invalidated.push(s);
                            });
      }
    });
    self.state.overwritten.clear();
    for i in invalidated {
      let value = self.stubs.remove(&i).unwrap();
      self.ranges_compiled.remove(&value.final_pc());
    }
  }
  fn handle_events(&mut self) -> bool {
    let event_rate: u32 = 100_000;
    if self.i % event_rate == 0 {
      for event in self.state.screen.event_pump().poll_iter() {
        match event {
          Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
            println!("Executed {} steps", self.i);
            return false;
          },
          Event::Quit {..} => panic!(""),
          _ => {},
        }
      }
    }
    true
  }
}
