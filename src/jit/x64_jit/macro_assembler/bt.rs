use crate::register::BitTwiddle;
use crate::jit::x64_jit::macro_assembler::MacroAssembler;

impl MacroAssembler {
  //TODO: do more thorough testing than the one in jump.rs
  pub fn emit_btl_ir(&mut self, imm: u32, reg: u32) {
    assert!(imm < 32);
    self.emit_conditional_rexb(reg);
    self.buffer.push(0x0f);
    self.buffer.push(0xba);
    self.buffer.push(0xe0 | reg.lowest_bits(3) as u8);
    self.buffer.push(imm as u8);
  }
  //TODO: test this
  pub fn emit_btl_im(&mut self, imm: u32, ptr: u32) {
    assert!(imm < 32);
    self.emit_conditional_rexb(ptr);
    self.buffer.push(0x0f);
    self.buffer.push(0xba);
    self.buffer.push(0x20 | ptr.lowest_bits(3) as u8);
    if ptr.lowest_bits(3) == 5 {
      self.buffer.push(0x24);
    }
    self.buffer.push(imm as u8);
  }
  //TODO: test this
  pub fn emit_btl_im_offset(&mut self, imm: u32, ptr: u32, offset: i32) {
    if offset == 0 {
      self.emit_btl_im(imm, ptr);
    } else {
      assert!(imm < 32);
      self.emit_conditional_rexb(ptr);
      self.buffer.push(0x0f);
      self.buffer.push(0xba);
      match offset {
        0 => unreachable!(""),
        -128..=127 => {
          self.buffer.push(0x60 | ptr.lowest_bits(3) as u8);
          if ptr.lowest_bits(3) == 4 {
            self.buffer.push(0x24);
          };
          self.buffer.push(offset as u8);
        },
        _ => {
          self.buffer.push(0xa0 | ptr.lowest_bits(3) as u8);
          if ptr.lowest_bits(3) == 4 {
            self.buffer.push(0x24);
          };
          self.emit_imm32(offset as u32);
        },
      }
    }
  }
}
