use crate::register::Register;

#[derive(Debug)]
pub enum Cop0Exception {
  Interrupt,
  Syscall,
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
  pub fn generate_exception(&mut self, kind: Cop0Exception, current_pc: Register) -> Register {
    println!("generated a {:?} exception", kind);
    self.store_pc(current_pc);
    let cause = match kind {
      Cop0Exception::Interrupt => {
        0x00
      },
      Cop0Exception::Syscall => {
        0x08
      },
      Cop0Exception::Overflow => {
        0x0C
      },
    };
    self.set_exception_cause(cause);
    self.disable_interrupts();
    self.exception_vector()
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
  fn store_pc(&mut self, current_pc: Register) {
    self.r14 = current_pc;
  }
  fn set_exception_cause(&mut self, cause: u32) {
    assert!(cause < 0x20);
    self.r13 = (self.r13 & 0xffff_ff83) | (cause << 2);
  }
  fn exception_vector(&self) -> Register {
    match self.r12 & 0x0040_0000 {
      0 => {
        0x80000080
      },
      0x0040_0000 => {
        0xbfc00180
      },
      _ => {
        unreachable!("");
      },
    }
  }
  fn disable_interrupts(&mut self) {
    let prev = self.r12 & 1;
    self.r12 = (self.r12 & 0xffff_fffa) | (prev << 2);
  }
}

