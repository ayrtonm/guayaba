use crate::register::BitTwiddle;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::register_allocator::X64RegNum;

impl MacroAssembler {
  pub fn emit_push_imm8(&mut self, imm8: u8) {
    self.buffer.push(0x6A);
    self.buffer.push(imm8);
  }
  pub fn emit_push_imm16(&mut self, imm16: u16) {
    self.buffer.push(0x68);
    self.emit_imm16(imm16);
  }
  pub fn emit_push_imm32(&mut self, imm32: u32) {
    self.buffer.push(0x68);
    self.emit_imm32(imm32);
  }
  pub fn emit_pop_r16(&mut self, reg: u32) {
    self.buffer.push(0x58 | reg.lowest_bits(3) as u8);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn push_pop() {
    let mut masm = MacroAssembler::new();
    masm.emit_push_imm32(0xdead_beef);
    masm.emit_pop_r16(X64RegNum::RAX as u32);
    let jit_fn = masm.compile_buffer().unwrap();
    let out: u32;
    unsafe {
      asm!("callq *$1"
          :"={rax}"(out)
          :"r"(jit_fn.name));
    }
    assert!(out == 0xdead_beef);
  }
}
