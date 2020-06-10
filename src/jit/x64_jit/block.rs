use std::io;
use crate::jit::insn::Insn;
use crate::jit::insn::InsnRegisterFrequency;
use crate::jit::jit_fn::JIT_Fn;
use crate::jit::x64_jit::macro_compiler::MacroCompiler;
use crate::console::Console;
use crate::r3000::R3000;

pub struct Block {
  pub function: JIT_Fn,
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
                     initial_pc: u32, logging: bool) -> io::Result<JIT_Fn> {
    let mut compiler = MacroCompiler::new(tagged_opcodes);
    let jit_fn = compiler.compile()?;
    Ok(jit_fn)
  }
}
