pub type Register = u32;

pub trait Parts {
  fn word(&self) -> u32;
  fn half(&self) -> u32;
  fn byte(&self) -> u32;
  fn half_sign_extended(&self) -> u32;
  fn byte_sign_extended(&self) -> u32;
}
impl Parts for Register{
  fn word(&self) -> u32 {
    *self
  }
  fn half(&self) -> u32 {
    self & 0x0000_ffff
  }
  fn byte(&self) -> u32 {
    self & 0x0000_00ff
  }
  fn half_sign_extended(&self) -> u32 {
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
  fn byte_sign_extended(&self) -> u32 {
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
}
