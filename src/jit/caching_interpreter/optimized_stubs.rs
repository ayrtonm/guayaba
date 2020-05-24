use crate::register::BitTwiddle;
use crate::r3000::MaybeSet;
use crate::jit::caching_interpreter::block::Block;
use crate::jit::insn::Insn;
use crate::jit::caching_interpreter::stub::Stub;
use crate::common::*;

impl Block {
  pub(super) fn create_optimized_stubs(tagged_opcodes: &Vec<Insn>, logging: bool) -> Vec<Stub> {
    let mut ret = Vec::new();
    let mut constant_table: [Option<u32>;32] = [None; 32];
    macro_rules! default_stub {
      ($insn:expr, $n:expr) => {
        {
          $insn.output().map(|output| constant_table[output] = None);
          match $insn.output() {
            Some(output) => {
              //if we're writing to R0 we can skip the stub if it's not a branch delay slot
              if output == 0 {
                //the first stub will never be a branch delay slot
                if $n != 0 {
                  if Insn::has_branch_delay_slot(tagged_opcodes[$n - 1].op()) {
                    ret.push(Stub::from_closure(Box::new(move |vm| None)));
                  }
                }
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
    for (n, insn) in tagged_opcodes.iter().enumerate() {
      constant_table[0] = Some(0);
      let op = insn.op();
      match get_primary_field(op) {
        0x00 => {
          match get_secondary_field(op) {
            _ => {
              default_stub!(insn, n);
            },
          }
        },
        0x09..=0x0E => {
          //ADDIU, SLTI, SLTIU, ANDI, ORI, XORI
          assert!(insn.inputs().len() == 1);
          let input = insn.inputs()[0] as usize;
          let output = insn.output().expect("ADDIU should have an output");
          match constant_table[input] {
            Some(constant) => {
              let imm16 = get_imm16(insn.op());
              let result = match get_primary_field(op) {
                0x09 => {
                  let imm16 = get_imm16(insn.op()).half_sign_extended();
                  constant.wrapping_add(imm16)
                },
                0x0A => {
                  constant.signed_compare(imm16)
                },
                0x0B => {
                  constant.compare(imm16)
                },
                0x0C => {
                  constant.and(imm16)
                },
                0x0D => {
                  constant.or(imm16)
                },
                0x0E => {
                  constant.xor(imm16)
                },
                _ => unreachable!("")
              };
              let t = get_rt(insn.op());
              ret.push(Stub::from_closure(Box::new(move |vm| {
                let rt = vm.r3000.nth_reg_mut(t);
                vm.modified_register = rt.maybe_set(result);
                if logging {
                  println!("> Constant load\nR{} = {:#x}", t, result);
                };
                None
              })));
              constant_table[output] = Some(result);
            },
            None => {
              ret.push(Stub::new(&insn, logging));
              constant_table[output] = None;
            },
          }
        },
        0x0F => {
          //LUI
          let output = insn.output().expect("LUI should have an output");
          constant_table[output] = Some(get_imm16(insn.op()) << 16);
          //FIXME: we can skip this stub if one of the coming instructions will also write to Rn
          //however this requires modifying all closures in Stub::new() to work with constant_table
          //also it would be costly to iterate through future instructions here, so it's probably best
          //to use a map to say we write to Rn later on, remove the stub at position x
          //and remember that x = ret.len(), not n since default stub!() may skip stubs
          ret.push(Stub::new(&insn, logging));
        },
        _ => {
          default_stub!(insn, n);
        },
      }
    };
    ret
  }
}
