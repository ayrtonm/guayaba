use crate::register::BitTwiddle;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::register_allocator::*;

impl MacroAssembler {
  pub fn emit_callq_r64(&mut self, reg: u32) {
    self.emit_conditional_rexb(reg);
    self.buffer.push(0xff);
    self.buffer.push(0xd0 | reg.lowest_bits(3) as u8);
  }
  pub fn emit_callq_m64(&mut self, ptr: u32) {
    self.emit_conditional_rexb(ptr);
    self.buffer.push(0xff);
    if ptr.lowest_bits(3) == 5 {
      self.buffer.push(0x55);
      self.buffer.push(0x00);
    } else {
      self.buffer.push(0x10 | ptr.lowest_bits(3) as u8);
      if ptr.lowest_bits(3) == 4 {
        self.buffer.push(0x24);
      }
    }
  }
  pub fn emit_callq_m64_offset(&mut self, ptr: u32, offset: i32) {
    if offset == 0 {
      self.emit_callq_m64(ptr);
    } else {
      self.emit_conditional_rexb(ptr);
      self.buffer.push(0xff);
      match offset {
        0 => unreachable!(""),
        -128..=127 => {
          self.buffer.push(0x50 | ptr.lowest_bits(3) as u8);
          if ptr.lowest_bits(3) == 4 {
            self.buffer.push(0x24);
          };
          self.buffer.push(offset as u8);
        },
        _ => {
          self.buffer.push(0x90 | ptr.lowest_bits(3) as u8);
          if ptr.lowest_bits(3) == 4 {
            self.buffer.push(0x24);
          };
          self.emit_imm32(offset as u32);
        },
      }
    }
  }
}

extern "C" fn no_arg() -> u32 {
  println!("called a function with no arguments");
  1
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn callq_r64_no_args() {
    for reg in MacroAssembler::free_regs() {
      let mut masm = MacroAssembler::new();
      masm.emit_movq_ir(no_arg as u64, reg);
      for i in MacroAssembler::caller_saved_regs() {
        masm.emit_push_r64(i);
      }
      masm.emit_callq_r64(reg);
      //store return value in r15 since there's a pop rax coming up
      masm.emit_movq_rr(0, 15);
      for &i in MacroAssembler::caller_saved_regs().iter().rev() {
        masm.emit_pop_r64(i);
      }
      //mov return value back to rax
      masm.emit_movq_rr(15, 0);
      let jit_fn = masm.compile_buffer().unwrap();
      let out: u32;
      unsafe {
        asm!("callq *%rbp"
            :"={rax}"(out)
            :"{rbp}"(jit_fn.name));
      }
      assert_eq!(out, 1);
    }
  }

  #[test]
  fn callq_m64_no_args() {
    for reg in MacroAssembler::free_regs() {
      let mut masm = MacroAssembler::new();
      for i in MacroAssembler::caller_saved_regs() {
        masm.emit_push_r64(i);
      }
      masm.emit_addq_ir(-8, X64_RSP);
      masm.emit_push_imm64(no_arg as u64);
      masm.emit_movq_rr(X64_RSP, reg);
      masm.emit_callq_m64(reg);
      masm.emit_movq_rr(0, 15);
      masm.emit_addq_ir(16, X64_RSP);
      for &i in MacroAssembler::caller_saved_regs().iter().rev() {
        masm.emit_pop_r64(i);
      }
      masm.emit_movq_rr(15, 0);
      let jit_fn = masm.compile_buffer().unwrap();
      let out: u64;
      unsafe {
        asm!("callq *%rbp"
            :"={rax}"(out)
            :"{rbp}"(jit_fn.name));
      }
      assert_eq!(out, 1);
    }
  }

  #[test]
  fn callq_m64_offset_no_args() {
    for reg in MacroAssembler::free_regs() {
      let mut masm = MacroAssembler::new();
      masm.emit_push_imm64(no_arg as u64);
      for i in MacroAssembler::caller_saved_regs() {
        masm.emit_push_r64(i);
      }
      masm.emit_addq_ir(-8, X64_RSP);
      masm.emit_movq_rr(X64_RSP, reg);
      masm.emit_callq_m64_offset(reg, MacroAssembler::caller_saved_regs().len() as i32 * 8);
      masm.emit_addq_ir(8, X64_RSP);
      masm.emit_movq_rr(0, 15);
      for &i in MacroAssembler::caller_saved_regs().iter().rev() {
        masm.emit_pop_r64(i);
      }
      masm.emit_movq_rr(15, 0);
      masm.emit_addq_ir(8, X64_RSP);
      let jit_fn = masm.compile_buffer().unwrap();
      let out: u64;
      println!("running {}", reg);
      unsafe {
        asm!("callq *%rbp"
            :"={rax}"(out)
            :"{rbp}"(jit_fn.name));
      }
      assert_eq!(out, 1);
    }
  }
}
