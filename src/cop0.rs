use crate::register::Register;

pub struct Cop0 {
  registers: [Register; 32],
}

impl Cop0 {
  pub fn new() -> Self {
    let registers = Default::default();
    Cop0 {
      registers,
    }
  }
  pub fn nth_reg(&self, idx: u32) -> Register {
    assert!(idx < 32);
    let idx = idx as usize;
    self.registers[idx]
  }
  pub fn nth_reg_mut(&mut self, idx: u32) -> &mut Register {
    assert!(idx < 32);
    let idx = idx as usize;
    &mut self.registers[idx]
  }
}
