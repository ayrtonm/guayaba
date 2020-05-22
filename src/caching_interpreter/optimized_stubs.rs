use crate::caching_interpreter::block::Block;
use crate::caching_interpreter::insn::Insn;
use crate::caching_interpreter::stub::Stub;
use crate::common::*;

impl Block {
  pub fn create_optimized_stubs(tagged_opcodes: &Vec<Insn>, logging: bool) -> Vec<Stub> {
    let mut ret = Vec::new();
    let mut constant_table = [None; 32];
    macro_rules! default_stub {
      ($insn:expr) => {
        {
          $insn.output().map(|output| constant_table[output] = None);
          match $insn.output() {
            Some(output) => {
              if output == 0 {
                ret.push(Stub::from_closure(Box::new(move |vm| None)));
              } else {
                ret.push(Stub::new(&$insn, logging));
              }
            },
            None => {
              ret.push(Stub::new(&$insn, logging));
            },
          }
        }
      };
    }
    for insn in tagged_opcodes {
      constant_table[0] = Some(0);
      let op = insn.op();
      match get_primary_field(op) {
        0x00 => {
          match get_secondary_field(op) {
            _ => {
              default_stub!(insn);
            },
          }
        },
        0x0F => {
          //LUI
          let output = insn.output().expect("LUI should have an output");
          constant_table[output] = Some(get_imm16(insn.op()) << 16);
          ret.push(Stub::new(&insn, logging));
        },
        _ => {
          default_stub!(insn);
        },
      }
    };
    ret
  }
}
