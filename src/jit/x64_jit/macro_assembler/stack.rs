use crate::register::BitTwiddle;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::register_allocator::*;

//note that none of these methods should be called with %rsp as an operand
//while these types of instructions are encodable, I should avoid having the JIT
//produce complex x64 code for now
impl MacroAssembler {
  //TODO: test this
  pub fn emit_pushfq(&mut self) {
    self.buffer.push(0x9c);
  }
  //TODO: test this
  pub fn emit_popfq(&mut self) {
    self.buffer.push(0x9d);
  }
  pub fn emit_push_r64(&mut self, reg: u32) {
    assert!(reg != X64_RSP);
    self.emit_conditional_rexb(reg);
    self.buffer.push(0x50 | reg.lowest_bits(3) as u8);
  }
  pub fn emit_push_r16(&mut self, reg: u32) {
    assert!(reg != X64_RSP);
    self.emit_16bit_prefix();
    self.emit_push_r64(reg);
  }
  pub fn emit_pop_r64(&mut self, reg: u32) {
    assert!(reg != X64_RSP);
    self.emit_conditional_rexb(reg);
    self.buffer.push(0x58 | reg.lowest_bits(3) as u8);
  }
  pub fn emit_pop_r16(&mut self, reg: u32) {
    assert!(reg != X64_RSP);
    self.emit_16bit_prefix();
    self.emit_pop_r64(reg);
  }
  //TODO: modify this to use xchg and not trash %rax
  pub fn emit_push_imm64(&mut self, imm64: u64) {
    self.emit_movq_ir(imm64, X64_RAX);
    self.emit_push_r64(X64_RAX);
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
    assert!(reg != X64_RSP);
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
    assert!(reg != X64_RSP);
    self.emit_conditional_rexb(reg);
    self.buffer.push(0x8f);
    self.buffer.push(0x00 | reg.lowest_bits(3) as u8);
  }
  pub fn emit_push_m16(&mut self, reg: u32) {
    assert!(reg != X64_RSP);
    self.emit_16bit_prefix();
    self.emit_push_m32(reg);
  }
  pub fn emit_pop_m16(&mut self, reg: u32) {
    assert!(reg != X64_RSP);
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

  #[test]
  fn stack_64bit() {
    for reg in MacroAssembler::free_regs() {
      let mut masm = MacroAssembler::new();
      masm.emit_push_imm32(0xdead_beef);
      masm.emit_pop_r64(reg);
      masm.emit_push_r64(reg);
      masm.emit_pop_r64(0);
      let jit_fn = masm.compile_buffer().unwrap();
      let out: u32;
      unsafe {
        llvm_asm!("callq *%rbp"
            :"={rax}"(out)
            :"{rbp}"(jit_fn.name));
      }
      assert_eq!(out as i64 as u64, 0xdead_beef);
    }
  }
  #[test]
  fn stack_16bit() {
    for reg in MacroAssembler::free_regs() {
      let mut masm = MacroAssembler::new();
      masm.emit_push_imm16(0xbeef);
      masm.emit_pop_r16(reg);
      masm.emit_push_r16(reg);
      masm.emit_pop_r16(0);
      let jit_fn = masm.compile_buffer().unwrap();
      let out: u16;
      unsafe {
        llvm_asm!("callq *%rbp"
            :"={ax}"(out)
            :"{rbp}"(jit_fn.name));
      }
      assert_eq!(out, 0xbeef);
    }
  }
  #[test]
  fn stack_memory_16bit() {
    for reg in MacroAssembler::free_regs() {
      let read_me: u16 = 0xf0f0;
      let ptr = &read_me as *const u16 as u64;
      let mut masm = MacroAssembler::new();
      masm.emit_movq_ir(ptr, reg);
      masm.emit_push_m16(reg);
      masm.emit_pop_r16(0);
      let jit_fn = masm.compile_buffer().unwrap();
      let out: u16;
      unsafe {
        llvm_asm!("callq *%rbp"
            :"={ax}"(out)
            :"{rbp}"(jit_fn.name));
      }
      assert_eq!(out, read_me);
    }
  }
  #[test]
  fn stack_memory_32bit() {
    for reg in MacroAssembler::free_regs() {
      let read_me: u32 = 0xff00_f0f0;
      let ptr = &read_me as *const u32 as u64;
      let mut masm = MacroAssembler::new();
      masm.emit_movq_ir(ptr, reg);
      masm.emit_push_m32(reg);
      masm.emit_pop_r64(0);
      let jit_fn = masm.compile_buffer().unwrap();
      let out: u32;
      unsafe {
        llvm_asm!("callq *%rbp"
            :"={rax}"(out)
            :"{rbp}"(jit_fn.name));
      }
      assert_eq!(out, read_me);
    }
  }
}
