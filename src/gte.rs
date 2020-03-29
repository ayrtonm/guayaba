use crate::register::Register;

#[derive(Default)]
pub struct GTE {
  data_registers: [Register; 32],
  ctrl_registers: [Register; 32],
}

impl GTE {
  pub fn nth_data_reg(&self, idx: u32) -> Register {
    assert!(idx < 32);
    let idx = idx as usize;
    self.data_registers[idx]
  }
  pub fn nth_data_reg_mut(&mut self, idx: u32) -> Option<&mut Register> {
    assert!(idx < 32);
    let idx = idx as usize;
    Some(&mut self.data_registers[idx])
  }
  pub fn nth_ctrl_reg(&self, idx: u32) -> Register {
    assert!(idx < 32);
    let idx = idx as usize;
    self.ctrl_registers[idx]
  }
  pub fn nth_ctrl_reg_mut(&mut self, idx: u32) -> Option<&mut Register> {
    assert!(idx < 32);
    let idx = idx as usize;
    Some(&mut self.ctrl_registers[idx])
  }
  pub fn bcnf(&self, imm16: u32) -> Option<Register> {
    Some(imm16)
  }
  pub fn execute_command(&mut self, imm25: u32) -> Option<Register> {
    None
  }
}
