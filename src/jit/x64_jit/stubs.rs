use jam::recompiler::Recompiler;
use jam::ArgNumber;
use jam::Label;
use crate::register::BitTwiddle;
use crate::console::r3000::R3000;
use crate::jit::insn::Insn;
use crate::jit::x64_jit::Block;
use crate::common::*;

pub trait DynaRec {
  fn emit_insn(&mut self, insn: &Insn, initial_pc: u32) -> Option<Label>;
}
//TODO: remember to handle R0's explicitly (remove all unwraps in emit_insn)
impl DynaRec for Recompiler {
  fn emit_insn(&mut self, insn: &Insn, initial_pc: u32) -> Option<Label> {
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
            let rd = self.reg(d);
            match rd {
              Some(rd) => {
                let rt = self.reg(t).unwrap();
                todo!("");
              },
              None => (),
            }
          },
          0x25 => {
            //OR
            let s = get_rs(op);
            let t = get_rt(op);
            let d = get_rd(op);
            match self.reg(d) {
              Some(rd) => {
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
              },
              None => (),
            }
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
        let delay_slot = self.new_label();
        let this_op = self.new_long_label();
        self.jump(delay_slot);

        self.define_label(this_op);
        let jit_pc = self.reg(R3000::PC_IDX as u32).unwrap();
        self.seti_u32(jit_pc, dest);
        self.ret();
        self.define_label(delay_slot);
        return Some(this_op)
      },
      0x05 => {
        //BNE
        let imm16 = get_imm16(op);
        let inc = ((imm16.half_sign_extended() as i32) << 2) as u32;
        let t = get_rt(op);
        let s = get_rs(op);
        let rt = self.reg(t);
        let rs = self.reg(s);
        let delay_slot = self.new_label();
        let this_op = self.new_long_label();
        match (rs, rt) {
          (None, None) => {
          },
          (Some(rs), None) => {
          },
          (None, Some(rt)) => {
          },
          (Some(rs), Some(rt)) => {
          },
        }
        self.jump(delay_slot);
        self.define_label(this_op);
        self.ret();
        self.define_label(delay_slot);
        return Some(this_op)
      },
      0x09 => {
        //ADDIU
        let s = get_rs(op);
        let t = get_rt(op);
        let imm16 = get_imm16(op).half_sign_extended();
        match self.reg(t) {
          Some(rt) => {
            match self.reg(s) {
              Some(rs) => {
                self.setv_u32(rt, rs);
                self.addi_u32(rt, imm16 as i32);
              },
              None => {
                self.seti_u32(rt, imm16);
              },
            }
          },
          None => (),
        }
      },
      0x0D => {
        //ORI
        let s = get_rs(op);
        let t = get_rt(op);
        let imm16 = get_imm16(op);
        let rs = self.reg(s).unwrap();
        let rt = self.reg(t).unwrap();
        self.setv_u32(rt, rs);
        self.ori_u32(rt, imm16);
      },
      0x0F => {
        //LUI
        let t = get_rt(op);
        let rt = self.reg(t).unwrap();
        let imm16 = get_imm16(op);
        let result = imm16 << 16;
        self.seti_u32(rt, result);
      },
      0x10 => {
        //COP0
        match get_rs(op) {
          0x04 => {
            //MTCn
            let t = get_rt(op);
            let d = get_rd(op);
            let rt = self.reg(t).unwrap();
            let rd = self.new_u64();
            self.load_ptr(rd, Block::COP0_REG_POS);
            self.index_mut_u32(rd, rt, 0);
          },
          _ => todo!("COP0 {:#x}", get_rs(op)),
        }
      },
      0x2B => {
        //SW
        let s = get_rs(op);
        let t = get_rt(op);
        let imm16 = get_imm16(op);
        let cop0r12 = self.new_u32();
        self.load_ptr(cop0r12, Block::COP0_REG_POS);
        self.deref_u32(cop0r12);
        self.bti_u32(cop0r12, 16);

        let label = self.new_label();
        let console = self.new_u64();
        let address = self.new_u32();
        let zero = self.new_u32();
        self.jump_if_no_carry(label);
        let rs = self.reg(s).unwrap();
        self.load_ptr(console, Block::CONSOLE_POS);
        self.setv_u32(address, rs);
        self.addi_u32(address, imm16 as i32);
        self.set_argn(console, ArgNumber::Arg1);
        self.set_argn(address, ArgNumber::Arg2);
        match self.reg(t) {
          Some(rt) => {
            self.set_argn(rt, ArgNumber::Arg3);
          },
          None => {
            self.seti_u32(zero, 0);
            self.set_argn(zero, ArgNumber::Arg3);
          },
        }
        self.call_ptr(Block::WRITE_WORD_POS);
        self.define_label(label);
      },
      _ => todo!("primary field {:#x}", get_primary_field(op)),
    };
    None
  }
}
