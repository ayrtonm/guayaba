use std::ops::*;

pub type Register = u32;

pub trait MaybeSet {
  fn maybe_set(self, value: Register);
}
impl MaybeSet for Option<&mut Register> {
  fn maybe_set(self, value: Register) {
    self.map(|reg| *reg = value);
  }
}

pub trait Parts {
  fn word(&self) -> Register;
  fn half(&self) -> Register;
  fn byte(&self) -> Register;
  fn half_sign_extended(&self) -> Register;
  fn byte_sign_extended(&self) -> Register;
  fn sra(&self, rhs: Register) -> Register;
  fn and(&self, rhs: Register) -> Register;
  fn or(&self, rhs: Register) -> Register;
  fn xor(&self, rhs: Register) -> Register;
  fn nor(&self, rhs: Register) -> Register;
  fn compare(&self, rhs: Register) -> Register;
  fn signed_compare(&self, rhs: Register) -> Register;
}
impl Parts for Register{
  fn word(&self) -> Register {
    *self
  }
  fn half(&self) -> Register {
    self & 0x0000_ffff
  }
  fn byte(&self) -> Register {
    self & 0x0000_00ff
  }
  fn half_sign_extended(&self) -> Register {
    let ret = self.half();
    match ret >> 15 {
      0 => {
        ret
      },
      1 => {
        ret | 0xffff_0000
      },
      _ => {
        unreachable!("")
      },
    }
  }
  fn byte_sign_extended(&self) -> Register {
    let ret = self.byte();
    match ret >> 7 {
      0 => {
        ret
      },
      1 => {
        ret | 0xffff_ff00
      },
      _ => {
        unreachable!("")
      },
    }
  }
  fn sra(&self, rhs: Register) -> Register {
    match self & 0x8000_0000 {
      0 => {
        self.shr(rhs)
      },
      0x8000_0000 => {
        todo!("sra")
      },
      _ => {
        unreachable!("sra")
      },
    }
  }
  fn and(&self, rhs: Register) -> Register {
    self.bitand(rhs)
  }
  fn or(&self, rhs: Register) -> Register {
    self.bitor(rhs)
  }
  fn xor(&self, rhs: Register) -> Register {
    self.bitxor(rhs)
  }
  fn nor(&self, rhs: Register) -> Register {
    self.bitor(rhs).not()
  }
  fn compare(&self, rhs: Register) -> Register {
    if *self < rhs {
      1
    } else {
      0
    }
  }
  //FIXME: fix signed comparison
  fn signed_compare(&self, rhs: Register) -> Register {
    if *self < rhs {
      1
    } else {
      0
    }
  }
}
