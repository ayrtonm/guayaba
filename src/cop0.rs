use crate::register::Register;

#[derive(Debug)]
pub enum Cop0Exception {
  Overflow,
}

#[derive(Default)]
pub struct Cop0 {
  r12: Register,
  r13: Register,
  r14: Register,
}

impl Cop0 {
  pub fn nth_data_reg(&self, idx: u32) -> Register {
    match idx {
      12 => {
        self.r12
      },
      13 => {
        self.r13
      },
      14 => {
        self.r14
      },
      _ => {
        println!("tried reading from commonly unused COP0 data register R{}", idx);
        0
      },
    }
  }
  pub fn nth_data_reg_mut(&mut self, idx: u32) -> Option<&mut Register> {
    match idx {
      12 => {
        Some(&mut self.r12)
      },
      13 => {
        Some(&mut self.r13)
      },
      14 => {
        Some(&mut self.r14)
      },
      _ => {
        println!("tried writing to commonly unused COP0 data register R{}", idx);
        None
      },
    }
  }
  pub fn nth_ctrl_reg(&self, idx: u32) -> Register {
    println!("tried reading from commonly unused COP0 control register R{}", idx);
    0
  }
  pub fn nth_ctrl_reg_mut(&mut self, idx: u32) -> Option<&mut Register> {
    println!("tried writing to commonly unused COP0 control register R{}", idx);
    None
  }
  pub fn bcnf(&self, _: u32) -> Option<Register> {
    //this is technically an illegal instruction since COP0 does not implement it
    None
  }
  pub fn exception(&mut self, kind: Cop0Exception) {
    println!("generated an {:?} exception", kind);
  }
  pub fn cache_isolated(&self) -> bool {
    self.r12 & 0x10000 != 0
  }
  pub fn execute_command(&mut self, imm25: u32) -> Option<Register> {
    //this is the only legal COP0 command
    if imm25 == 0x0000_0010 {
      let bits2_3 = (self.r12 & 0x0000_000c) >> 2;
      let bits4_5 = (self.r12 & 0x0000_0030) >> 2;
      self.r12 &= 0xffff_fff0;
      self.r12 |= bits2_3 | bits4_5;
    }
    None
  }
}

