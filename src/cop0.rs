use crate::register::Register;

#[derive(Default)]
pub struct Cop0 {
  registers: [Register; 32],
}

impl Cop0 {
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
  pub fn execute_command(&mut self, imm25: u32) -> Option<Register> {
    None
  }
}
