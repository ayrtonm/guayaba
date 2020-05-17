use crate::register::Register;
use crate::register::BitBang;
use crate::r3000::MaybeSet;
use crate::r3000::DelayedWrite;
use crate::r3000::Name;
use crate::caching_interpreter::CachingInterpreter;
use crate::console::Console;
use super::insn_ir::Insn;
use super::insn_ir::Kind;
use crate::common::*;

impl CachingInterpreter {
  pub(super) fn compile_stub(&mut self, operations: &mut Vec<(Register, Insn)>, logging: bool) -> Vec<Box<dyn Fn(&mut Console)>> {
    let mut ret = Vec::new();
    for (op, _) in operations {
      ret.push(self.compile_opcode(*op, logging).expect(""));
    };
    ret
  }
  fn emit_optimized_read_byte(ret: &mut Vec<Box<dyn Fn(&mut Console)>>, op: Register, constant: Register, logging: bool) {
    let t = get_rt(op);
    let imm16 = get_imm16(op).half_sign_extended();
    let constant_address = constant.wrapping_add(imm16);
    ret.push(Box::new(move |vm| {
      let result = vm.resolve_memresponse(vm.memory.read_byte(constant_address));
      vm.delayed_writes.push_back(DelayedWrite::new(Name::Rn(t), result));
      if logging {
        println!("R{} = [{:#x} + {:#x}] \n  = [{:#x}] \n  = {:#x} {}",
                  t, constant, imm16, constant_address, result, stringify!(read_byte));
      }
    }));
  }
  fn emit_optimized_read_half(ret: &mut Vec<Box<dyn Fn(&mut Console)>>, op: Register, constant: Register, logging: bool) {
    let t = get_rt(op);
    let imm16 = get_imm16(op).half_sign_extended();
    let constant_address = constant.wrapping_add(imm16);
    ret.push(Box::new(move |vm| {
      let result = vm.resolve_memresponse(vm.memory.read_half(constant_address));
      vm.delayed_writes.push_back(DelayedWrite::new(Name::Rn(t), result));
      if logging {
        println!("R{} = [{:#x} + {:#x}] \n  = [{:#x}] \n  = {:#x} {}",
                  t, constant, imm16, constant_address, result, stringify!(read_byte));
      }
    }));
  }
  fn emit_optimized_read_word(ret: &mut Vec<Box<dyn Fn(&mut Console)>>, op: Register, constant: Register, logging: bool) {
    let t = get_rt(op);
    let imm16 = get_imm16(op).half_sign_extended();
    let constant_address = constant.wrapping_add(imm16);
    ret.push(Box::new(move |vm| {
      let result = vm.resolve_memresponse(vm.memory.read_word(constant_address));
      vm.delayed_writes.push_back(DelayedWrite::new(Name::Rn(t), result));
      if logging {
        println!("R{} = [{:#x} + {:#x}] \n  = [{:#x}] \n  = {:#x} {}",
                  t, constant, imm16, constant_address, result, stringify!(read_byte));
      }
    }));
  }
  fn emit_optimized_write_byte(ret: &mut Vec<Box<dyn Fn(&mut Console)>>, op: Register, constant: Register, logging: bool) {
    let t = get_rt(op);
    let imm16 = get_imm16(op).half_sign_extended();
    let constant_address = constant.wrapping_add(imm16);
    ret.push(Box::new(move |vm| {
      let rt = vm.r3000.nth_reg(t);
      if logging {
        println!("optimized [{:#x} + {:#x}] = [{:#x}] \n  = R{}\n  = {:#x} {}",
                  constant, imm16, constant_address, t, rt, stringify!(write_byte));
      }
      if !vm.cop0.cache_isolated() {
        vm.write_byte(constant_address, rt);
      } else {
        if logging {
          println!("ignoring write while cache is isolated");
        }
      }
    }));
  }
  fn emit_optimized_write_half(ret: &mut Vec<Box<dyn Fn(&mut Console)>>, op: Register, constant: Register, logging: bool) {
    let t = get_rt(op);
    let imm16 = get_imm16(op).half_sign_extended();
    let constant_address = constant.wrapping_add(imm16);
    ret.push(Box::new(move |vm| {
      let rt = vm.r3000.nth_reg(t);
      if logging {
        println!("optimized [{:#x} + {:#x}] = [{:#x}] \n  = R{}\n  = {:#x} {}",
                  constant, imm16, constant_address, t, rt, stringify!(write_half));
      }
      if !vm.cop0.cache_isolated() {
        vm.write_half(constant_address, rt);
      } else {
        if logging {
          println!("ignoring write while cache is isolated");
        }
      }
    }));
  }
  fn emit_optimized_write_word(ret: &mut Vec<Box<dyn Fn(&mut Console)>>, op: Register, constant: Register, logging: bool) {
    let t = get_rt(op);
    let imm16 = get_imm16(op).half_sign_extended();
    let constant_address = constant.wrapping_add(imm16);
    ret.push(Box::new(move |vm| {
      let rt = vm.r3000.nth_reg(t);
      if logging {
        println!("optimized [{:#x} + {:#x}] = [{:#x}] \n  = R{}\n  = {:#x} {}",
                  constant, imm16, constant_address, t, rt, stringify!(write_word));
      }
      if !vm.cop0.cache_isolated() {
        vm.write_word(constant_address, rt);
      } else {
        if logging {
          println!("ignoring write while cache is isolated");
        }
      }
    }));
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
    for (i, (op, tag)) in operations.iter().enumerate() {
      const_registers[0] = Some(0);
      let opcode = get_primary_field(*op);
      match opcode {
        0x00 => {
          let second_opcode = get_secondary_field(*op);
          match second_opcode {
            0x21 | 0x23..=0x27 | 0x2A | 0x2B => {
              match (const_registers[tag.input_i(0)], const_registers[tag.input_i(1)]) {
                (Some(c1), Some(c2)) => {
                  let result = match second_opcode {
                    0x21 => c1.wrapping_add(c2),
                    0x23 => c1.wrapping_sub(c2),
                    0x24 => c1.and(c2),
                    0x25 => c1.or(c2),
                    0x26 => c1.xor(c2),
                    0x27 => c1.nor(c2),
                    0x2A => c1.signed_compare(c2),
                    0x2B => c1.compare(c2),
                    _ => unreachable!("{:x}", get_secondary_field(*op)),
                  };
                  let output = tag.output().expect("");
                  CachingInterpreter::emit_store_constant(&mut ret, result, output, logging);
                  const_registers[output as usize] = Some(result);
                },
                _ => {
                  let output = tag.output().expect("");
                  const_registers[output as usize] = None;
                  ret.push(self.compile_opcode(*op, logging).expect(""));
                },
              }
            },
            _ => {
              match tag.output() {
                Some(output) => const_registers[output as usize] = None,
                None => {},
              }
              ret.push(self.compile_opcode(*op, logging).expect(""));
            },
          }
        },
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
              let output = tag.output().expect("");
              //if previous operation was a LUI with the same output
              if i != 0 {
                if get_primary_field(operations[i - 1].0) == 0x0F &&
                  operations[i - 1].1.output() == Some(output) {
                  ret.pop();
                }
              }
              CachingInterpreter::emit_store_constant(&mut ret, result, output, logging);
              //mark output as constant
              const_registers[output as usize] = Some(result);
            },
            None => {
              let output = tag.output().expect("");
              const_registers[output as usize] = None;
              ret.push(self.compile_opcode(*op, logging).expect(""));
            },
          }
        },
        0x0F => {
          //LUI
          let result = get_imm16(*op) << 16;
          let output = tag.output().expect("");
          const_registers[output as usize] = Some(result);
          CachingInterpreter::emit_store_constant(&mut ret, result, output, logging);
        },
        0x23 => {
          let output = tag.output().expect("");
          const_registers[output as usize] = None;
          match const_registers[tag.input_i(0)] {
            Some(addr) => {
              CachingInterpreter::emit_optimized_read_word(&mut ret, *op, addr, logging);
            },
            None => {
              ret.push(self.compile_opcode(*op, logging).expect(""));
            },
          }
        },
        0x24 => {
          let output = tag.output().expect("");
          const_registers[output as usize] = None;
          match const_registers[tag.input_i(0)] {
            Some(addr) => {
              CachingInterpreter::emit_optimized_read_byte(&mut ret, *op, addr, logging);
            },
            None => {
              ret.push(self.compile_opcode(*op, logging).expect(""));
            },
          }
        },
        0x25 => {
          let output = tag.output().expect("");
          const_registers[output as usize] = None;
          match const_registers[tag.input_i(0)] {
            Some(addr) => {
              CachingInterpreter::emit_optimized_read_half(&mut ret, *op, addr, logging);
            },
            None => {
              ret.push(self.compile_opcode(*op, logging).expect(""));
            },
          }
        },
        0x28 => {
          match (const_registers[tag.input_i(0)], const_registers[tag.input_i(1)]) {
            (Some(addr), _) => {
              CachingInterpreter::emit_optimized_write_byte(&mut ret, *op, addr, logging);
            },
            _ => {
              ret.push(self.compile_opcode(*op, logging).expect(""));
            },
          }
        },
        0x29 => {
          match (const_registers[tag.input_i(0)], const_registers[tag.input_i(1)]) {
            (Some(addr), _) => {
              CachingInterpreter::emit_optimized_write_half(&mut ret, *op, addr, logging);
            },
            _ => {
              ret.push(self.compile_opcode(*op, logging).expect(""));
            },
          }
        },
        0x2B => {
          match (const_registers[tag.input_i(0)], const_registers[tag.input_i(1)]) {
            (Some(addr), _) => {
              CachingInterpreter::emit_optimized_write_word(&mut ret, *op, addr, logging);
            },
            _ => {
              ret.push(self.compile_opcode(*op, logging).expect(""));
            },
          }
        },
        _ => {
          match tag.output() {
            Some(output) => const_registers[output as usize] = None,
            None => {},
          }
          ret.push(self.compile_opcode(*op, logging).expect(""));
        },
      }
    }
    ret
  }
}
