use std::io;
use jam::jit_fn::JITFn;
use jam::recompiler::Recompiler;
use crate::jit::insn::Insn;
use crate::jit::insn::InsnRegisters;
use crate::console::Console;
use crate::console::r3000::R3000;
use crate::jit::x64_jit::dynarec::DynaRec;
use crate::common::*;

pub struct Block {
  pub function: JITFn,
  final_phys_pc: u32,
  nominal_len: u32,
}

impl Block {
  pub const R3000_REG_POS: usize = 0;
  pub const COP0_REG_POS: usize = 1;
  pub const CONSOLE_POS: usize = 2;
  pub const WRITE_WORD_POS: usize = 3;
  pub const WRITE_HALF_POS: usize = 4;
  pub const WRITE_BYTE_POS: usize = 5;
  pub const READ_WORD_POS: usize = 6;
  pub const READ_HALF_POS: usize = 7;
  pub const READ_BYTE_POS: usize = 8;
  pub const READ_HALF_SIGN_EXTENDED_POS: usize = 9;
  pub const READ_BYTE_SIGN_EXTENDED_POS: usize = 10;
  pub const DEBUG_POS: usize = 11;
  pub fn new(tagged_opcodes: &Vec<Insn>, console: &Console,
             initial_pc: u32, final_phys_pc: u32,
             nominal_len: u32, logging: bool) -> io::Result<Self> {
    let function = Block::create_function(tagged_opcodes, &console,
                                          initial_pc, logging)?;
    Ok(Block {
      function,
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
    let mut inputs = tagged_opcodes.registers();
    inputs.push(R3000::PC_IDX as u32);
    let mut ptrs = vec![0; 12];
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
    ptrs[Block::DEBUG_POS] = Console::print_value as u64;
    let mut rc = Recompiler::new(&inputs, &ptrs);
    let mut this_label = None;
    let end = rc.new_long_label();
    for (n, insn) in tagged_opcodes.iter().enumerate() {
      match this_label.take() {
        Some(jump) => {
          rc.save_flags();
          this_label = rc.emit_insn(insn, initial_pc);
          rc.load_flags();
          rc.prepare_for_exit();
          rc.debug_bind(rc.reg(R3000::PC_IDX as u32).unwrap());
          //FIXME: remove this unconditional set carry and rely on the save/load_flags above
          //when calling load_flags, make sure that the stack is where it was when we called save_flags
          rc.set_carry();
          rc.jump_if_carry(end);
        },
        None => {
          this_label = rc.emit_insn(insn, initial_pc);
        },
      }
      if initial_pc.wrapping_add(4 * n as u32) == 0xbfc0_02a0 {
        break
      }
    }
    rc.prepare_for_exit();
    let jit_pc = rc.reg(R3000::PC_IDX as u32).unwrap();
    rc.seti_u32(jit_pc, initial_pc.wrapping_add(4 * tagged_opcodes.len() as u32));
    rc.define_label(end);
    let jitfn = rc.compile().unwrap();
    println!("compiled {} bytes", jitfn.size());
    Ok(jitfn)
  }
}
