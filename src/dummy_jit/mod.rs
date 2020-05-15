use std::ops::Add;
use std::ops::Shl;
use std::ops::Shr;
use std::ops::Sub;
use std::io;
use std::collections::HashMap;
use std::time::Instant;
use std::time::Duration;
use crate::console::Console;
use crate::register::Register;

mod insn_ir;
mod opcodes;
mod jumps;
mod optimize;

struct Stub {
  operations: Vec<Box<dyn Fn(&mut Console)>>,
  final_pc: Register,
}

impl Stub {
  fn operations(&self) -> &Vec<Box<dyn Fn(&mut Console)>> {
    &self.operations
  }
  fn final_pc(&self) -> Register {
    self.final_pc
  }
}

pub struct Dummy_JIT {
  console: Console,
  //maps start addresses to stubs for efficient execution
  stubs: HashMap<Register, Stub>,
  //maps end addresses to start addresses for efficient cache invalidation
  ranges_compiled: HashMap<Register, Vec<Register>>,
}

impl Dummy_JIT {
  pub fn run(&mut self, n: Option<u32>, logging: bool) {
    let start_time = Instant::now();
    let mut down_time = start_time - start_time;
    let mut compile_time = start_time - start_time;
    println!("running in dummy JIT mode");
    loop {
      let address = Console::physical(self.console.r3000.pc());
      //let t0 = Instant::now();
      //if self.console.overwritten.len() >= 1000 {
      //  self.cache_invalidation();
      //}
      //let maybe_invalidated_stub = self.stubs.get(&address);
      //match maybe_invalidated_stub {
      //  Some(stub) => {
      //    let mut intersection = self.console.overwritten.clone();
      //    //these are the executable addresses that have been overwritten
      //    //this will be no bigger than the size of the stub
      //    intersection.retain(|&t| address <= t && t <= Console::physical(stub.final_pc()));

      //    if !intersection.is_empty() {
      //      self.cache_invalidation();
      //    };
      //  },
      //  None => {
      //  },
      //}
      //let t1 = Instant::now();
      //down_time += t1 - t0;
      let maybe_stub = self.stubs.get(&address);
      match maybe_stub {
        Some(stub) => {
          //println!("running block {:#x}", self.console.r3000.pc());
          let operations = stub.operations();
          *self.console.r3000.pc_mut() = stub.final_pc();
          for f in operations {
            self.console.r3000.flush_write_cache(&mut self.console.delayed_writes,
                                               &mut self.console.modified_register);
            f(&mut self.console);
            match self.console.gpu.exec_next_gp0_command() {
              Some(object) => self.console.screen.draw(object),
              None => (),
            };
            self.console.cd.exec_command();
            self.console.i += 1;
            n.map(|n| {
              if self.console.i == n {
                let end_time = Instant::now();
                panic!("Executed {} steps in {:?}\nwith {:?} of down time and {:?} of compile time",
                       self.console.i, end_time - start_time, down_time, compile_time);
              };
            });
            //println!("on step {} of block", self.i);
          }
          *self.console.r3000.pc_mut() = self.console.next_pc
                                                     .map_or_else(|| self.console.r3000.pc().wrapping_add(4),
                                                                  |next_pc| next_pc);
          if !self.console.handle_events() {
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
  pub fn new(bios_filename: &String, infile: Option<&String>, gpu_logging: bool,
             wx: u32, wy: u32) -> io::Result<Self> {
    let console = Console::new(bios_filename, infile, gpu_logging, wx, wy)?;
    Ok(Self {
        console,
        stubs: Default::default(),
        ranges_compiled: Default::default(),
    })
  }
  fn compile_stub(&mut self, logging: bool) -> Duration {
    let t0 = Instant::now();
    let mut operations = Vec::new();
    let start = self.console.r3000.pc();
    let mut op = self.console.resolve_memresponse(self.console.memory.read_word(start));
    let mut address = start;
    let mut tagged = self.tag_insn(op, logging);
    //add all instructions before the next jump to the stub
    while tagged.is_some() {
      if op != 0x00 {
        operations.push((op, tagged.take().expect("")));
      }
      address = address.wrapping_add(4);
      op = self.console.resolve_memresponse(self.console.memory.read_word(address));
      //println!("{:#x}", op);
      tagged = self.tag_insn(op, logging);
    }
    //do stub analysis and optimizations here
    let mut compiled_stub = self.compile_optimized_stub(&mut operations, logging);

    ////compile tagged stub
    //let mut compiled_stub = Vec::new();
    //compiled_stub = operations.iter().map(|(op, tag)| self.compile_opcode(*op, logging).expect("")).collect();
    //println!("jump {:#x} at {:#x}", op, address);
    //get the jump instruction that ended the block
    let jump_op = op;
    let compiled_jump = self.compile_jump(op, logging);
    compiled_stub.push(compiled_jump);

    //if the jump was not a SYSCALL
    if jump_op != 0xc {
      //add the branch delay slot to the stub
      address = address.wrapping_add(4);
      let op = self.console.resolve_memresponse(self.console.memory.read_word(address));
      //println!("branch delay slot contained {:#x}", op);
      let compiled = self.compile_opcode(op, logging);
      //println!("{:#x} followed by {:#x}", jump_op, op);
      compiled_stub.push(compiled.expect("Consecutive jumps are not allowed in the MIPS ISA"));
    }

    //println!("compiled a block with {} operations for {:#x}", operations.len(), start);
    //let's try limiting the size of the cache
    //if self.stubs.len() >= 128 {
    //  self.stubs.clear();
    //  self.ranges_compiled.clear();
    //};
    self.stubs.insert(Console::physical(start), Stub { operations: compiled_stub, final_pc: address });
    //let end = Console::physical(address);
    //self.ranges_compiled.get_mut(&end)
    //                    .map(|v| {
    //                      v.push(Console::physical(start));
    //                    })
    //                    .or_else(|| {
    //                      self.ranges_compiled.insert(end, vec![Console::physical(start)]);
    //                      None
    //                    });
    //self.ranges_compiled.push((Console::physical(start), Console::physical(address));
    let t1 = Instant::now();
    t1 - t0
  }
  fn cache_invalidation(&mut self) {
    let mut invalidated = Vec::new();
    self.console.overwritten.iter()
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
    self.console.overwritten.clear();
    for i in invalidated {
      self.stubs.remove(&i);
      //let value = self.stubs.remove(&i).unwrap();
      //self.ranges_compiled.remove(&value.final_pc());
    }
  }
}
