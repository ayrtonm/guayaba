use std::io;
use jam::jit_fn::JITFn;
use jam::recompiler::Recompiler;
use crate::jit::insn::Insn;
use crate::jit::insn::InsnRegisters;
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
    let inputs = tagged_opcodes.registers();
    let mut ptrs = vec![0; 11];
    ptrs[Block::R3000_REG_POS] = console.r3000.reg_ptr() as u64;
    ptrs[Block::COP0_REG_POS] = console.cop0.reg_ptr() as u64;
    ptrs[Block::CONSOLE_POS] = console as *const Console as u64;
    ptrs[Block::WRITE_WORD_POS] = Console::write_word as u64;
    ptrs[Block::WRITE_HALF_POS] = Console::write_half as u64;
    ptrs[Block::WRITE_BYTE_POS] = Console::write_byte as u64;
    ptrs[Block::READ_WORD_POS] = Console::read_word as u64;
    ptrs[Block::READ_HALF_POS] = Console::read_half as u64;
    ptrs[Block::READ_BYTE_POS] = Console::read_byte as u64;
    ptrs[Block::READ_HALF_SIGN_EXTENDED_POS] = Console::read_half_sign_extended as u64;
    ptrs[Block::READ_BYTE_SIGN_EXTENDED_POS] = Console::read_byte_sign_extended as u64;
    let mut rc = Recompiler::new(&inputs, &ptrs);
    for insn in &tagged_opcodes[0..3] {
      Block::emit_insn(&mut rc, insn);
    }
    let jitfn = rc.compile().unwrap();
    println!("compiled {} bytes", jitfn.size());
    Ok(jitfn)
  }
}
