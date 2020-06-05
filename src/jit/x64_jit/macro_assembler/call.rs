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
