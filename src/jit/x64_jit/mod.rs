use std::io;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use crate::jit::x64_jit::block::Block;
use crate::jit::insn::Insn;
use crate::console::Console;

mod block;
mod dynarec;

pub struct X64JIT {
  console: Console,
  blocks: HashMap<u32, Block>,
  ranges_compiled: HashMap<u32, Vec<u32>>,
}

impl X64JIT {
  pub fn new(bios_filename: &String, infile: Option<&String>, gpu_logging: bool,
             wx: u32, wy: u32) -> io::Result<Self> {
    let console = Console::new(bios_filename, infile, gpu_logging, wx, wy)?;
    Ok(Self {
      console,
      blocks: Default::default(),
      ranges_compiled: Default::default(),
    })
  }
  pub fn run(&mut self, n: Option<u32>, optimize: bool, logging: bool) -> io::Result<()> {
    println!("running in x64 JIT mode");
    let t0 = Instant::now();
    let mut compile_time = t0 - t0;
    let mut run_time = t0 - t0;
    let mut refresh_timer: i64 = Console::REFRESH_RATE;
    loop {
      let address = Console::physical(self.console.r3000.pc());
      let maybe_block = self.blocks.get(&address);
      match maybe_block {
        Some(block) => {
          let t0 = Instant::now();
          let init_pc = self.console.r3000.pc();
          block.run();
          let final_pc = self.console.r3000.pc();
          //println!("ran block from {:#x} to {:#x}", init_pc, final_pc);
          match self.console.gpu.exec_next_gp0_command() {
            Some(object) => self.console.screen.draw(object),
            None => (),
          }
          self.console.cd.exec_command();
          refresh_timer -= block.nominal_len() as i64;
          if refresh_timer < 0 {
            self.console.screen.refresh_window();
            refresh_timer = Console::REFRESH_RATE;
          }
          self.console.i += block.nominal_len();
          n.map(|n| {
            if self.console.i >= n {
              panic!("Executed {} steps with {:?} of compile time and {:?} of run time",
                     self.console.i, compile_time, run_time);
            };
          });
          let block_invalidated = self.console
                                      .overwritten
                                      .iter()
                                      .any(|&x| {
                                        address <= x && x <= block.final_phys_pc()
                                      });
          //if this block was invalidated by a write
          if block_invalidated {
            self.cache_invalidation(address);
          }
          self.console.overwritten.clear();
          if !self.console.handle_events() {
            return Ok(());
          }
          let t1 = Instant::now();
          run_time += t1 - t0;
        },
        None => {
          compile_time += self.translate(optimize, logging)?;
        },
      }
    }
  }
  fn translate(&mut self, optimize: bool, logging: bool) -> io::Result<Duration> {
    let t0 = Instant::now();
    //first define the opcodes in this block and tag them along the way
    let mut address = self.console.r3000.pc();
    let initial_pc = address;
    let initial_phys_pc = Console::physical(initial_pc);
    let mut op = self.console.read_word(address);
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
    let final_phys_pc = Console::physical(address);
    //compile the tagged opcodes into a block
    let block =
      if optimize {
        Block::new_optimized(&tagged_opcodes, &self.console, initial_pc, final_phys_pc, nominal_len, logging)
      } else {
        Block::new(&tagged_opcodes, &self.console, initial_pc, final_phys_pc, nominal_len, logging)
    }?;
    self.blocks.insert(initial_phys_pc, block);
    //store the address range of the new block to simplify cache invalidation
    match self.ranges_compiled.get_mut(&final_phys_pc) {
      Some(v) => {
        v.push(initial_phys_pc);
      },
      None => {
        self.ranges_compiled.insert(final_phys_pc, vec![initial_phys_pc]);
      },
    }
    let t1 = Instant::now();
    Ok(t1 - t0)
  }
  fn cache_invalidation(&mut self, address: u32) {
    //remove the previously executed block
    let deleted_block = self.blocks.remove(&address).unwrap();
    //get all blocks containing the deleted block as a subset
    let overlapping_blocks = self.ranges_compiled
                                 .get(&deleted_block.final_phys_pc())
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
        .entry(deleted_block.final_phys_pc())
        .and_modify(|v| {
          v.retain(|&start| address < start);
        });
  }
}
