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
  //TODO: test this
  pub fn emit_xchgq_rm(&mut self, reg: u32, ptr: u32) {
  }
  //TODO: test this
  pub fn emit_xchgq_rm_offset(&mut self, reg: u32, ptr: u32, offset: i32) {
  }
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
}
