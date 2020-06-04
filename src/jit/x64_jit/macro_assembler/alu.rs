use crate::register::BitTwiddle;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;
use crate::jit::x64_jit::register_allocator::X64_RAX;

impl MacroAssembler {
  //FIXME: the general case (i.e. not eax) seems to be wrong here
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
