use jam::recompiler::Recompiler;
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
          0x21 => {
            //ADDU
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
                    self.addv_u32(rd, rt);
                  },
                }
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
          0x2B => {
            //SLTU
            let s = get_rs(op);
            let t = get_rt(op);
            let d = get_rd(op);
            let zero = self.new_u32();
            let set_rd = self.new_label();
            let end = self.new_label();
            self.seti_u32(zero, 0);
            match self.reg(d) {
              Some(rd) => {
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
                    self.cmpv_u32(rs, rt);
                    self.jump_if_carry(set_rd);
                    self.seti_u32(rd, 0);
                    self.jump(end);
                    self.define_label(set_rd);
                    self.seti_u32(rd, 1);
                    self.define_label(end);
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
        let jit_pc = self.reg(R3000::PC_IDX as u32).unwrap();
        self.seti_u32(jit_pc, dest);
        self.set_carry();
        return Some(this_op)
      },
      0x05 => {
        //BNE
        let imm16 = get_imm16(op);
        let inc = ((imm16.half_sign_extended() as i32) << 2) as u32;
        let pc = initial_pc.wrapping_add(offset);
        let dest = pc.wrapping_add(inc);
        let t = get_rt(op);
        let s = get_rs(op);
        let delay_slot = self.new_label();
        let this_op = self.new_long_label();
        let took_jump = self.new_label();
        let end_jump = self.new_label();
        match (s, t) {
          (0, 0) => {
            self.set_zero();
          },
          (s, 0) => {
            let rs = self.reg(s).unwrap();
            self.testv_u32(rs, rs);
          },
          (0, t) => {
            let rt = self.reg(t).unwrap();
            self.testv_u32(rt, rt);
          },
          (s, t) => {
            let rs = self.reg(s).unwrap();
            let rt = self.reg(t).unwrap();
            self.cmpv_u32(rs, rt);
          },
        }
        self.jump_if_zero(took_jump);
        self.clear_carry();
        self.jump(end_jump);
        self.define_label(took_jump);
        let jit_pc = self.reg(R3000::PC_IDX as u32).unwrap();
        self.seti_u32(jit_pc, dest);
        self.set_carry();
        self.define_label(end_jump);
        return Some(this_op)
      },
      0x08 => {
        //ADDI
        let s = get_rs(op);
        let t = get_rt(op);
        let imm16 = get_imm16(op).half_sign_extended();
        let delay_slot = self.new_label();
        let this_op = self.new_long_label();
        self.clear_carry();
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
        //self.jump(delay_slot);

        //self.define_label(this_op);
        //self.clear_carry();
        //self.ret();

        //self.define_label(delay_slot);
        //return Some(this_op)
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
            let zero = self.new_u32();
            self.seti_u32(zero, 0);
            let rd = self.new_u64();
            self.load_ptr(rd, Block::COP0_REG_POS);
            match self.reg(t) {
              Some(rt) => {
                self.index_mut_u32(rd, rt, 0);
              },
              None => {
                self.index_mut_u32(rd, zero, 0);
              },
            }
          },
          _ => todo!("COP0 {:#x}", get_rs(op)),
        }
      },
      0x23 => {
        //LW
        println!("implement LW {:#x}",op);
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
        self.save_flags();

        let label = self.new_label();
        let console = self.new_u64();
        let address = self.new_u32();
        let zero = self.new_u32();
        self.seti_u32(zero, 0);
        self.load_ptr(console, Block::CONSOLE_POS);
        match self.reg(s) {
          Some(rs) => {
            self.setv_u32(address, rs);
          },
          None => {
            self.setv_u32(address, zero);
          },
        }
        self.addi_u32(address, imm16 as i32);

        self.seti_u32(zero, 0);

        self.set_arg1(console);
        self.set_arg2(address);
        match self.reg(t) {
          Some(rt) => {
            self.set_arg3(rt);
          },
          None => {
            self.set_arg3(zero);
          },
        }
        self.load_flags();
        self.jump_if_not_carry(label);
        self.call_ptr(Block::WRITE_WORD_POS);
        self.define_label(label);
      },
      _ => todo!("primary field {:#x}", get_primary_field(op)),
    };
    None
  }
}
