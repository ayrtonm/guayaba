use crate::register::BitTwiddle;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::register_allocator::X64RegNum;

impl MacroAssembler {
  //FIXME: doesn't seem like this'll work with extended registers i.e. r8-r15
  fn specify_rr(src: u32, dest: u32) -> u8 {
    MacroAssembler::MOD11 | ((src as u8) << 3) | (dest as u8)
  }
  fn specify_rm(src: u32, ptr_dest: u32) -> u8 {
    (src.lowest_bits(3) << 3) as u8 | (ptr_dest.lowest_bits(3) as u8)
  }
  fn specify_mr(src: u32, ptr_dest: u32) -> u8 {
    MacroAssembler::specify_rm(src, ptr_dest)
  }
  fn specify_sib(base: u32, index: u32, scale: u32) -> u8 {
    let scale = match scale {
      1 => 0,
      2 => 1,
      4 => 2,
      8 => 3,
      _ => unreachable!("invalid SIB scale"),
    } << 6;
    base.lowest_bits(3) as u8 |  ((index.lowest_bits(3) as u8) << 3) | scale
  }
  pub fn emit_movl_rr(&mut self, src: u32, dest: u32) {
    self.buffer.push(0x89);
    let specify_regs = MacroAssembler::specify_rr(src, dest);
    self.buffer.push(specify_regs);
  }
  pub fn emit_movl_rm(&mut self, src: u32, ptr_dest: u32) {
    if src.nth_bit_bool(3) {
      self.buffer.push(MacroAssembler::REXRB);
    } else {
      self.buffer.push(MacroAssembler::REXB);
    }
    let specify_regs = MacroAssembler::specify_rm(src, ptr_dest); 
    self.buffer.push(0x89);
    self.buffer.push(specify_regs);
  }
  pub fn emit_movl_rm_sib_offset(&mut self, src: u32, ptr_dest: u32, idx: u32, scale: u32, offset: u32) {
    let specify_regs = MacroAssembler::specify_rm(src, ptr_dest);
    let sib = MacroAssembler::specify_sib(ptr_dest, idx, scale);
    self.buffer.push(0x89);
    self.buffer.push(sib);
    self.buffer.push(specify_regs);
    self.emit_imm32(offset);
  }
  pub fn emit_movl_rm_offset(&mut self, src: u32, ptr_dest: u32, offset: u32) {
    let specify_regs = MacroAssembler::specify_rm(src, ptr_dest); 
    //let sib = MacroAssembler::specify_sib(ptr_dest, ptr_dest, 1);
    self.buffer.push(0x89);
    //self.buffer.push(sib);
    self.buffer.push(specify_regs);
    self.emit_imm32(offset);
  }
  pub fn emit_movl_mr(&mut self, ptr_src: u32, dest: u32) {
    if ptr_src.nth_bit_bool(3) {
      self.buffer.push(MacroAssembler::REXRB);
    } else {
      self.buffer.push(MacroAssembler::REXB);
    }
    let specify_regs = MacroAssembler::specify_mr(ptr_src, dest);
    self.buffer.push(0x8b);
    self.buffer.push(specify_regs);
  }
  pub fn emit_movl_ir(&mut self, imm32: u32, dest: u32) {
    if dest.nth_bit_bool(3) {
      self.buffer.push(MacroAssembler::REXB);
    };
    let specify_reg = dest.lowest_bits(3) as u8;
    self.buffer.push(0xb8 | specify_reg);
    self.emit_imm32(imm32);
  }
  pub fn emit_movq_ir(&mut self, imm64: u64, dest: u32) {
    self.buffer.push(MacroAssembler::REXWB);
    let specify_reg = dest.lowest_bits(4) as u8;
    self.buffer.push(0xb0 | specify_reg);
    self.emit_imm64(imm64);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_movq() {
    let mut masm = MacroAssembler::new();
    const TEST_VALUE: u64 = 0xdeadbeef_bfc0_0001;
    masm.emit_movq_ir(TEST_VALUE, X64RegNum::R8 as u32);
    let jit_fn = masm.compile_buffer().unwrap();
    let out: u64;
    unsafe {
      asm!("callq *$1"
          :"={r8}"(out)
          :"r"(jit_fn.name));
    }
    assert_eq!(out, TEST_VALUE);
  }
}
