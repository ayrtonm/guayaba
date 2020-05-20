use crate::register::BitTwiddle;

#[derive(Debug)]
pub enum Cop0Exception {
  Interrupt,
  LoadAddress,
  StoreAddress,
  Syscall,
  Overflow,
}

#[derive(Default)]
pub struct Cop0 {
  r12: u32,
  r13: u32,
  r14: u32,
}

impl Cop0 {
  pub fn nth_data_reg(&self, idx: u32) -> u32 {
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
        //println!("tried reading from commonly unused COP0 data register R{}", idx);
        0
      },
    }
  }
  pub fn nth_data_reg_mut(&mut self, idx: u32) -> Option<&mut u32> {
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
        //println!("tried writing to commonly unused COP0 data register R{}", idx);
        None
      },
    }
  }
  pub fn nth_ctrl_reg(&self, idx: u32) -> u32 {
    println!("tried reading from commonly unused COP0 control register R{}", idx);
    0
  }
  pub fn nth_ctrl_reg_mut(&mut self, idx: u32) -> Option<&mut u32> {
    println!("tried writing to commonly unused COP0 control register R{}", idx);
    None
  }
  pub fn bcnf(&self, _: u32) -> Option<u32> {
    //this is technically an illegal instruction since COP0 does not implement it
    None
  }
  pub fn request_interrupt(&mut self, irq: u32) {
    //FIXME: double check what else needs to be done
    //there should be something that specifies which interrupt was requested right?
    self.r13.set(10);
  }
  pub fn generate_exception(&mut self, kind: Cop0Exception, current_pc: u32) -> u32 {
    self.store_pc(current_pc);
    let cause = match kind {
      Cop0Exception::Interrupt => 0x00,
      Cop0Exception::LoadAddress => 0x04,
      Cop0Exception::StoreAddress => 0x05,
      Cop0Exception::Syscall => 0x08,
      Cop0Exception::Overflow => 0x0C,
    };
    self.set_exception_cause(cause);
    self.disable_interrupts();
    self.exception_vector()
  }
  pub fn cache_isolated(&self) -> bool {
    self.r12.nth_bit_bool(16)
  }
  pub fn execute_command(&mut self, imm25: u32) -> Option<u32> {
    //this is the only legal COP0 command
    if imm25 == 0x0000_0010 {
      let bits2_3 = (self.r12 & 0x0000_000c) >> 2;
      let bits4_5 = (self.r12 & 0x0000_0030) >> 2;
      self.r12.clear_mask(0x0f).set_mask(bits2_3).set_mask(bits4_5);
    }
    None
  }
  fn store_pc(&mut self, current_pc: u32) {
    self.r14 = current_pc;
  }
  fn set_exception_cause(&mut self, cause: u32) {
    assert!(cause < 0x20);
    self.r13.clear(2).clear(3).clear(4).clear(5).clear(6).set_mask(cause << 2);
  }
  fn exception_vector(&self) -> u32 {
    if self.r12.nth_bit_bool(22) {
      0xbfc00180
    } else {
      0x80000080
    }
  }
  fn disable_interrupts(&mut self) {
    let prev = self.r12.nth_bit(0);
    self.r12.clear(0).clear(2).set_mask(prev << 2);
  }
}

