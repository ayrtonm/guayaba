use crate::register::BitTwiddle;
use crate::jit::x64_jit::macro_compiler::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::macro_compiler::macro_assembler::registers::*;

//note that none of these methods should be called with %rsp as an operand
//while these types of instructions are encodable, I should avoid having the JIT
//produce complex x64 code for now
#[deny(unused_must_use)]
impl MacroAssembler {
  #[must_use]
  pub fn realign_stack(&mut self, misalignment: isize) -> isize {
    println!("realigning stack {}", misalignment);
    self.emit_addq_ir(misalignment as i32, X64_RSP);
    -misalignment
  }
  //TODO: test this
  #[must_use]
  pub fn emit_pushfq(&mut self) -> isize {
    self.buffer.push(0x9c);
    8
  }
  //TODO: test this
  #[must_use]
  pub fn emit_popfq(&mut self) -> isize {
    self.buffer.push(0x9d);
    -8
  }
  fn emit_push_r(&mut self, reg: u32) {
    self.emit_conditional_rexb(reg);
    self.buffer.push(0x50 | reg.lowest_bits(3) as u8);
  }
  #[must_use]
  pub fn emit_pushq_r(&mut self, reg: u32) -> isize {
    assert!(reg != X64_RSP);
    self.emit_push_r(reg);
    8
  }
  #[must_use]
  pub fn emit_pushw_r(&mut self, reg: u32) -> isize {
    assert!(reg != X64_RSP);
    self.emit_16bit_prefix();
    self.emit_push_r(reg);
    2
  }
  fn emit_pop_r(&mut self, reg: u32) {
    self.emit_conditional_rexb(reg);
    self.buffer.push(0x58 | reg.lowest_bits(3) as u8);
  }
  #[must_use]
  pub fn emit_popq_r(&mut self, reg: u32) -> isize {
    assert!(reg != X64_RSP);
    self.emit_pop_r(reg);
    -8
  }
  #[must_use]
  pub fn emit_popw_r(&mut self, reg: u32) -> isize {
    assert!(reg != X64_RSP);
    self.emit_16bit_prefix();
    self.emit_pop_r(reg);
    -2
  }
  //TODO: modify this to use xchg and not trash %rax
  #[must_use]
  pub fn emit_pushq_i(&mut self, imm64: u64) -> isize {
    self.emit_movq_ir(imm64, X64_RAX);
    self.emit_pushq_r(X64_RAX)
  }
  #[must_use]
  pub fn emit_pushl_i(&mut self, imm32: u32) -> isize {
    self.buffer.push(0x68);
    self.emit_imm32(imm32);
    4
  }
  #[must_use]
  pub fn emit_pushw_i(&mut self, imm16: u16) -> isize {
    self.emit_16bit_prefix();
    self.buffer.push(0x68);
    self.emit_imm16(imm16);
    2
  }
  fn emit_push_m(&mut self, reg: u32) {
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
  #[must_use]
  pub fn emit_pushl_m(&mut self, reg: u32) -> isize {
    assert!(reg != X64_RSP);
    self.emit_push_m(reg);
    4
  }
  #[must_use]
  pub fn emit_pushw_m(&mut self, reg: u32) -> isize {
    assert!(reg != X64_RSP);
    self.emit_16bit_prefix();
    self.emit_push_m(reg);
    2
  }
  fn emit_pop_m(&mut self, reg: u32) {
    self.emit_conditional_rexb(reg);
    self.buffer.push(0x8f);
    self.buffer.push(0x00 | reg.lowest_bits(3) as u8);
  }
  #[must_use]
  pub fn emit_popl_m(&mut self, reg: u32) -> isize {
    assert!(reg != X64_RSP);
    self.emit_pop_m(reg);
    -4
  }
  #[must_use]
  pub fn emit_popw_m(&mut self, reg: u32) -> isize {
    assert!(reg != X64_RSP);
    self.emit_16bit_prefix();
    self.emit_pop_m(reg);
    -2
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
      let mut stack_offset = 0;
      stack_offset += masm.emit_pushl_i(0xdead_beef);
      stack_offset += masm.emit_popq_r(reg);
      stack_offset += masm.emit_pushq_r(reg);
      stack_offset += masm.emit_popq_r(0);
      stack_offset += masm.realign_stack(stack_offset);
      assert_eq!(stack_offset, 0);
      let jit_fn = masm.assemble().unwrap();
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
      let mut stack_offset = 0;
      stack_offset += masm.emit_pushw_i(0xbeef);
      stack_offset += masm.emit_popw_r(reg);
      stack_offset += masm.emit_pushw_r(reg);
      stack_offset += masm.emit_popw_r(0);
      stack_offset += masm.realign_stack(stack_offset);
      assert_eq!(stack_offset, 0);
      let jit_fn = masm.assemble().unwrap();
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
      let mut stack_offset = 0;
      stack_offset += masm.emit_pushw_m(reg);
      stack_offset += masm.emit_popw_r(0);
      stack_offset += masm.realign_stack(stack_offset);
      assert_eq!(stack_offset, 0);
      let jit_fn = masm.assemble().unwrap();
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
      let mut stack_offset = 0;
      stack_offset += masm.emit_pushl_m(reg);
      stack_offset += masm.emit_popq_r(0);
      stack_offset += masm.realign_stack(stack_offset);
      assert_eq!(stack_offset, 0);
      let jit_fn = masm.assemble().unwrap();
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
