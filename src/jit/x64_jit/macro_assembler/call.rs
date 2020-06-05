use crate::register::BitTwiddle;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;

impl MacroAssembler {
  //TODO: test this
  pub fn emit_callq_r64(&mut self, reg: u32) {
    self.emit_conditional_rexb(reg);
    self.buffer.push(0xff);
    self.buffer.push(0xd0 | reg.lowest_bits(3) as u8);
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
}
