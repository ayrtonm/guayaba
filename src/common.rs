use crate::register::Register;

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
    self[address] = (value & 0x0000_00ff) as u8;
  }
  fn write_half(&mut self, address: Register, value: Register) {
    let address = address as usize;
    self[address] = (value & 0x0000_00ff) as u8;
    self[address + 1] = ((value >> 8) & 0x0000_00ff) as u8;
  }
  fn write_word(&mut self, address: Register, value: Register) {
    let address = address as usize;
    self[address] = (value & 0x0000_00ff) as u8;
    self[address + 1] = ((value >> 8) & 0x0000_00ff) as u8;
    self[address + 2] = ((value >> 16) & 0x0000_00ff) as u8;
    self[address + 3] = ((value >> 24) & 0x0000_00ff) as u8;
  }
}
