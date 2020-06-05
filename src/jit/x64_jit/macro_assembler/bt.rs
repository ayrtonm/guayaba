use crate::register::BitTwiddle;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;

impl MacroAssembler {
  //TODO: test this
  pub fn emit_btl_ir(&mut self, imm: u32, reg: u32) {
    assert!(imm < 32);
    self.emit_conditional_rexb(reg);
    self.buffer.push(0x0f);
    self.buffer.push(0xba);
    self.buffer.push(0xe0 | reg.lowest_bits(3) as u8);
    self.buffer.push(imm as u8);
  }
}
