use std::io;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use crate::jit::x64_jit::block::Block;
use crate::jit::insn::Insn;
use crate::console::Console;

mod block;
mod optimized_x64_macros;
mod x64_macros;
mod register_allocator;
mod macro_assembler;
mod assembler;

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
          block.function.execute();
          for i in 1..=31 {
            println!("{:#x}", self.console.r3000.nth_reg(i));
          }
          println!("{:#x}", self.console.r3000.pc());
          for i in 1..=31 {
            if i == 1 {
              assert_eq!(self.console.r3000.nth_reg(1), 0x1f80_0000);
            } else if i == 8 {
              assert_eq!(self.console.r3000.nth_reg(8), 0xb88);
            } else {
              assert_eq!(self.console.r3000.nth_reg(i), 0);
            }
          }
          //these assertions should pass the first block and fail on the second
          assert_eq!(self.console.r3000.pc(), 0xbfc0_0150);
          //assert_eq!(self.console.read_word(0x1f80_1010), 0x13243f);
          //assert_eq!(self.console.read_word(0x1f80_1060), 0xb88);
          //let stubs = block.stubs();
          //for stub in stubs {
          //  self.console.r3000.flush_write_cache(&mut self.console.delayed_writes,
          //                                       &mut self.console.modified_register);
          //  let temp_pc = stub.execute(&mut self.console, logging);
          //  //check result of previous opcode
          //  match self.console.next_pc {
          //    Some(next_pc) => {
          //      //block ended early so let's move pc since we just executed the
          //      //branch delay slot
          //      *self.console.r3000.pc_mut() = next_pc;
          //      break;
          //    },
          //    None => {
          //    },
          //  }
          //  self.console.next_pc = temp_pc;
          //  match self.console.gpu.exec_next_gp0_command() {
          //    Some(object) => self.console.screen.draw(object),
          //    None => (),
          //  }
          //  self.console.cd.exec_command();
          //}
          refresh_timer -= block.nominal_len() as i64;
          if refresh_timer < 0 {
            self.console.screen.refresh_window();
            refresh_timer = Console::REFRESH_RATE;
          }
          self.console.i += block.nominal_len();
          n.map(|n| {
            if self.console.i >= n {
              panic!("Executed {} steps with {:?} of compile time and {:?} of run time", self.console.i, compile_time, run_time);
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
    Ok(())
  }
  fn translate(&mut self, optimize: bool, logging: bool) -> io::Result<Duration> {
    let t0 = Instant::now();
    //first define the opcodes in this block and tag them along the way
    let mut address = self.console.r3000.pc();
    let initial_pc = address;
    let initial_phys_pc = Console::physical(initial_pc);
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
