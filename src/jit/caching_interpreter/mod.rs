use std::io;
use std::collections::HashMap;
use std::time::Instant;
use block::Block;
use crate::jit::insn::Insn;
use crate::console::Console;

mod block;
mod optimized_stubs;
mod stubs;

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
    let start_time = Instant::now();
    let mut refresh_timer: i64 = Console::REFRESH_RATE;
    loop {
      let address = Console::physical(self.console.r3000.pc());
      let maybe_block = self.blocks.get(&address);
      match maybe_block {
        Some(block) => {
    //println!("ran block from {:#x}", address);
    let init_pc = self.console.r3000.pc();
          let t0 = Instant::now();
          let stubs = block.stubs();
          //this is updated if we updated early
          let mut steps_taken = block.nominal_len();
          for (i, stub) in stubs.iter().enumerate() {
            self.console.r3000.flush_write_cache(&mut self.console.delayed_writes,
                                                 &mut self.console.modified_register);
            match stub.execute(&mut self.console, logging) {
              Some(next_pc) => {
                steps_taken = i as u32 + 1;
                if i + 1 != stubs.len() {
                  steps_taken = i as u32 + 2;
                  match self.console.gpu.exec_next_gp0_command() {
                    Some(object) => self.console.screen.draw(object),
                    None => (),
                  }
                  self.console.cd.exec_command();
                  self.console.r3000.flush_write_cache(&mut self.console.delayed_writes,
                                                       &mut self.console.modified_register);
                  stubs[i + 1].execute(&mut self.console, logging);
                  match self.console.gpu.exec_next_gp0_command() {
                    Some(object) => self.console.screen.draw(object),
                    None => (),
                  }
                  self.console.cd.exec_command();
                };
                *self.console.r3000.pc_mut() = next_pc;
                break;
              },
              None => (),
            }
            match self.console.gpu.exec_next_gp0_command() {
              Some(object) => self.console.screen.draw(object),
              None => (),
            }
            self.console.cd.exec_command();
          }
    let final_pc = self.console.r3000.pc();
    println!("ran block from {:#x} to {:#x}", init_pc, final_pc);
          refresh_timer -= steps_taken as i64;
          if refresh_timer < 0 {
            self.console.screen.refresh_window();
            refresh_timer = Console::REFRESH_RATE;
          }
          self.console.i += steps_taken;
          n.map(|n| {
            if self.console.i >= n {
              let end_time = Instant::now();
              panic!("Executed {} steps in {:?}", self.console.i, end_time - start_time);
            };
          });
          let end = block.final_pc();
          let block_invalidated = self.console
                                      .overwritten
                                      .iter()
                                      .any(|&x| {
                                        address <= x && x <= end
                                      });
          //if this block was invalidated by a write
          if block_invalidated {
            self.cache_invalidation(address);
          }
          self.console.overwritten.clear();
          if !self.console.handle_events() {
            return
          }
        },
        None => {
          self.translate(optimize, logging);
        },
      }
    }
  }
  fn translate(&mut self, optimize: bool, logging: bool) {
    //first define the opcodes in this block and tag them along the way
    let mut address = self.console.r3000.pc();
    let start = Console::physical(address);
    let mut op = self.console.read_word(address);
    //start with an offset of 4 since pc is incremented before the next instruction is executed
    //this makes sure that pc has the correct value when a jump is taken in a branch delay slot
    let mut counter = 4;
    let mut insn = Insn::new(op, counter);
    let mut tagged_opcodes = Vec::new();
    while !Insn::is_unconditional_jump(op) {
      tagged_opcodes.push(insn);
      address = address.wrapping_add(4);
      op = self.console.read_word(address);
      counter += 4;
      insn = Insn::new(op, counter);
    }
    //append the tagged unconditional jump or syscall that ended the block
    tagged_opcodes.push(insn);
    //if the block ended in an unconditional jump, tag and append the delay slot
    if Insn::has_branch_delay_slot(op) {
      address = address.wrapping_add(4);
      op = self.console.read_word(address);
      counter += 4;
      insn = Insn::new(op, counter);
      tagged_opcodes.push(insn);
    }
    //get the length before doing optimizations
    let nominal_len = tagged_opcodes.len() as u32;
    //get the address of the last instruction in the block
    let final_pc = Console::physical(address);
    //compile the tagged opcodes into a block
    let block =
      if optimize {
        Block::new_optimized(&tagged_opcodes, final_pc, nominal_len, logging)
      } else {
        Block::new(&tagged_opcodes, final_pc, nominal_len, logging)
    };
    self.blocks.insert(start, block);
    //store the address range of the new block to simplify cache invalidation
    match self.ranges_compiled.get_mut(&final_pc) {
      Some(v) => {
        v.push(start);
      },
      None => {
        self.ranges_compiled.insert(final_pc, vec![start]);
      },
    }
  }
  fn cache_invalidation(&mut self, address: u32) {
    //remove the previously executed block
    let deleted_block = self.blocks.remove(&address).unwrap();
    self.ranges_compiled
        .entry(deleted_block.final_pc())
        .and_modify(|v| v.retain(|&start| start != address));
    //get all blocks containing the deleted block as a subset
    let overlapping_blocks = self.ranges_compiled
                                 .get(&deleted_block.final_pc())
                                 .unwrap()
                                 .iter()
                                 .copied()
                                 .filter(|&start| start <= address)
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
          v.retain(|&start| address < start);
        });
  }
}
