use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::register_allocator::X64_RAX;

impl MacroAssembler {
  const PLACEHOLDER: u8 = 0xff;
  pub fn emit_jmp_rel8(&mut self, offset: i8) {
    self.buffer.push(0xeb);
    self.buffer.push(offset as u8);
  }
  pub fn emit_jmp_label(&mut self, label: usize) {
    self.buffer.push(0xeb);
    let placeholder_location = self.buffer.len();
    self.buffer.push(MacroAssembler::PLACEHOLDER);
    self.labels_used.insert(label, placeholder_location);
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
      asm!("callq *%rbp"
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
        asm!("callq *%rbp"
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
}
