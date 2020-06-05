use crate::register::BitTwiddle;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::register_allocator::X64_RAX;

impl MacroAssembler {
  fn emit_add_rr(&mut self, src: u32, dest: u32) {
    self.buffer.push(0x01);
    self.buffer.push(0xc0 | (src.lowest_bits(3) << 3) as u8 | dest.lowest_bits(3) as u8);
  }
  fn emit_add_ir(&mut self, imm32: i32, dest: u32) {
    match imm32 {
      -128..=127 => {
        self.buffer.push(0x83);
        self.buffer.push(0xc0 | dest.lowest_bits(3) as u8);
        self.buffer.push(imm32 as u8);
      },
      _ => {
        if dest == X64_RAX {
          self.buffer.push(0x05);
        } else {
          self.buffer.push(0x81);
          self.buffer.push(0xc0 | dest.lowest_bits(3) as u8);
        }
        self.emit_imm32(imm32 as u32);
      },
    }
  }
  pub fn emit_addl_rr(&mut self, src: u32, dest: u32) {
    self.emit_conditional_rexrb(src, dest);
    self.emit_add_rr(src, dest);
  }
  pub fn emit_addq_rr(&mut self, src: u32, dest: u32) {
    let rex_prefix = MacroAssembler::REXW |
                     MacroAssembler::conditional_rexb(dest) |
                     MacroAssembler::conditional_rexr(src);
    self.buffer.push(rex_prefix);
    self.emit_add_rr(src, dest);
  }
  pub fn emit_addl_ir(&mut self, imm32: i32, dest: u32) {
    self.emit_conditional_rexb(dest);
    self.emit_add_ir(imm32, dest);
  }
  pub fn emit_addq_ir(&mut self, imm32: i32, dest: u32) {
    let rex_prefix = MacroAssembler::REXW |
                     MacroAssembler::conditional_rexb(dest);
    self.buffer.push(rex_prefix);
    self.emit_add_ir(imm32, dest);
  }
  //FIXME: the general case (i.e. not eax) seems to be wrong here
  //TODO: test this
  pub fn emit_xorw_ir(&mut self, imm16: u16, dest: u32) {
    self.buffer.push(0x66);
    if dest == X64_RAX {
      self.buffer.push(0x35);
    } else {
      if dest.nth_bit_bool(3) {
        self.buffer.push(MacroAssembler::REXB);
      };
      self.buffer.push(0x81);
      let specify_reg = dest.lowest_bits(3) as u8;
      self.buffer.push(0xc8 | specify_reg);
    }
    self.emit_imm16(imm16);
  }
  //TODO: test this
  pub fn emit_orw_ir(&mut self, imm16: u16, dest: u32) {
    self.buffer.push(0x66);
    if dest == X64_RAX {
      self.buffer.push(0x0d);
    } else {
      if dest.nth_bit_bool(3) {
        self.buffer.push(MacroAssembler::REXB);
      };
      self.buffer.push(0x81);
      let specify_reg = dest.lowest_bits(3) as u8;
      self.buffer.push(0xc8 | specify_reg);
    }
    self.emit_imm16(imm16);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn addq_rr() {
    for src in MacroAssembler::free_regs() {
      for dest in MacroAssembler::free_regs() {
        let mut masm = MacroAssembler::new();
        let x = 0x1238_4289_fdf0_0123;
        let y = 0x1434_5892_ffbc_bcc0;
        masm.emit_movq_ir(x, src);
        if src != dest {
          masm.emit_movq_ir(y, dest);
        }
        masm.emit_addq_rr(src, dest);
        masm.emit_movq_rr(dest, 0);
        let jit_fn = masm.compile_buffer().unwrap();
        let out: u64;
        unsafe {
          asm!("callq *%rbp"
              :"={rax}"(out)
              :"{rbp}"(jit_fn.name));
        }
        if src != dest {
          assert_eq!(out, x + y);
        } else {
          assert_eq!(out, x + x);
        }
      }
    }
  }

  #[test]
  fn addl_rr() {
    for src in MacroAssembler::free_regs() {
      for dest in MacroAssembler::free_regs() {
        let mut masm = MacroAssembler::new();
        let x = 0x1238_4289;
        let y = 0x1434_5892;
        masm.emit_movl_ir(x, src);
        if src != dest {
          masm.emit_movl_ir(y, dest);
        }
        masm.emit_addl_rr(src, dest);
        masm.emit_movl_rr(dest, 0);
        let jit_fn = masm.compile_buffer().unwrap();
        let out: u32;
        unsafe {
          asm!("callq *%rbp"
              :"={rax}"(out)
              :"{rbp}"(jit_fn.name));
        }
        if src != dest {
          assert_eq!(out, x + y);
        } else {
          assert_eq!(out, x + x);
        }
      }
    }
  }

  #[test]
  fn addq_imm8_r() {
    for &imm8 in &[-128, 127] {
      for dest in MacroAssembler::free_regs() {
        let mut masm = MacroAssembler::new();
        let x = 0x1238_4289_2378_2395;
        masm.emit_movq_ir(x, dest);
        masm.emit_addq_ir(imm8, dest);
        masm.emit_movq_rr(dest, 0);
        let jit_fn = masm.compile_buffer().unwrap();
        let out: u64;
        unsafe {
          asm!("callq *%rbp"
              :"={rax}"(out)
              :"{rbp}"(jit_fn.name));
        }
        assert_eq!(out, x.wrapping_add(imm8 as u64));
      }
    }
  }

  #[test]
  fn addq_imm32_r() {
    for &imm32 in &[-0x7cf0_3295, -0x7cf0_3295] {
      for dest in MacroAssembler::free_regs() {
        let mut masm = MacroAssembler::new();
        let x = 0x1238_4289_2378_2390;
        masm.emit_movq_ir(x, dest);
        masm.emit_addq_ir(imm32, dest);
        masm.emit_movq_rr(dest, 0);
        let jit_fn = masm.compile_buffer().unwrap();
        let out: u64;
        unsafe {
          asm!("callq *%rbp"
              :"={rax}"(out)
              :"{rbp}"(jit_fn.name));
        }
        assert_eq!(out, x.wrapping_add(imm32 as u64));
      }
    }
  }

  #[test]
  fn addl_imm8_r() {
    for &imm8 in &[-128, 127] {
      for dest in MacroAssembler::free_regs() {
        let mut masm = MacroAssembler::new();
        let x = 0x1238_4289;
        masm.emit_movl_ir(x, dest);
        masm.emit_addl_ir(imm8, dest);
        masm.emit_movl_rr(dest, 0);
        let jit_fn = masm.compile_buffer().unwrap();
        let out: u32;
        unsafe {
          asm!("callq *%rbp"
              :"={rax}"(out)
              :"{rbp}"(jit_fn.name));
        }
        assert_eq!(out, x.wrapping_add(imm8 as u32));
      }
    }
  }

  #[test]
  fn addl_imm32_r() {
    for &imm32 in &[-0x7cf0_3295, -0x7cf0_3295] {
      for dest in MacroAssembler::free_regs() {
        let mut masm = MacroAssembler::new();
        let x = 0x1238_4289;
        masm.emit_movl_ir(x, dest);
        masm.emit_addl_ir(imm32, dest);
        masm.emit_movl_rr(dest, 0);
        let jit_fn = masm.compile_buffer().unwrap();
        let out: u32;
        unsafe {
          asm!("callq *%rbp"
              :"={rax}"(out)
              :"{rbp}"(jit_fn.name));
        }
        assert_eq!(out, x.wrapping_add(imm32 as u32));
      }
    }
  }
}
