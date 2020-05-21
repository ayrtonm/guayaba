use crate::common::*;

//this tags each opcode with its input and output registers
pub struct Insn {
  opcode: u32,
  //registers which are used directly, i.e. not as an index into memory
  inputs: Option<Vec<u32>>,
  //sometimes an input register is used as an index into memory
  indices: Option<u32>,
  //the modified register if any
  output: Option<u32>,
}

impl Insn {
  pub fn new(opcode: u32) -> Self {
    let inputs = None;
    let indices = None;
    let output = None;
    Insn {
      opcode,
      inputs,
      indices,
      output,
    }
  }
  pub fn op(&self) -> u32 {
    self.opcode
  }
  pub fn is_inside_block(op: u32) -> bool {
    !(Insn::is_syscall(op) || Insn::is_unconditional_jump(op))
  }
  pub fn is_syscall(op: u32) -> bool {
    get_primary_field(op) == 0xc
  }
  pub fn is_unconditional_jump(op: u32) -> bool {
    false
  }
}
