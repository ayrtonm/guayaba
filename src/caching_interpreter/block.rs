use crate::caching_interpreter::insn::Insn;
use crate::caching_interpreter::stub::Stub;
use crate::common::*;

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
  pub fn create_optimized_stubs(tagged_opcodes: &Vec<Insn>, logging: bool) -> Vec<Stub> {
    let mut ret = Vec::new();
    let mut constant_table = [None; 32];
    for insn in tagged_opcodes {
      constant_table[0] = Some(0);
      let op = insn.op();
      match get_primary_field(op) {
        0x0F => {
          //LUI
          let output = insn.output().expect("LUI should have an output");
          constant_table[output] = Some(get_imm16(insn.op()) << 16);
          ret.push(Stub::new(&insn, logging));
        },
        _ => {
          insn.output().map(|output| constant_table[output] = None);
          match insn.output() {
            Some(output) => {
              if output == 0 {
                ret.push(Stub::from_closure(Box::new(move |vm| None)));
              } else {
                ret.push(Stub::new(&insn, logging));
              }
            },
            None => {
              ret.push(Stub::new(&insn, logging));
            },
          }
        },
      }
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

