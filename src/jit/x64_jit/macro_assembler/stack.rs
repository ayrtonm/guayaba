use crate::register::BitTwiddle;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::register_allocator::X64RegNum;

//note that none of these methods should be called with %rsp as an operand
//while these types of instructions are encodable, I should avoid having the JIT
//produce complex x64 code for now
impl MacroAssembler {
  pub fn emit_push_r32(&mut self, reg: u32) {
    assert!(reg != X64RegNum::RSP as u32);
    self.emit_conditional_rexb(reg);
    self.buffer.push(0x50 | reg.lowest_bits(3) as u8);
  }
  pub fn emit_push_r16(&mut self, reg: u32) {
    assert!(reg != X64RegNum::RSP as u32);
    self.emit_16bit_prefix();
    self.emit_push_r32(reg);
  }
  pub fn emit_pop_r32(&mut self, reg: u32) {
    assert!(reg != X64RegNum::RSP as u32);
    self.emit_conditional_rexb(reg);
    self.buffer.push(0x58 | reg.lowest_bits(3) as u8);
  }
  pub fn emit_pop_r16(&mut self, reg: u32) {
    assert!(reg != X64RegNum::RSP as u32);
    self.emit_16bit_prefix();
    self.emit_pop_r32(reg);
  }
  pub fn emit_push_imm32(&mut self, imm32: u32) {
    self.buffer.push(0x68);
    self.emit_imm32(imm32);
  }
  pub fn emit_push_imm16(&mut self, imm16: u16) {
    self.emit_16bit_prefix();
    self.buffer.push(0x68);
    self.emit_imm16(imm16);
  }
  pub fn emit_push_m32(&mut self, reg: u32) {
    assert!(reg != X64RegNum::RSP as u32);
    self.emit_conditional_rexb(reg);
    self.buffer.push(0xff);
    if reg.lowest_bits(3) == 5 {
      self.buffer.push(0x75);
      self.buffer.push(0x00);
    } else {
      self.buffer.push(0x30 | reg.lowest_bits(3) as u8);
      if reg.lowest_bits(3) == 4 {
        self.buffer.push(0x24);
      };
    }
  }
  pub fn emit_pop_m32(&mut self, reg: u32) {
    assert!(reg != X64RegNum::RSP as u32);
    self.emit_conditional_rexb(reg);
    self.buffer.push(0x8f);
    self.buffer.push(0x00 | reg.lowest_bits(3) as u8);
  }
  pub fn emit_push_m16(&mut self, reg: u32) {
    assert!(reg != X64RegNum::RSP as u32);
    self.emit_16bit_prefix();
    self.emit_push_m32(reg);
  }
  pub fn emit_pop_m16(&mut self, reg: u32) {
    assert!(reg != X64RegNum::RSP as u32);
    self.emit_16bit_prefix();
    self.emit_pop_m32(reg);
  }
}

//the JIT functions here may return values of various sizes so we have to call
//them with inline assembly. Note that there are implicit conversions from fn()
//to u64 when assigning %rbp for callq
#[cfg(test)]
mod tests {
  use super::*;

  fn test_regs() -> Vec<u32> {
    (0..=15).filter(|&x| x != X64RegNum::RSP as u32).collect()
  }
  #[test]
  fn stack_32bit() {
    for reg in test_regs() {
      let mut masm = MacroAssembler::new();
      masm.emit_push_imm32(0xdead_beef);
      masm.emit_pop_r32(reg);
      masm.emit_push_r32(reg);
      masm.emit_pop_r32(0);
      let jit_fn = masm.compile_buffer().unwrap();
      let out: u32;
      unsafe {
        asm!("callq *%rbp"
            :"={rax}"(out)
            :"{rbp}"(jit_fn.name));
      }
      assert_eq!(out as i64 as u64, 0xdead_beef);
    }
  }
  #[test]
  fn stack_16bit() {
    for reg in test_regs() {
      let mut masm = MacroAssembler::new();
      masm.emit_push_imm16(0xbeef);
      masm.emit_pop_r16(reg);
      masm.emit_push_r16(reg);
      masm.emit_pop_r16(0);
      let jit_fn = masm.compile_buffer().unwrap();
      let out: u16;
      unsafe {
        asm!("callq *%rbp"
            :"={ax}"(out)
            :"{rbp}"(jit_fn.name));
      }
      assert_eq!(out, 0xbeef);
    }
  }
  #[test]
  fn stack_memory_16bit() {
    for reg in test_regs() {
      let read_me: u16 = 0xf0f0;
      let ptr = &read_me as *const u16 as u64;
      let mut masm = MacroAssembler::new();
      masm.emit_movq_ir(ptr, reg);
      masm.emit_push_m16(reg);
      masm.emit_pop_r16(0);
      let jit_fn = masm.compile_buffer().unwrap();
      let out: u16;
      unsafe {
        asm!("callq *%rbp"
            :"={ax}"(out)
            :"{rbp}"(jit_fn.name));
      }
      assert_eq!(out, read_me);
    }
  }
  #[test]
  fn stack_memory_32bit() {
    for reg in test_regs() {
      let read_me: u32 = 0xff00_f0f0;
      let ptr = &read_me as *const u32 as u64;
      let mut masm = MacroAssembler::new();
      masm.emit_movq_ir(ptr, reg);
      masm.emit_push_m32(reg);
      masm.emit_pop_r32(0);
      let jit_fn = masm.compile_buffer().unwrap();
      let out: u32;
      unsafe {
        asm!("callq *%rbp"
            :"={rax}"(out)
            :"{rbp}"(jit_fn.name));
      }
      assert_eq!(out, read_me);
    }
  }
}
