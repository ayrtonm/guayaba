use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::macro_assembler::Label;
use crate::jit::x64_jit::register_allocator::*;

impl MacroAssembler {
  const PLACEHOLDER: u8 = 0xff;
  fn emit_label_placeholder(&mut self, label: Label) {
    let placeholder_location = self.buffer.len();
    self.buffer.push(MacroAssembler::PLACEHOLDER);
    self.labels_used.insert(label, placeholder_location);
  }
  pub fn emit_jmp_rel8(&mut self, offset: i8) {
    self.buffer.push(0xeb);
    self.buffer.push(offset as u8);
  }
  pub fn emit_jmp_label(&mut self, label: Label) {
    self.buffer.push(0xeb);
    self.emit_label_placeholder(label);
  }
  pub fn emit_jae_rel8(&mut self, offset: i8) {
    self.buffer.push(0x73);
    self.buffer.push(offset as u8);
  }
  pub fn emit_jb_rel8(&mut self, offset: i8) {
    self.buffer.push(0x72);
    self.buffer.push(offset as u8);
  }
  pub fn emit_jae_label(&mut self, label: Label) {
    self.buffer.push(0x73);
    self.emit_label_placeholder(label);
  }
  pub fn emit_jb_label(&mut self, label: Label) {
    self.buffer.push(0x72);
    self.emit_label_placeholder(label);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn jmp_rel8() {
    let mut masm = MacroAssembler::new();
    masm.emit_movl_ir(0x1235_8732, X64_RAX);
    masm.emit_jmp_rel8(5);
    masm.emit_movl_ir(0, X64_RAX);
    let jit_fn = masm.compile_buffer().unwrap();
    let out: u32;
    unsafe {
      llvm_asm!("callq *%rbp"
          :"={rax}"(out)
          :"{rbp}"(jit_fn.name));
    }
    assert_eq!(out, 0x1235_8732);
  }

  #[test]
  fn jmp_label() {
    for i in 0..=1 {
      let mut masm = MacroAssembler::new();
      let dest = masm.create_undefined_label();
      masm.emit_movl_ir(0x1235_8732, X64_RAX);
      masm.emit_jmp_label(dest);
      if i == 0 {
        masm.emit_movl_ir(0, X64_RAX);
        masm.define_label(dest);
      } else {
        masm.define_label(dest);
        masm.emit_movl_ir(0, X64_RAX);
      }
      let jit_fn = masm.compile_buffer().unwrap();
      let out: u32;
      unsafe {
        llvm_asm!("callq *%rbp"
            :"={rax}"(out)
            :"{rbp}"(jit_fn.name));
      }
      if i == 0 {
        assert_eq!(out, 0x1235_8732);
      } else {
        assert_eq!(out, 0);
      }
    }
  }

  #[test]
  fn jcc_rel8() {
    for test_value in 0..=1 {
      for i in 0..=1 {
        let mut masm = MacroAssembler::new();
        masm.emit_movl_ir(0x1235_8732, X64_RAX);
        masm.emit_movl_ir(test_value, X64_RCX);
        masm.emit_btl_ir(0, X64_RCX);
        if i == 0 {
          masm.emit_jae_rel8(5);
        } else {
          masm.emit_jb_rel8(5);
        }
        masm.emit_movl_ir(0, X64_RAX);
        let jit_fn = masm.compile_buffer().unwrap();
        let out: u32;
        unsafe {
          llvm_asm!("callq *%rbp"
              :"={rax}"(out)
              :"{rbp}"(jit_fn.name));
        }
        if i == test_value {
          assert_eq!(out, 0x1235_8732);
        } else {
          assert_eq!(out, 0);
        }
      }
    }
  }

  #[test]
  fn jcc_label() {
    for test_value in 0..=1 {
      for i in 0..=1 {
        let mut masm = MacroAssembler::new();
        let label = masm.create_undefined_label();
        masm.emit_movl_ir(0x1235_8732, X64_RAX);
        masm.emit_movl_ir(test_value, X64_RCX);
        masm.emit_btl_ir(0, X64_RCX);
        if i == 0 {
          masm.emit_jae_label(label);
        } else {
          masm.emit_jb_label(label);
        }
        masm.emit_movl_ir(0, X64_RAX);
        masm.define_label(label);
        let jit_fn = masm.compile_buffer().unwrap();
        let out: u32;
        unsafe {
          llvm_asm!("callq *%rbp"
              :"={rax}"(out)
              :"{rbp}"(jit_fn.name));
        }
        if i == test_value {
          assert_eq!(out, 0x1235_8732);
        } else {
          assert_eq!(out, 0);
        }
      }
    }
  }
}
