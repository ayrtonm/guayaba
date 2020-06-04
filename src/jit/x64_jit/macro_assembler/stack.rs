use crate::register::BitTwiddle;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::register_allocator::X64RegNum;

impl MacroAssembler {
  pub fn emit_push_r32(&mut self, reg: u32) {
    if reg.nth_bit_bool(3) {
      self.emit_rexb();
    };
    self.buffer.push(0x50 | reg.lowest_bits(3) as u8);
  }
  pub fn emit_push_r16(&mut self, reg: u32) {
    self.emit_16bit_prefix();
    self.emit_push_r32(reg);
  }
  pub fn emit_pop_r32(&mut self, reg: u32) {
    if reg.nth_bit_bool(3) {
      self.emit_rexb();
    };
    self.buffer.push(0x58 | reg.lowest_bits(3) as u8);
  }
  pub fn emit_pop_r16(&mut self, reg: u32) {
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
  //FIXME: may be wrong for rsp and rbp
  pub fn emit_push_m64(&mut self, reg: u32) {
    if reg.nth_bit_bool(3) {
      self.emit_rexb();
    };
    self.buffer.push(0xff);
    self.buffer.push(0x30 | reg.lowest_bits(3) as u8);
  }
  //FIXME: may be wrong for rsp and rbp
  pub fn emit_pop_m64(&mut self, reg: u32) {
    if reg.nth_bit_bool(3) {
      self.emit_rexb();
    };
    self.buffer.push(0x8f);
    self.buffer.push(0x00 | reg.lowest_bits(3) as u8);
  }
  pub fn emit_push_m16(&mut self, reg: u32) {
    self.emit_16bit_prefix();
    self.emit_push_m64(reg);
  }
  pub fn emit_pop_m16(&mut self, reg: u32) {
    self.emit_16bit_prefix();
    self.emit_pop_m64(reg);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn test_regs() -> Vec<u32> {
    (0..=15).filter(|&x| x != 4 && x != 5).collect()
  }
  fn save_reserved_registers(masm: &mut MacroAssembler) {
    masm.emit_push_r32(12);
    masm.emit_push_r32(13);
    masm.emit_push_r32(14);
    masm.emit_push_r32(15);
  }
  fn load_reserved_registers(masm: &mut MacroAssembler) {
    masm.emit_pop_r32(15);
    masm.emit_pop_r32(14);
    masm.emit_pop_r32(13);
    masm.emit_pop_r32(12);
  }
  #[test]
  fn stack_32bit() {
    for reg in test_regs() {
      let mut masm = MacroAssembler::new();
      save_reserved_registers(&mut masm);
      masm.emit_push_imm32(0xdead_beef);
      masm.emit_pop_r32(reg);
      masm.emit_push_r32(reg);
      masm.emit_pop_r32(0);
      load_reserved_registers(&mut masm);
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
      save_reserved_registers(&mut masm);
      masm.emit_push_imm16(0xbeef);
      masm.emit_pop_r16(reg);
      masm.emit_push_r16(reg);
      masm.emit_pop_r16(0);
      load_reserved_registers(&mut masm);
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
      let modify_me: u16 = 0xf0f0;
      let ptr = &modify_me as *const u16 as u64;
      let mut masm = MacroAssembler::new();
      save_reserved_registers(&mut masm);
      //masm.emit_movq_ir(ptr, reg);
      //masm.emit_push_m16(reg);
      //masm.emit_pop_m16(0);
      load_reserved_registers(&mut masm);
      let jit_fn = masm.compile_buffer().unwrap();
      let out: u16;
      unsafe {
        asm!("callq *%rbp"
            :"={ax}"(out)
            :"{rbp}"(jit_fn.name));
      }
    }
  }
}
