use std::io;
use jam::jit_fn::JITFn;
use crate::jit::insn::Insn;
use crate::jit::insn::InsnRegisterFrequency;
use crate::console::Console;
use crate::r3000::R3000;

pub struct Block {
  pub function: JITFn,
  initial_pc: u32,
  final_phys_pc: u32,
  nominal_len: u32,
}

impl Block {
  pub fn new(tagged_opcodes: &Vec<Insn>, console: &Console,
             initial_pc: u32, final_phys_pc: u32,
             nominal_len: u32, logging: bool) -> io::Result<Self> {
    let function = Block::create_function(tagged_opcodes, &console,
                                          initial_pc, logging)?;
    Ok(Block {
      function,
      initial_pc,
      final_phys_pc,
      nominal_len,
    })
  }
  pub fn new_optimized(tagged_opcodes: &Vec<Insn>, console: &Console,
                       initial_pc: u32, final_phys_pc: u32,
                       nominal_len: u32, logging: bool) -> io::Result<Self> {
    Block::new(tagged_opcodes, console, initial_pc, final_phys_pc, nominal_len, logging)
  }
  pub fn final_phys_pc(&self) -> u32 {
    self.final_phys_pc
  }
  pub fn nominal_len(&self) -> u32 {
    self.nominal_len
  }
  fn create_function(tagged_opcodes: &Vec<Insn>, console: &Console,
                     initial_pc: u32, logging: bool) -> io::Result<JITFn> {
    todo!("")
  }
}
