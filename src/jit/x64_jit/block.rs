use std::io;
use jam::jit_fn::JITFn;
use jam::recompiler::Recompiler;
use crate::jit::insn::Insn;
use crate::jit::insn::InsnRegisters;
use crate::console::Console;
use crate::console::r3000::R3000;
use crate::jit::x64_jit::dynarec::DynaRec;

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
    if !inputs.iter().any(|&i| i == R3000::RA_IDX) {
      inputs.push(R3000::RA_IDX);
    }
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
    let mut delay_slot_next = false;
    let end = rc.new_long_label();
    for insn in tagged_opcodes {
      let delay_slot = delay_slot_next;
      delay_slot_next = rc.emit_insn(insn, initial_pc);
      rc.process_delayed_write();
      if delay_slot_next {
        rc.save_flags();
      };
      if delay_slot {
        rc.prepare_for_exit();
        rc.load_flags();
        rc.jump_if_carry(end);
      };
    }
    let jit_pc = rc.reg(R3000::PC_IDX as u32).unwrap();
    rc.seti_u32(jit_pc, initial_pc.wrapping_add(4 * tagged_opcodes.len() as u32));
    rc.prepare_for_exit();
    rc.define_label(end);
    let jitfn = rc.compile().unwrap();
    println!("recompiled {} instructions into {} bytes", tagged_opcodes.len(), jitfn.size());
    Ok(jitfn)
  }
}
