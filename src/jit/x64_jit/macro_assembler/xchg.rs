use crate::register::BitTwiddle;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::register_allocator::*;

impl MacroAssembler {
  pub fn emit_xchgq_rr(&mut self, reg1: u32, reg2: u32) {
    if reg1 == X64_RAX || reg2 == X64_RAX {
      let rex_prefix = MacroAssembler::REXW |
                       MacroAssembler::conditional_rexb(reg1 | reg2);
      self.buffer.push(rex_prefix);
      self.buffer.push(0x90 | (reg1 | reg2).lowest_bits(3) as u8);
    } else {
      let rex_prefix = MacroAssembler::REXW |
                       MacroAssembler::conditional_rexb(reg1) |
                       MacroAssembler::conditional_rexr(reg2);
      self.buffer.push(rex_prefix);
      self.buffer.push(0x87);
      self.buffer.push(0xc0 | reg1.lowest_bits(3) as u8 | (reg2.lowest_bits(3) << 3) as u8);
    }
  }
  pub fn emit_xchgl_rm(&mut self, reg: u32, ptr: u32) {
    self.emit_conditional_rexrb(reg, ptr);
    self.buffer.push(0x87);
    if ptr.lowest_bits(3) == 5 {
      self.buffer.push(0x45 | (reg.lowest_bits(3) << 3) as u8);
      self.buffer.push(0x00);
    } else {
      self.buffer.push((reg.lowest_bits(3) << 3) as u8 | ptr.lowest_bits(3) as u8);
      if ptr.lowest_bits(3) == 4 {
        self.buffer.push(0x24);
      }
    }
  }
  pub fn emit_xchgl_rm_offset(&mut self, reg: u32, ptr: u32, offset: i32) {
    if offset == 0 {
      self.emit_xchgl_rm(reg, ptr);
    } else {
      self.emit_conditional_rexrb(reg, ptr);
      self.buffer.push(0x87);
      self.buffer.push(0x40 | (reg.lowest_bits(3) << 3) as u8 | ptr.lowest_bits(3) as u8);
      if ptr.lowest_bits(3) == 4 {
        self.buffer.push(0x24);
      }
      match offset {
        0 => unreachable!(""),
        -128..=127 => self.buffer.push(offset as u8),
        _ => self.emit_imm32(offset as u32),
      }
    }
  }
  ////TODO: understand the behavior of this then test it
  //pub fn emit_xchgq_rm(&mut self, reg: u32, ptr: u32) {
  //  self.emit_conditional_rexwrb(reg, ptr);
  //  self.buffer.push(0x87);
  //  if ptr.lowest_bits(3) == X64_RBP {
  //    self.buffer.push(0x45 | (reg.lowest_bits(3) << 3) as u8);
  //    self.buffer.push(0x00);
  //  } else {
  //    self.buffer.push((reg.lowest_bits(3) << 3) as u8 | ptr.lowest_bits(3) as u8);
  //    if ptr.lowest_bits(3) == X64_RSP {
  //      self.buffer.push(0x24);
  //    }
  //  }
  //}
  ////TODO: test this
  //pub fn emit_xchgq_rm_offset(&mut self, reg: u32, ptr: u32, offset: i32) {
  //}
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn xchgq_rr() {
    for r1 in MacroAssembler::free_regs() {
      for r2 in MacroAssembler::free_regs() {
        let mut masm = MacroAssembler::new();
        let x = 0xabec_230f_0f03_4452;
        masm.emit_movq_ir(x, r1);
        masm.emit_xchgq_rr(r1, r2);
        masm.emit_movq_rr(r2, 0);
        let jit_fn = masm.compile_buffer().unwrap();
        let out: u64;
        unsafe {
          asm!("callq *%rbp"
              :"={rax}"(out)
              :"{rbp}"(jit_fn.name));
        }
        assert_eq!(x,out);
      }
    }
  }

  #[test]
  fn xchgl_rm() {
    for reg in MacroAssembler::free_regs() {
      for ptr in MacroAssembler::all_regs() {
        let mut masm = MacroAssembler::new();
        let x = 0x0703_b452;
        masm.emit_movl_ir(x, 0);
        masm.emit_push_r64(0);
        masm.emit_movq_rr(X64_RSP, ptr);
        masm.emit_xchgl_rm(reg, ptr);
        masm.emit_movl_rr(reg, 0);
        masm.emit_addq_ir(8, X64_RSP);
        let jit_fn = masm.compile_buffer().unwrap();
        let out: u32;
        unsafe {
          asm!("callq *%rbp"
              :"={rax}"(out)
              :"{rbp}"(jit_fn.name));
        }
        assert_eq!(x,out);
      }
    }
  }

  #[test]
  fn xchgl_rm_offset() {
    for reg in MacroAssembler::free_regs() {
      for ptr in MacroAssembler::all_regs() {
        let mut masm = MacroAssembler::new();
        let x = 0x0703_b452;
        masm.emit_movl_ir(x, 0);
        masm.emit_push_r64(0);
        masm.emit_push_r64(1);
        masm.emit_movq_rr(X64_RSP, ptr);
        masm.emit_xchgl_rm_offset(reg, ptr, 8);
        masm.emit_movl_rr(reg, 0);
        masm.emit_addq_ir(16, X64_RSP);
        let jit_fn = masm.compile_buffer().unwrap();
        let out: u32;
        unsafe {
          asm!("callq *%rbp"
              :"={rax}"(out)
              :"{rbp}"(jit_fn.name));
        }
        assert_eq!(x,out);
      }
    }
  }

  //FIXME: see emit_xchgq_rm
  //#[test]
  //fn xchgq_rm() {
  //  for reg in MacroAssembler::free_regs() {
  //    for ptr in MacroAssembler::free_regs() {
  //      let mut masm = MacroAssembler::new();
  //      let x = 0x0703_b452_ffde_423b;
  //      masm.emit_movq_ir(x, 0);
  //      masm.emit_push_r64(0);
  //      masm.emit_movq_rr(X64_RSP, ptr);
  //      masm.emit_xchgq_rm(reg, ptr);
  //      masm.emit_movq_rr(reg, 0);
  //      masm.emit_addq_ir(8, X64_RSP);
  //      let jit_fn = masm.compile_buffer().unwrap();
  //      let out: u64;
  //      unsafe {
  //        asm!("callq *%rbp"
  //            :"={rax}"(out)
  //            :"{rbp}"(jit_fn.name));
  //      }
  //      if reg != ptr {
  //        assert_eq!(x,out);
  //      }
  //    }
  //  }
  //}
}
