use crate::x64_jit::insn::Insn;
use crate::x64_jit::stub::Stub;

pub struct Block {
  //a vec of closures to be executed in order
  stubs: Vec<Stub>,
  //the physical address of the last instruction
  //this will be either the branch delay slot or a syscall
  final_pc: u32,
  //the number of MIPS opcodes represented by this Block
  //may be more than the length of stubs
  nominal_len: u32,
}

impl Block {
  pub fn new(stubs: Vec<Stub>, final_pc: u32, nominal_len: u32) -> Self {
    Block {
      stubs,
      final_pc,
      nominal_len,
    }
  }
  pub fn create_stubs(tagged_opcodes: &Vec<Insn>, logging: bool) -> Vec<Stub> {
    let mut ret = Vec::new();
    for insn in tagged_opcodes {
      ret.push(Stub::new(&insn, logging));
    };
    ret
  }
  pub fn stubs(&self) -> &Vec<Stub> {
    &self.stubs
  }
  pub fn final_pc(&self) -> u32 {
    self.final_pc
  }
  pub fn nominal_len(&self) -> u32 {
    self.nominal_len
  }
}

