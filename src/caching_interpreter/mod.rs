use std::io;
use std::collections::HashMap;
use crate::console::Console;

type Stub = Box<dyn Fn(&mut Console) -> Option<u32>>;

struct Block {
  //a vec of closures to be executed in order
  stubs: Vec<Stub>,
  //the physical address of the last instruction
  //this will be either the branch delay slot or a syscall
  final_pc: u32,
  //the number of MIPS opcodes represented by this Block
  //may be more than the length of stubs
  nominal_len: u32,
}

impl Block {
  fn stubs(&self) -> &Vec<Stub> {
    &self.stubs
  }
  fn final_pc(&self) -> u32 {
    self.final_pc
  }
  fn nominal_len(&self) -> u32 {
    self.nominal_len
  }
}

pub struct CachingInterpreter {
  console: Console,
  blocks: HashMap<u32, Block>,
  ranges_compiled: HashMap<u32, Vec<u32>>,
}

impl CachingInterpreter {
  pub fn new(bios_filename: &String, infile: Option<&String>, gpu_logging: bool,
             wx: u32, wy: u32) -> io::Result<Self> {
    let console = Console::new(bios_filename, infile, gpu_logging, wx, wy)?;
    Ok(Self {
      console,
      blocks: Default::default(),
      ranges_compiled: Default::default(),
    })
  }
  pub fn run(&mut self, n: Option<u32>, optimize: bool, logging: bool) {
    println!("running in caching interpreter mode");
    let mut refresh_timer: i64 = Console::REFRESH_RATE;
    loop {
      let address = Console::physical(self.console.r3000.pc());
      let maybe_block = self.blocks.get(&address);
      match maybe_block {
        Some(block) => {
          let stubs = block.stubs();
          for stub in stubs {
            let temp_pc = stub(&mut self.console);
            //check result of previous opcode
            match self.console.next_pc {
              Some(next_pc) => {
                //block ended early so let's move pc since we just executed the
                //branch delay slot
                *self.console.r3000.pc_mut() = next_pc;
                break;
              },
              None => {
              },
            }
            self.console.next_pc = temp_pc;
            match self.console.gpu.exec_next_gp0_command() {
              Some(object) => self.console.screen.draw(object),
              None => (),
            }
            self.console.cd.exec_command();
          }
          refresh_timer -= block.nominal_len() as i64;
          if refresh_timer < 0 {
            self.console.screen.refresh_window();
            refresh_timer = Console::REFRESH_RATE;
          }
          self.console.i += block.nominal_len();
          n.map(|n| {
            if self.console.i >= n {
              panic!("Executed {} steps", self.console.i);
            };
          });
          match self.console.next_pc.take() {
            //if we ended on a syscall
            Some(next_pc) => *self.console.r3000.pc_mut() = next_pc,
            None => (),
          }
          let block_invalidated = self.console
                                      .overwritten
                                      .iter()
                                      .any(|&x| {
                                        address <= x && x <= block.final_pc()
                                      });
          //if this block was invalidated by a write
          if block_invalidated {
            self.cache_invalidation(address);
          }
          self.console.overwritten.clear();
        },
        None => {
          self.create_block(optimize, logging);
        },
      }
    }
  }
  fn create_block(&mut self, optimize: bool, logging: bool) {
  }
  fn cache_invalidation(&mut self, address: u32) {
    //remove the previously executed block
    let deleted_block = self.blocks.remove(&address).unwrap();
    //get the blocks that are subsets of the deleted block
    let overlapping_blocks = self.ranges_compiled
                                 .get(&deleted_block.final_pc())
                                 .unwrap()
                                 .iter()
                                 .copied()
                                 .filter(|&start| address <= start)
                                 .collect::<Vec<u32>>();
    //remove the overlapping blocks
    overlapping_blocks.iter()
                      .for_each(|s| {
                        self.blocks.remove(&s).unwrap();
                      });
    //clean up the auxilary map of ranges compiled
    self.ranges_compiled
        .entry(deleted_block.final_pc())
        .and_modify(|v| {
          v.retain(|&start| start < address);
        });
  }
}
