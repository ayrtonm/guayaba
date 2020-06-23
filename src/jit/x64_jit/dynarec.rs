use jam::recompiler::Recompiler;
use jam::Label;
use crate::register::BitTwiddle;
use crate::console::r3000::R3000;
use crate::jit::insn::Insn;
use crate::jit::x64_jit::Block;
use crate::common::*;

pub trait DynaRec {
  fn emit_insn(&mut self, insn: &Insn, initial_pc: u32) -> bool;
  fn emit_load(&mut self, op: u32, function_ptr: usize);
  fn emit_store(&mut self, op: u32, function_ptr: usize);
  fn emit_addi(&mut self, op: u32);
}

impl DynaRec for Recompiler {
  fn emit_insn(&mut self, insn: &Insn, initial_pc: u32) -> bool {
    let op = insn.op();
    let offset = insn.offset();
    match get_primary_field(op) {
      0x00 => {
        //SPECIAL
        match get_secondary_field(op) {
          0x00 => {
            //SLL
            let t = get_rt(op);
            let d = get_rd(op);
            let imm5 = get_imm5(op);
            self.reg(d).map(|rd| {
              match self.reg(t) {
                Some(rt) => {
                  self.setv_u32(rd, rt);
                  self.slli_u32(rd, imm5);
                },
                None => self.seti_u32(rd, 0),
              }
            });
          },
          0x21 => {
            //ADDU
            let s = get_rs(op);
            let t = get_rt(op);
            let d = get_rd(op);
            self.reg(d).map(|rd| {
              match (self.reg(s), self.reg(t)) {
                (None, None) => {
                  self.seti_u32(rd, 0);
                },
                (None, Some(rt)) => {
                  self.setv_u32(rd, rt);
                },
                (Some(rs), None) => {
                  self.setv_u32(rd, rs);
                },
                (Some(rs), Some(rt)) => {
                  self.setv_u32(rd, rs);
                  self.addv_u32(rd, rt);
                },
              }
            });
          },
          0x25 => {
            //OR
            let s = get_rs(op);
            let t = get_rt(op);
            let d = get_rd(op);
            self.reg(d).map(|rd| {
              match (self.reg(s), self.reg(t)) {
                (None, None) => {
                  self.seti_u32(rd, 0);
                },
                (None, Some(rt)) => {
                  self.setv_u32(rd, rt);
                },
                (Some(rs), None) => {
                  self.setv_u32(rd, rs);
                },
                (Some(rs), Some(rt)) => {
                  self.setv_u32(rd, rs);
                  self.orv_u32(rd, rt);
                },
              }
            });
          },
          0x2B => {
            //SLTU
            let s = get_rs(op);
            let t = get_rt(op);
            let d = get_rd(op);
            let zero = self.new_u32();
            let set_rd = self.new_label();
            let end = self.new_label();
            self.seti_u32(zero, 0);
            self.reg(d).map(|rd| {
              match (self.reg(s), self.reg(t)) {
                (None, None) => {
                  self.seti_u32(rd, 0);
                },
                (None, Some(rt)) => {
                  self.cmpv_u32(zero, rt);
                  self.jump_if_carry(set_rd);
                  self.seti_u32(rd, 0);
                  self.jump(end);
                  self.define_label(set_rd);
                  self.seti_u32(rd, 1);
                  self.define_label(end);
                },
                (Some(rs), None) => {
                  self.seti_u32(rd, 0);
                },
                (Some(rs), Some(rt)) => {
                  self.cmpv_u32(rt, rs);
                  self.jump_if_carry(set_rd);
                  self.seti_u32(rd, 0);
                  self.jump(end);
                  self.define_label(set_rd);
                  self.seti_u32(rd, 1);
                  self.define_label(end);
                },
              }
            });
          },
          _ => todo!("secondary field {:#x}", get_secondary_field(op)),
        }
      },
      0x02 => {
        //J
        let imm26 = get_imm26(op);
        let shifted_imm26 = imm26 << 2;
        let pc = initial_pc.wrapping_add(offset);
        let pc_hi_bits = pc & 0xf000_0000;
        let dest = pc_hi_bits.wrapping_add(shifted_imm26);
        let jit_pc = self.reg(R3000::PC_IDX as u32).expect("");
        self.seti_u32(jit_pc, dest);
        self.set_carry();
        return true
      },
      0x05 => {
        //BNE
        let imm16 = get_imm16(op);
        let inc = ((imm16.half_sign_extended() as i32) << 2) as u32;
        let pc = initial_pc.wrapping_add(offset);
        let dest = pc.wrapping_add(inc);
        let t = get_rt(op);
        let s = get_rs(op);
        let took_jump = self.new_label();
        let next_op = self.new_label();
        match (self.reg(s), self.reg(t)) {
          (None, None) => self.set_zero(),
          (Some(rs), None) => self.testv_u32(rs, rs),
          (None, Some(rt)) => self.testv_u32(rt, rt),
          (Some(rs), Some(rt)) => self.cmpv_u32(rs, rt),
        }
        self.jump_if_not_zero(took_jump);
        self.clear_carry();
        self.jump(next_op);

        self.define_label(took_jump);
        let jit_pc = self.reg(R3000::PC_IDX as u32).expect("");
        self.seti_u32(jit_pc, dest);
        self.set_carry();

        self.define_label(next_op);
        return true
      },
      0x08 => {
        //ADDI
        self.emit_addi(op);
        return true
      },
      0x09 => {
        //ADDIU
        self.emit_addi(op);
      },
      0x0D => {
        //ORI
        let s = get_rs(op);
        let t = get_rt(op);
        let imm16 = get_imm16(op);
        self.reg(t).map(|rt| {
          self.seti_u32(rt, imm16);
          self.reg(s).map(|rs| {
            self.orv_u32(rt, rs);
          });
        });
      },
      0x0F => {
        //LUI
        let t = get_rt(op);
        self.reg(t).map(|rt| {
          let imm16 = get_imm16(op);
          let result = imm16 << 16;
          self.seti_u32(rt, result);
        });
      },
      0x10 => {
        //COP0
        match get_rs(op) {
          0x04 => {
            //MTCn
            let t = get_rt(op);
            let d = get_rd(op);
            let zero = self.new_u32();
            self.seti_u32(zero, 0);
            let cop0_rd = self.new_u64();
            self.load_ptr(cop0_rd, Block::COP0_REG_POS);
            match self.reg(t) {
              Some(rt) => {
                self.index_mut_u32(cop0_rd, rt, 0);
              },
              None => {
                self.index_mut_u32(cop0_rd, zero, 0);
              },
            }
          },
          _ => todo!("COP0 {:#x}", get_rs(op)),
        }
      },
      0x23 => {
        //LW
        self.emit_load(op, Block::READ_WORD_POS);
      },
      0x29 => {
        //SH
        self.emit_store(op, Block::WRITE_HALF_POS);
      },
      0x2B => {
        //SW
        self.emit_store(op, Block::WRITE_WORD_POS);
      },
      _ => todo!("primary field {:#x}", get_primary_field(op)),
    };
    false
  }
  fn emit_load(&mut self, op: u32, function_ptr: usize) {
    let t = get_rt(op);
    match self.reg(t) {
      Some(rt) => {
        let s = get_rs(op);
        let imm16 = get_imm16(op);
        let cop0r12 = self.new_u32();

        let label = self.new_label();
        let console = self.new_u64();
        let address = self.new_u32();

        self.load_ptr(console, Block::CONSOLE_POS);
        match self.reg(s) {
          Some(rs) => {
            self.setv_u32(address, rs);
          },
          None => {
            self.seti_u32(address, 0);
          },
        }
        self.addi_u32(address, imm16 as i32);

        self.set_arg1(console);
        self.set_arg2(address);
        let delayed_write = self.new_delayed_write(rt);
        self.set_ret(delayed_write);
        self.load_ptr(cop0r12, Block::COP0_REG_POS);
        self.deref_u32(cop0r12);
        self.bti_u32(cop0r12, 16);
        self.jump_if_not_carry(label);
        self.call_ptr(function_ptr);
        self.define_label(label);
      },
      None => (),
    }
  }
  fn emit_store(&mut self, op: u32, function_ptr: usize) {
    let s = get_rs(op);
    let t = get_rt(op);
    let imm16 = get_imm16(op);
    let cop0r12 = self.new_u32();

    let label = self.new_label();
    let console = self.new_u64();
    let address = self.new_u32();

    self.load_ptr(console, Block::CONSOLE_POS);
    match self.reg(s) {
      Some(rs) => {
        self.setv_u32(address, rs);
      },
      None => {
        self.seti_u32(address, 0);
      },
    }
    self.addi_u32(address, imm16 as i32);

    self.set_arg1(console);
    self.set_arg2(address);
    match self.reg(t) {
      Some(rt) => {
        self.set_arg3(rt);
      },
      None => {
        self.zero_arg3();
      },
    }
    self.load_ptr(cop0r12, Block::COP0_REG_POS);
    self.deref_u32(cop0r12);
    self.bti_u32(cop0r12, 16);
    self.jump_if_not_carry(label);
    self.call_ptr(function_ptr);
    self.define_label(label);
  }
  fn emit_addi(&mut self, op: u32) {
    let s = get_rs(op);
    let t = get_rt(op);
    let imm16 = get_imm16(op).half_sign_extended();
    self.clear_carry();
    self.reg(t).map(|rt| {
      if t == s {
        self.addi_u32(rt, imm16 as i32);
      } else {
        self.seti_u32(rt, imm16);
        self.reg(s).map(|rs| {
          self.addv_u32(rt, rs);
        });
      }
    });
  }
}
