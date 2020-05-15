use crate::register::Register;
use crate::register::BitBang;
use crate::r3000::MaybeSet;
use crate::dummy_jit::Dummy_JIT;
use crate::console::Console;
use super::insn_ir::Insn;
use super::insn_ir::Kind;
use crate::common::*;

impl Dummy_JIT {
  pub(super) fn compile_stub_as_is(&mut self, operations: &mut Vec<(Register, Insn)>, logging: bool) -> Vec<Box<dyn Fn(&mut Console)>> {
    let mut ret = Vec::new();
    for (op, _) in operations {
      ret.push(self.compile_opcode(*op, logging).expect(""));
    };
    ret
  }
  fn emit_store_constant(ret: &mut Vec<Box<dyn Fn(&mut Console)>>, result: Register, name: Register, logging: bool) {
    ret.push(Box::new(move |vm| {
      let out = vm.r3000.nth_reg_mut(name);
      vm.modified_register = out.maybe_set(result);
      if logging {
        println!("> optimized register load R{} = {:#x}", name, result);
      }
    }));
  }
  pub(super) fn compile_optimized_stub(&mut self, operations: &mut Vec<(Register, Insn)>, logging: bool) -> Vec<Box<dyn Fn(&mut Console)>> {
    let initial_n = operations.len();
    let mut ret = Vec::new();
    let mut const_registers: [Option<Register>; 32] = [None; 32];
    for (op, tag) in operations {
      const_registers[0] = Some(0);
      match tag.inputs_len() {
        0 => {
          match tag.output() {
            Some(output) => {
              match tag.kind() {
                Kind::Immediate => {
                  //LUI
                  assert!(get_primary_field(*op) == 0x0F);
                  const_registers[output as usize] = Some(get_imm16(*op) << 16);
                  ret.push(self.compile_opcode(*op, logging).expect(""));
                },
                _ => {
                  const_registers[output as usize] = None;
                  ret.push(self.compile_opcode(*op, logging).expect(""));
                },
              }
            },
            None => {
              ret.push(self.compile_opcode(*op, logging).expect(""));
            },
          }
        },
        1 => {
          match tag.output() {
            Some(output) => {
              let opcode = get_primary_field(*op);
              match opcode {
                0x09..=0x0E => {
                  match const_registers[tag.input_i(0)] {
                    Some(constant) => {
                      let result = match opcode {
                        0x09 => constant.wrapping_add(get_imm16(*op).half_sign_extended()),
                        0x0A => constant.signed_compare(get_imm16(*op)),
                        0x0B => constant.compare(get_imm16(*op)),
                        0x0C => constant.and(get_imm16(*op)),
                        0x0D => constant.or(get_imm16(*op)),
                        0x0E => constant.xor(get_imm16(*op)),
                        _ => unreachable!(""),
                      };
                      Dummy_JIT::emit_store_constant(&mut ret, result, output, logging);
                      //mark output as constant
                      const_registers[output as usize] = Some(result);
                    },
                    None => {
                      const_registers[output as usize] = None;
                      ret.push(self.compile_opcode(*op, logging).expect(""));
                    },
                  }
                },
                _ => {
                  const_registers[output as usize] = None;
                  ret.push(self.compile_opcode(*op, logging).expect(""));
                },
              }
            },
            None => {
              ret.push(self.compile_opcode(*op, logging).expect(""));
            },
          }
        },
        2 => {
          match tag.output() {
            Some(output) => {
              const_registers[output as usize] = None;
              ret.push(self.compile_opcode(*op, logging).expect(""));
            },
            None => {
              ret.push(self.compile_opcode(*op, logging).expect(""));
            },
          }
        },
        _ => {
          unreachable!("")
        },
      };
    }
    println!("{}/{}", ret.len(), initial_n);
    ret
  }
}
