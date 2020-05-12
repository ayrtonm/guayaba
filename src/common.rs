use crate::register::Register;
use crate::register::BitBang;

pub trait ReadArray {
  fn read_byte(&self, address: Register) -> Register;
  fn read_half(&self, address: Register) -> Register;
  fn read_word(&self, address: Register) -> Register;
}

pub trait WriteArray {
  fn write_byte(&mut self, address: Register, value: Register);
  fn write_half(&mut self, address: Register, value: Register);
  fn write_word(&mut self, address: Register, value: Register);
}

impl ReadArray for &[u8] {
  fn read_byte(&self, address: Register) -> Register {
    let address = address as usize;
    self[address] as Register
  }
  fn read_half(&self, address: Register) -> Register {
    let address = address as usize;
    self[address] as Register |
    (self[address + 1] as Register) << 8
  }
  fn read_word(&self, address: Register) -> Register {
    let address = address as usize;
    self[address] as Register |
    (self[address + 1] as Register) << 8 |
    (self[address + 2] as Register) << 16 |
    (self[address + 3] as Register) << 24
  }
}

impl WriteArray for &mut [u8] {
  fn write_byte(&mut self, address: Register, value: Register) {
    let address = address as usize;
    self[address] = value.byte() as u8;
  }
  fn write_half(&mut self, address: Register, value: Register) {
    let address = address as usize;
    self[address] = value.byte() as u8;
    self[address + 1] = value.upper_bits(24).byte() as u8;
  }
  fn write_word(&mut self, address: Register, value: Register) {
    let address = address as usize;
    self[address] = value.byte() as u8;
    self[address + 1] = value.upper_bits(24).byte() as u8;
    self[address + 2] = value.upper_bits(16).byte() as u8;
    self[address + 3] = value.upper_bits(8).byte() as u8;
  }
}

pub fn get_rs(op: u32) -> u32 {
  (op & 0x03e0_0000) >> 21
}
pub fn get_rt(op: u32) -> u32 {
  (op & 0x001f_0000) >> 16
}
pub fn get_rd(op: u32) -> u32 {
  (op & 0x0000_f800) >> 11
}
pub fn get_imm5(op: u32) -> u32 {
  (op & 0x0000_07c0) >> 6
}
pub fn get_imm16(op: u32) -> u32 {
  op & 0x0000_ffff
}
pub fn get_imm25(op: u32) -> u32 {
  op & 0x01ff_ffff
}
pub fn get_imm26(op: u32) -> u32 {
  op & 0x03ff_ffff
}
pub fn get_primary_field(op: u32) -> u32 {
  (op & 0xfc00_0000) >> 26
}
pub fn get_secondary_field(op: u32) -> u32 {
  op & 0x0000_003f
}
