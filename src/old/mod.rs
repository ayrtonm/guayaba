use std::io;
use std::collections::HashMap;
use std::time::Instant;
use std::time::Duration;
use crate::console::Console;

mod insn_ir;
mod opcodes;
mod jumps;
mod optimize;

struct Stub {
  operations: Vec<Box<dyn Fn(&mut Console)>>,
  final_pc: u32,
  len: u32,
}

impl Stub {
  fn operations(&self) -> &Vec<Box<dyn Fn(&mut Console)>> {
    &self.operations
  }
  fn final_pc(&self) -> u32 {
    self.final_pc
  }
  fn len(&self) -> u32{
    self.len
  }
}

pub struct CachingInterpreter {
  console: Console,
  //maps start addresses to stubs for efficient execution
  stubs: HashMap<u32, Stub>,
  //maps end addresses to start addresses for efficient cache invalidation
  ranges_compiled: HashMap<u32, Vec<u32>>,
}

impl CachingInterpreter {
  pub fn run(&mut self, n: Option<u32>, optimize: bool, logging: bool) {
    let start_time = Instant::now();
    let mut compile_time = start_time - start_time;
    let mut cache_time = start_time - start_time;
    let mut refresh_timer: i64 = Console::REFRESH_RATE;
    println!("running in dummy JIT mode");
    loop {
      let address = Console::physical(self.console.r3000.pc());
      let maybe_stub = self.stubs.get(&address);
      match maybe_stub {
        Some(stub) => {
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
          }
          refresh_timer -= stub.len() as i64;
          if refresh_timer < 0 {
            self.console.screen.refresh_window();
            refresh_timer = Console::REFRESH_RATE;
          }
          self.console.i += stub.len();
          n.map(|n| {
            if self.console.i >= n {
              let end_time = Instant::now();
              panic!("Executed {} steps in {:?}\nwith {:?} of compile time and {:?} of cache time",
                     self.console.i, end_time - start_time, compile_time, cache_time);
            };
          });
          if self.console.overwritten.iter().any(|&t| address <= t && t <= stub.final_pc()) {
            self.cache_invalidation(address);
          }
          self.console.overwritten.clear();
          *self.console
               .r3000
               .pc_mut() = self.console
                               .next_pc
                               .take()
                               .map_or_else(|| self.console.r3000.pc().wrapping_add(4),
                                            |next_pc| next_pc);
          if !self.console.handle_events() {
            return
          }
        },
        None => {
          //if the stub was invalidated, compile another one
          compile_time += self.parse_stub(optimize, logging);
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
  fn parse_stub(&mut self, optimize: bool, logging: bool) -> Duration {
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
    let mut compiled_stub = if optimize {
      self.compile_optimized_stub(&mut operations, logging)
    } else {
      self.compile_stub(&mut operations, logging)
    };

    //println!("jump {:#x} at {:#x}", op, address);
    //get the jump instruction that ended the block
    let jump_op = op;
    let compiled_jump = self.compile_jump(op, logging);
    compiled_stub.push(compiled_jump);

    let mut len = operations.len() as u32 + 1;
    //if the jump was not a SYSCALL
    if jump_op != 0xc {
      //add the branch delay slot to the stub
      address = address.wrapping_add(4);
      let op = self.console.resolve_memresponse(self.console.memory.read_word(address));
      //println!("branch delay slot contained {:#x}", op);
      let compiled = self.compile_opcode(op, logging);
      //println!("{:#x} followed by {:#x}", jump_op, op);
      compiled_stub.push(compiled.expect("Consecutive jumps are not allowed in the MIPS ISA"));
      len += 1;
    }
    let stub = Stub {
      operations: compiled_stub,
      final_pc: Console::physical(address),
      len,
    };
    self.stubs.insert(Console::physical(start), stub);
    let end = Console::physical(address);
    self.ranges_compiled.get_mut(&end)
                        .map(|v| {
                          v.push(Console::physical(start));
                        })
                        .or_else(|| {
                          self.ranges_compiled.insert(end, vec![Console::physical(start)]);
                          None
                        });
    let t1 = Instant::now();
    t1 - t0
  }
  fn cache_invalidation(&mut self, address: u32) {
    let deleted_stub = self.stubs.remove(&address).unwrap();
    let overlapping_blocks = self.ranges_compiled.get(&deleted_stub.final_pc())
                                                 .unwrap()
                                                 .iter()
                                                 .copied()
                                                 .filter(|&start| address <= start)
                                                 .collect::<Vec<u32>>();
    overlapping_blocks.iter()
                      .for_each(|s| {
                        self.stubs.remove(&s).unwrap();
                      });
    self.ranges_compiled
        .entry(deleted_stub.final_pc())
        .and_modify(|v| {
          v.retain(|&start| start < address);
        });
  }
}
