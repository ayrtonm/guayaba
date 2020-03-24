use std::ops;

#[derive(Debug,Default)]
pub struct Register {
  value: u32,
}

impl Register {
  pub const fn new(value: u32) -> Self {
    Register {
      value
    }
  }
  pub fn get_value(&self) -> u32 {
    self.value
  }
  pub fn word(self) -> Register {
    self
  }
  pub fn lower_half(&self) -> Register {
    self & 0x0000_ffff
  }
  pub fn lowest_byte(&self) -> Register {
    self & 0x0000_00ff
  }
}

//overloading + operator for Register
impl ops::Add<Register> for Register {
  type Output = Register;
  fn add(self, rhs: Register) -> Register {
    Register::new(self.value.wrapping_add(rhs.value))
  }
}
impl ops::Add<Register> for u32 {
  type Output = Register;
  fn add(self, rhs: Register) -> Register {
    Register::new(self.wrapping_add(rhs.value))
  }
}
impl ops::Add<u32> for Register {
  type Output = Register;
  fn add(self, rhs: u32) -> Register {
    Register::new(self.value.wrapping_add(rhs))
  }
}
//overloading + operator for &Register
impl ops::Add<&Register> for &Register {
  type Output = Register;
  fn add(self, rhs: &Register) -> Register {
    Register::new(self.value.wrapping_add(rhs.value))
  }
}
impl ops::Add<&Register> for u32 {
  type Output = Register;
  fn add(self, rhs: &Register) -> Register {
    Register::new(self.wrapping_add(rhs.value))
  }
}
impl ops::Add<u32> for &Register {
  type Output = Register;
  fn add(self, rhs: u32) -> Register {
    Register::new(self.value.wrapping_add(rhs))
  }
}
//overloading + operator for &mut Register
impl ops::Add<&mut Register> for &mut Register {
  type Output = Register;
  fn add(self, rhs: &mut Register) -> Register {
    Register::new(self.value.wrapping_add(rhs.value))
  }
}
impl ops::Add<&mut Register> for u32 {
  type Output = Register;
  fn add(self, rhs: &mut Register) -> Register {
    Register::new(self.wrapping_add(rhs.value))
  }
}
impl ops::Add<u32> for &mut Register {
  type Output = Register;
  fn add(self, rhs: u32) -> Register {
    Register::new(self.value.wrapping_add(rhs))
  }
}
//overloading += operator for Register
impl ops::AddAssign<Register> for Register {
  fn add_assign(&mut self, rhs: Register) {
    *self = Register::new(self.value.wrapping_add(rhs.value));
  }
}
impl ops::AddAssign<Register> for u32 {
  fn add_assign(&mut self, rhs: Register) {
    *self = self.wrapping_add(rhs.value);
  }
}
impl ops::AddAssign<u32> for Register {
  fn add_assign(&mut self, rhs: u32) {
    *self = Register::new(self.value.wrapping_add(rhs));
  }
}
//overloading += operator for &Register
//it doesn't make sense to implement the two commented impl because the lhs is
//an immutable reference
//impl ops::AddAssign<&Register> for &Register {
//}
impl ops::AddAssign<&Register> for u32 {
  fn add_assign(&mut self, rhs: &Register) {
    *self = self.wrapping_add(rhs.value);
  }
}
//impl ops::AddAssign<u32> for &Register {
//}
//overloading += operator for &mut Register
impl ops::AddAssign<&mut Register> for &mut Register {
  fn add_assign(&mut self, rhs: &mut Register) {
    **self = Register::new(self.value.wrapping_add(rhs.value));
  }
}
impl ops::AddAssign<&mut Register> for u32 {
  fn add_assign(&mut self, rhs: &mut Register) {
    *self = self.wrapping_add(rhs.value);
  }
}
impl ops::AddAssign<u32> for &mut Register {
  fn add_assign(&mut self, rhs: u32) {
    **self = Register::new(self.value.wrapping_add(rhs));
  }
}
//overloading & operator for Register
impl ops::BitAnd<Register> for Register {
  type Output = Register;
  fn bitand(self, rhs: Register) -> Register {
    Register::new(self.value & rhs.value)
  }
}
impl ops::BitAnd<Register> for u32 {
  type Output = Register;
  fn bitand(self, rhs: Register) -> Register {
    Register::new(self & rhs.value)
  }
}
impl ops::BitAnd<u32> for Register {
  type Output = Register;
  fn bitand(self, rhs: u32) -> Register {
    Register::new(self.value & rhs)
  }
}
//overloading & operator for &Register
impl ops::BitAnd<&Register> for &Register {
  type Output = Register;
  fn bitand(self, rhs: &Register) -> Register {
    Register::new(self.value & rhs.value)
  }
}
impl ops::BitAnd<&Register> for u32 {
  type Output = Register;
  fn bitand(self, rhs: &Register) -> Register {
    Register::new(self & rhs.value)
  }
}
impl ops::BitAnd<u32> for &Register {
  type Output = Register;
  fn bitand(self, rhs: u32) -> Register {
    Register::new(self.value & rhs)
  }
}
//overloading & operator for &mut Register
impl ops::BitAnd<&mut Register> for &mut Register {
  type Output = Register;
  fn bitand(self, rhs: &mut Register) -> Register {
    Register::new(self.value & rhs.value)
  }
}
impl ops::BitAnd<&mut Register> for u32 {
  type Output = Register;
  fn bitand(self, rhs: &mut Register) -> Register {
    Register::new(self & rhs.value)
  }
}
impl ops::BitAnd<u32> for &mut Register {
  type Output = Register;
  fn bitand(self, rhs: u32) -> Register {
    Register::new(self.value & rhs)
  }
}
//overloading &= operator for Register
impl ops::BitAndAssign<Register> for Register {
  fn bitand_assign(&mut self, rhs: Register) {
    *self = Register::new(self.value & rhs.value);
  }
}
impl ops::BitAndAssign<Register> for u32 {
  fn bitand_assign(&mut self, rhs: Register) {
    *self = *self & rhs.value;
  }
}
impl ops::BitAndAssign<u32> for Register {
  fn bitand_assign(&mut self, rhs: u32) {
    *self = Register::new(self.value & rhs);
  }
}
//overloading &= operator for &Register
//it doesn't make sense to implement the two commented impl because the lhs is
//an immutable reference
//impl ops::BitAndAssign<&Register> for &Register {
//}
impl ops::BitAndAssign<&Register> for u32 {
  fn bitand_assign(&mut self, rhs: &Register) {
    *self = *self & rhs.value;
  }
}
//impl ops::BitAndAssign<u32> for &Register {
//}
//overloading &= operator for &mut Register
impl ops::BitAndAssign<&mut Register> for &mut Register {
  fn bitand_assign(&mut self, rhs: &mut Register) {
    **self = Register::new(self.value & rhs.value);
  }
}
impl ops::BitAndAssign<&mut Register> for u32 {
  fn bitand_assign(&mut self, rhs: &mut Register) {
    *self = *self & rhs.value;
  }
}
impl ops::BitAndAssign<u32> for &mut Register {
  fn bitand_assign(&mut self, rhs: u32) {
    **self = Register::new(self.value & rhs);
  }
}
//overloading | operator for Register
impl ops::BitOr<Register> for Register {
  type Output = Register;
  fn bitor(self, rhs: Register) -> Register {
    Register::new(self.value | rhs.value)
  }
}
impl ops::BitOr<Register> for u32 {
  type Output = Register;
  fn bitor(self, rhs: Register) -> Register {
    Register::new(self | rhs.value)
  }
}
impl ops::BitOr<u32> for Register {
  type Output = Register;
  fn bitor(self, rhs: u32) -> Register {
    Register::new(self.value | rhs)
  }
}
//overloading | operator for &Register
impl ops::BitOr<&Register> for &Register {
  type Output = Register;
  fn bitor(self, rhs: &Register) -> Register {
    Register::new(self.value | rhs.value)
  }
}
impl ops::BitOr<&Register> for u32 {
  type Output = Register;
  fn bitor(self, rhs: &Register) -> Register {
    Register::new(self | rhs.value)
  }
}
impl ops::BitOr<u32> for &Register {
  type Output = Register;
  fn bitor(self, rhs: u32) -> Register {
    Register::new(self.value | rhs)
  }
}
//overloading | operator for &mut Register
impl ops::BitOr<&mut Register> for &mut Register {
  type Output = Register;
  fn bitor(self, rhs: &mut Register) -> Register {
    Register::new(self.value | rhs.value)
  }
}
impl ops::BitOr<&mut Register> for u32 {
  type Output = Register;
  fn bitor(self, rhs: &mut Register) -> Register {
    Register::new(self | rhs.value)
  }
}
impl ops::BitOr<u32> for &mut Register {
  type Output = Register;
  fn bitor(self, rhs: u32) -> Register {
    Register::new(self.value | rhs)
  }
}
//overloading |= operator for Register
impl ops::BitOrAssign<Register> for Register {
  fn bitor_assign(&mut self, rhs: Register) {
    *self = Register::new(self.value | rhs.value);
  }
}
impl ops::BitOrAssign<Register> for u32 {
  fn bitor_assign(&mut self, rhs: Register) {
    *self = *self | rhs.value;
  }
}
impl ops::BitOrAssign<u32> for Register {
  fn bitor_assign(&mut self, rhs: u32) {
    *self = Register::new(self.value | rhs);
  }
}
//overloading |= operator for &Register
//it doesn't make sense to implement the two commented impl because the lhs is
//an immutable reference
//impl ops::BitOrAssign<&Register> for &Register {
//}
impl ops::BitOrAssign<&Register> for u32 {
  fn bitor_assign(&mut self, rhs: &Register) {
    *self = *self | rhs.value;
  }
}
//impl ops::BitOrAssign<u32> for &Register {
//}
//overloading |= operator for &mut Register
impl ops::BitOrAssign<&mut Register> for &mut Register {
  fn bitor_assign(&mut self, rhs: &mut Register) {
    **self = Register::new(self.value | rhs.value);
  }
}
impl ops::BitOrAssign<&mut Register> for u32 {
  fn bitor_assign(&mut self, rhs: &mut Register) {
    *self = *self | rhs.value;
  }
}
impl ops::BitOrAssign<u32> for &mut Register {
  fn bitor_assign(&mut self, rhs: u32) {
    **self = Register::new(self.value | rhs);
  }
}
//overloading ^ operator for Register
impl ops::BitXor<Register> for Register {
  type Output = Register;
  fn bitxor(self, rhs: Register) -> Register {
    Register::new(self.value ^ rhs.value)
  }
}
impl ops::BitXor<Register> for u32 {
  type Output = Register;
  fn bitxor(self, rhs: Register) -> Register {
    Register::new(self ^ rhs.value)
  }
}
impl ops::BitXor<u32> for Register {
  type Output = Register;
  fn bitxor(self, rhs: u32) -> Register {
    Register::new(self.value ^ rhs)
  }
}
//overloading ^ operator for &Register
impl ops::BitXor<&Register> for &Register {
  type Output = Register;
  fn bitxor(self, rhs: &Register) -> Register {
    Register::new(self.value ^ rhs.value)
  }
}
impl ops::BitXor<&Register> for u32 {
  type Output = Register;
  fn bitxor(self, rhs: &Register) -> Register {
    Register::new(self ^ rhs.value)
  }
}
impl ops::BitXor<u32> for &Register {
  type Output = Register;
  fn bitxor(self, rhs: u32) -> Register {
    Register::new(self.value ^ rhs)
  }
}
//overloading ^ operator for &mut Register
impl ops::BitXor<&mut Register> for &mut Register {
  type Output = Register;
  fn bitxor(self, rhs: &mut Register) -> Register {
    Register::new(self.value ^ rhs.value)
  }
}
impl ops::BitXor<&mut Register> for u32 {
  type Output = Register;
  fn bitxor(self, rhs: &mut Register) -> Register {
    Register::new(self ^ rhs.value)
  }
}
impl ops::BitXor<u32> for &mut Register {
  type Output = Register;
  fn bitxor(self, rhs: u32) -> Register {
    Register::new(self.value ^ rhs)
  }
}
//overloading ^= operator for Register
impl ops::BitXorAssign<Register> for Register {
  fn bitxor_assign(&mut self, rhs: Register) {
    *self = Register::new(self.value ^ rhs.value);
  }
}
impl ops::BitXorAssign<Register> for u32 {
  fn bitxor_assign(&mut self, rhs: Register) {
    *self = *self ^ rhs.value;
  }
}
impl ops::BitXorAssign<u32> for Register {
  fn bitxor_assign(&mut self, rhs: u32) {
    *self = Register::new(self.value ^ rhs);
  }
}
//overloading ^= operator for &Register
//it doesn't make sense to implement the two commented impl because the lhs is
//an immutable reference
//impl ops::BitXorAssign<&Register> for &Register {
//}
impl ops::BitXorAssign<&Register> for u32 {
  fn bitxor_assign(&mut self, rhs: &Register) {
    *self = *self ^ rhs.value;
  }
}
//impl ops::BitXorAssign<u32> for &Register {
//}
//overloading ^= operator for &mut Register
impl ops::BitXorAssign<&mut Register> for &mut Register {
  fn bitxor_assign(&mut self, rhs: &mut Register) {
    **self = Register::new(self.value ^ rhs.value);
  }
}
impl ops::BitXorAssign<&mut Register> for u32 {
  fn bitxor_assign(&mut self, rhs: &mut Register) {
    *self = *self ^ rhs.value;
  }
}
impl ops::BitXorAssign<u32> for &mut Register {
  fn bitxor_assign(&mut self, rhs: u32) {
    **self = Register::new(self.value ^ rhs);
  }
}
//overloading - operator for Register
impl ops::Sub<Register> for Register {
  type Output = Register;
  fn sub(self, rhs: Register) -> Register {
    Register::new(self.value.wrapping_sub(rhs.value))
  }
}
impl ops::Sub<Register> for u32 {
  type Output = Register;
  fn sub(self, rhs: Register) -> Register {
    Register::new(self.wrapping_sub(rhs.value))
  }
}
impl ops::Sub<u32> for Register {
  type Output = Register;
  fn sub(self, rhs: u32) -> Register {
    Register::new(self.value.wrapping_sub(rhs))
  }
}
//overloading - operator for &Register
impl ops::Sub<&Register> for &Register {
  type Output = Register;
  fn sub(self, rhs: &Register) -> Register {
    Register::new(self.value.wrapping_sub(rhs.value))
  }
}
impl ops::Sub<&Register> for u32 {
  type Output = Register;
  fn sub(self, rhs: &Register) -> Register {
    Register::new(self.wrapping_sub(rhs.value))
  }
}
impl ops::Sub<u32> for &Register {
  type Output = Register;
  fn sub(self, rhs: u32) -> Register {
    Register::new(self.value.wrapping_sub(rhs))
  }
}
//overloading - operator for &mut Register
impl ops::Sub<&mut Register> for &mut Register {
  type Output = Register;
  fn sub(self, rhs: &mut Register) -> Register {
    Register::new(self.value.wrapping_sub(rhs.value))
  }
}
impl ops::Sub<&mut Register> for u32 {
  type Output = Register;
  fn sub(self, rhs: &mut Register) -> Register {
    Register::new(self.wrapping_sub(rhs.value))
  }
}
impl ops::Sub<u32> for &mut Register {
  type Output = Register;
  fn sub(self, rhs: u32) -> Register {
    Register::new(self.value.wrapping_sub(rhs))
  }
}
//overloading -= operator for Register
impl ops::SubAssign<Register> for Register {
  fn sub_assign(&mut self, rhs: Register) {
    *self = Register::new(self.value.wrapping_sub(rhs.value));
  }
}
impl ops::SubAssign<Register> for u32 {
  fn sub_assign(&mut self, rhs: Register) {
    *self = self.wrapping_sub(rhs.value);
  }
}
impl ops::SubAssign<u32> for Register {
  fn sub_assign(&mut self, rhs: u32) {
    *self = Register::new(self.value.wrapping_sub(rhs));
  }
}
//overloading -= operator for &Register
//it doesn't make sense to implement the two commented impl because the lhs is
//an immutable reference
//impl ops::SubAssign<&Register> for &Register {
//}
impl ops::SubAssign<&Register> for u32 {
  fn sub_assign(&mut self, rhs: &Register) {
    *self = self.wrapping_sub(rhs.value);
  }
}
//impl ops::SubAssign<u32> for &Register {
//}
//overloading -= operator for &mut Register
impl ops::SubAssign<&mut Register> for &mut Register {
  fn sub_assign(&mut self, rhs: &mut Register) {
    **self = Register::new(self.value.wrapping_sub(rhs.value));
  }
}
impl ops::SubAssign<&mut Register> for u32 {
  fn sub_assign(&mut self, rhs: &mut Register) {
    *self = self.wrapping_sub(rhs.value);
  }
}
impl ops::SubAssign<u32> for &mut Register {
  fn sub_assign(&mut self, rhs: u32) {
    **self = Register::new(self.value.wrapping_sub(rhs));
  }
}





#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn wrap_around() {
    let x = Register::new(2);
    let y = Register::new(0xffff_ffff - 2);
    let w = Register::new(0xffff_ffff - 1);
    let z = Register::new(0xffff_ffff);
    let res1 = &x + &y;
    let res2 = &x + &w;
    let res3 = &x + &z;
    assert_eq!(res1.get_value(), 0xffff_ffff);
    assert_eq!(res2.get_value(), 0);
    assert_eq!(res3.get_value(), 1);
  }
}
