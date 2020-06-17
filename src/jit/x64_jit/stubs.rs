use jam::recompiler::Recompiler;
use jam::ArgNumber;
use jam::Label;
use crate::register::BitTwiddle;
use crate::r3000::R3000;
use crate::jit::insn::Insn;
use crate::jit::x64_jit::Block;
use crate::common::*;

//TODO: remember to handle R0's explicitly (remove all unwraps in emit_insn)
impl Block {
  pub(super) const R3000_REG_POS: usize = 0;
  pub(super) const COP0_REG_POS: usize = 1;
  pub(super) const CONSOLE_POS: usize = 2;
  pub(super) const WRITE_WORD_POS: usize = 3;
  pub(super) const WRITE_HALF_POS: usize = 4;
  pub(super) const WRITE_BYTE_POS: usize = 5;
  pub(super) const READ_WORD_POS: usize = 6;
  pub(super) const READ_HALF_POS: usize = 7;
  pub(super) const READ_BYTE_POS: usize = 8;
  pub(super) const READ_HALF_SIGN_EXTENDED_POS: usize = 9;
  pub(super) const READ_BYTE_SIGN_EXTENDED_POS: usize = 10;

  pub(super) fn emit_insn(rc: &mut Recompiler, insn: &Insn, initial_pc: u32) -> Option<Label> {
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
            let rd = rc.reg(d);
            match rd {
              Some(rd) => {
                let rt = rc.reg(t).unwrap();
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
            match rc.reg(d) {
              Some(rd) => {
                match (rc.reg(s), rc.reg(t)) {
                  (None, None) => {
                    rc.seti_u32(rd, 0);
                  },
                  (None, Some(rt)) => {
                    rc.setv_u32(rd, rt);
                  },
                  (Some(rs), None) => {
                    rc.setv_u32(rd, rs);
                  },
                  (Some(rs), Some(rt)) => {
                    rc.setv_u32(rd, rs);
                    rc.orv_u32(rd, rt);
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
        let delay_slot = rc.new_label();
        let this_op = rc.new_long_label();
        rc.jump(delay_slot);

        rc.define_label(this_op);
        let jit_pc = rc.reg(R3000::PC_IDX as u32).unwrap();
        rc.seti_u32(jit_pc, dest);
        rc.ret();
        rc.define_label(delay_slot);
        return Some(this_op)
      },
      0x05 => {
        //BNE
        let imm16 = get_imm16(op);
        let inc = ((imm16.half_sign_extended() as i32) << 2) as u32;
        let t = get_rt(op);
        let s = get_rs(op);
        let rt = rc.reg(t);
        let rs = rc.reg(s);
        let delay_slot = rc.new_label();
        let this_op = rc.new_long_label();
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
        rc.jump(delay_slot);
        rc.define_label(this_op);
        rc.ret();
        rc.define_label(delay_slot);
        return Some(this_op)
      },
      0x09 => {
        //ADDIU
        let s = get_rs(op);
        let t = get_rt(op);
        let imm16 = get_imm16(op).half_sign_extended();
        match rc.reg(t) {
          Some(rt) => {
            match rc.reg(s) {
              Some(rs) => {
                rc.setv_u32(rt, rs);
                rc.addi_u32(rt, imm16 as i32);
              },
              None => {
                rc.seti_u32(rt, imm16);
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
        let rs = rc.reg(s).unwrap();
        let rt = rc.reg(t).unwrap();
        rc.setv_u32(rt, rs);
        rc.ori_u32(rt, imm16);
      },
      0x0F => {
        //LUI
        let t = get_rt(op);
        let rt = rc.reg(t).unwrap();
        let imm16 = get_imm16(op);
        let result = imm16 << 16;
        rc.seti_u32(rt, result);
      },
      0x10 => {
        //COP0
        match get_rs(op) {
          0x04 => {
            //MTCn
            let t = get_rt(op);
            let d = get_rd(op);
            let rt = rc.reg(t).unwrap();
            let rd = rc.new_u64();
            rc.load_ptr(rd, Block::COP0_REG_POS);
            rc.index_mut_u32(rd, rt, 0);
          },
          _ => todo!("COP0 {:#x}", get_rs(op)),
        }
      },
      0x2B => {
        //SW
        let s = get_rs(op);
        let t = get_rt(op);
        let imm16 = get_imm16(op);
        let cop0r12 = rc.new_u32();
        rc.load_ptr(cop0r12, Block::COP0_REG_POS);
        rc.deref_u32(cop0r12);
        rc.bti_u32(cop0r12, 16);

        let label = rc.new_label();
        let console = rc.new_u64();
        let address = rc.new_u32();
        let zero = rc.new_u32();
        rc.jump_if_no_carry(label);
        let rs = rc.reg(s).unwrap();
        rc.load_ptr(console, Block::CONSOLE_POS);
        rc.setv_u32(address, rs);
        rc.addi_u32(address, imm16 as i32);
        rc.set_argn(console, ArgNumber::Arg1);
        rc.set_argn(address, ArgNumber::Arg2);
        match rc.reg(t) {
          Some(rt) => {
            rc.set_argn(rt, ArgNumber::Arg3);
          },
          None => {
            rc.seti_u32(zero, 0);
            rc.set_argn(zero, ArgNumber::Arg3);
          },
        }
        rc.call_ptr(Block::WRITE_WORD_POS);
        rc.define_label(label);
      },
      _ => todo!("primary field {:#x}", get_primary_field(op)),
    };
    None
  }
}
