use crate::register::BitTwiddle;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::macro_assembler::Label;
use crate::jit::x64_jit::register_allocator::*;

impl MacroAssembler {
  fn emit_label_placeholder_i32(&mut self, label: Label) {
    let placeholder_location = self.buffer.len();
    self.buffer.push(MacroAssembler::LABEL_PLACEHOLDER);
    self.buffer.push(MacroAssembler::LABEL_PLACEHOLDER);
    self.buffer.push(MacroAssembler::LABEL_PLACEHOLDER);
    self.buffer.push(MacroAssembler::LABEL_PLACEHOLDER);
    self.labels_used.insert(label, placeholder_location);
  }
  //TODO: test this
  pub fn emit_callq_i(&mut self, imm32: i32) {
    match imm32 {
      -0x8000..=0x7fff => {
        self.emit_callw_i(imm32 as i16);
      },
      _ => {
        self.buffer.push(0xe8);
        self.emit_imm32(imm32 as u32);
      },
    }
  }
  //TODO: test this
  pub fn emit_callw_i(&mut self, imm16: i16) {
    self.buffer.push(0x66);
    self.buffer.push(0xe8);
    self.emit_imm16(imm16 as u16);
  }
  //TODO: test this
  pub fn emit_call_label(&mut self, label: Label) {
    self.buffer.push(0xe8);
    self.emit_label_placeholder_i32(label);
  }
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
        llvm_asm!("callq *%rbp"
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
        llvm_asm!("callq *%rbp"
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
      unsafe {
        llvm_asm!("callq *%rbp"
            :"={rax}"(out)
            :"{rbp}"(jit_fn.name));
      }
      assert_eq!(out, 1);
    }
  }

  #[test]
  fn call_label() {
      let mut masm = MacroAssembler::new();
      let label = masm.create_undefined_label();
      let end = masm.create_undefined_label();
      masm.emit_movq_ir(0,0);
      masm.emit_call_label(label);
      masm.emit_jmp_label(end);
      masm.define_label(label);
      masm.emit_movq_ir(1,0);
      masm.emit_ret();
      masm.define_label(end);
      let jit_fn = masm.compile_buffer().unwrap();
      println!("{:x?}", masm.buffer);
      let out: u64;
      unsafe {
        llvm_asm!("callq *%rbp"
            :"={rax}"(out)
            :"{rbp}"(jit_fn.name));
            
      }
      assert_eq!(out, 1);
  }
}
