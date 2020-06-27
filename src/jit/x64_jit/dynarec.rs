use jam::recompiler::Recompiler;
use crate::register::BitTwiddle;
use crate::console::r3000::R3000;
use crate::console::cop0::Cop0Exception;
use crate::jit::insn::Insn;
use crate::jit::x64_jit::Block;
use crate::jit::x64_jit::block::NextOp;
use crate::common::*;

pub trait DynaRec {
  fn emit_insn(&mut self, insn: &Insn, initial_pc: u32) -> NextOp;
  fn emit_load(&mut self, op: u32, function_ptr: usize);
  fn emit_store(&mut self, op: u32, function_ptr: usize);
  fn emit_addi(&mut self, op: u32);
  fn emit_jump_imm26(&mut self, insn: &Insn, initial_pc: u32) -> NextOp;
  fn emit_jump_reg(&mut self, insn: &Insn, initial_pc: u32) -> NextOp;
  fn emit_branch_equal(&mut self, insn: &Insn, initial_pc: u32, invert: bool) -> NextOp;
  fn emit_branch_gtz(&mut self, insn: &Insn, initial_pc: u32, invert: bool) -> NextOp;
  fn emit_branch_gez(&mut self, insn: &Insn, initial_pc: u32, invert: bool) -> NextOp;
}

impl DynaRec for Recompiler {
  fn emit_insn(&mut self, insn: &Insn, initial_pc: u32) -> NextOp {
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
          0x02 => {
            //SRL
            let t = get_rt(op);
            let d = get_rd(op);
            let imm5 = get_imm5(op);
            self.reg(d).map(|rd| {
              match self.reg(t) {
                Some(rt) => {
                  self.setv_u32(rd, rt);
                  self.srli_u32(rd, imm5);
                },
                None => self.seti_u32(rd, 0),
              }
            });
          },
          0x03 => {
            //SRA
            let t = get_rt(op);
            let d = get_rd(op);
            let imm5 = get_imm5(op);
            self.reg(d).map(|rd| {
              match self.reg(t) {
                Some(rt) => {
                  self.setv_u32(rd, rt);
                  self.srai_u32(rd, imm5);
                },
                None => self.seti_u32(rd, 0),
              }
            });
          },
          0x08 => {
            //JR
            return self.emit_jump_reg(insn, initial_pc);
          },
          0x09 => {
            //JALR
            let ret = initial_pc.wrapping_add(offset).wrapping_add(4);
            let ra = self.reg(R3000::RA_IDX as u32).expect("");
            self.seti_u32(ra, ret);
            return self.emit_jump_reg(insn, initial_pc);
          },
          0x0C => {
            //SYSCALL
            let pc = self.reg(R3000::PC_IDX as u32).expect("");
            let jit_pc = self.new_u32();
            let exception = self.new_u32();
            let console = self.new_u64();
            self.load_ptr(console, Block::CONSOLE_POS);
            self.seti_u32(jit_pc, initial_pc.wrapping_add(offset));
            self.seti_u32(exception, Cop0Exception::Syscall as u32);

            self.set_arg1(console);
            self.set_arg2(exception);
            self.set_arg3(jit_pc);
            self.set_ret(pc);

            self.call_ptr(Block::GEN_EXCEPTION);
            return NextOp::Exit
          },
          0x0D => {
            //BREAK
            //TODO: I've never had to execute this in the BIOS
            //so let's put trigger a SIGILL if we ever execute this
            self.illegal_insn();
          },
          0x10 => {
            //MFHI
            let d = get_rd(op);
            let hi = self.reg(R3000::HI_IDX as u32).expect("");
            self.reg(d).map(|rd| {
              self.setv_u32(rd, hi);
            });
          },
          0x11 => {
            //MTHI
            let s = get_rs(op);
            let hi = self.reg(R3000::HI_IDX as u32).expect("");
            match self.reg(s) {
              Some(rs) => self.setv_u32(hi, rs),
              None => self.seti_u32(hi, 0),
            }
          },
          0x12 => {
            //MFLO
            let d = get_rd(op);
            let lo = self.reg(R3000::LO_IDX as u32).expect("");
            self.reg(d).map(|rd| {
              self.setv_u32(rd, lo);
            });
          },
          0x13 => {
            //MTLO
            let s = get_rs(op);
            let lo = self.reg(R3000::LO_IDX as u32).expect("");
            match self.reg(s) {
              Some(rs) => self.setv_u32(lo, rs),
              None => self.seti_u32(lo, 0),
            }
          },
          0x1A => {
            //DIV
            let s = get_rs(op);
            let t = get_rt(op);
            let hi = self.reg(R3000::HI_IDX as u32).expect("");
            let lo = self.reg(R3000::LO_IDX as u32).expect("");
            match (self.reg(s), self.reg(t)) {
              (None, None) => {
                self.seti_u32(hi, 0);
                self.seti_u32(lo, (-1 as i32) as u32);
              },
              (None, Some(rt)) => {
                self.seti_u32(hi, 0);
                self.seti_u32(lo, 0);
              },
              (Some(rs), None) => {
                self.setv_u32(hi, rs);
                todo!("self.seti_u32(lo, 0);");
              },
              //FIXME: double check to see how x86 div handles -0x8000_0000 / -1
              (Some(rs), Some(rt)) => {
                self.divv_u32(rs, rt, lo, hi);
              },
            }
          },
          0x1B => {
            //DIVU
            let s = get_rs(op);
            let t = get_rt(op);
            let hi = self.reg(R3000::HI_IDX as u32).expect("");
            let lo = self.reg(R3000::LO_IDX as u32).expect("");
            match (self.reg(s), self.reg(t)) {
              (None, None) => {
                self.seti_u32(hi, 0);
                self.seti_u32(lo, 0);
              },
              (None, Some(rt)) => {
                self.seti_u32(hi, 0);
                self.seti_u32(lo, 0);
              },
              (Some(rs), None) => {
                self.setv_u32(hi, rs);
                self.seti_u32(lo, 0);
              },
              (Some(rs), Some(rt)) => {
                self.divuv_u32(rs, rt, lo, hi);
              },
            }
          },
          0x20 => {
            //ADD
            //FIXME: implement overflow trap
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
            //return true
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
          0x23 => {
            //SUBU
            let s = get_rs(op);
            let t = get_rt(op);
            let d = get_rd(op);
            self.reg(d).map(|rd| {
              match self.reg(s) {
                Some(rs) => {
                  self.setv_u32(rd, rs);
                },
                None => {
                  self.seti_u32(rd, 0);
                },
              }
              self.reg(t).map(|rt| {
                self.subv_u32(rd, rt);
              });
            });
          },
          0x24 => {
            //AND
            let s = get_rs(op);
            let t = get_rt(op);
            let d = get_rd(op);
            self.reg(d).map(|rd| {
              match (self.reg(s), self.reg(t)) {
                (None, None) => {
                  self.seti_u32(rd, 0);
                },
                (None, Some(rt)) => {
                  self.seti_u32(rd, 0);
                },
                (Some(rs), None) => {
                  self.seti_u32(rd, 0);
                },
                (Some(rs), Some(rt)) => {
                  self.setv_u32(rd, rs);
                  self.andv_u32(rd, rt);
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
          0x2A => {
            //SLT
            let s = get_rs(op);
            let t = get_rt(op);
            let d = get_rd(op);
            self.reg(d).map(|rd| {
              let skip_set = self.new_label();
              let end = self.new_label();
              let zero = self.new_u32();
              self.seti_u32(zero, 0);
              match (self.reg(t), self.reg(s)) {
                (None, None) => {
                  self.jump(skip_set);
                },
                (Some(rt), None) => {
                  self.cmpv_u32(rt, zero);
                  self.jump_if_not_less(skip_set);
                },
                (None, Some(rs)) => {
                  self.cmpv_u32(zero, rs);
                  self.jump_if_not_less(skip_set);
                },
                (Some(rt), Some(rs)) => {
                  self.cmpv_u32(rt, rs);
                  self.jump_if_not_less(skip_set);
                },
              }
              self.seti_u32(rd, 1);
              self.jump(end);
              self.define_label(skip_set);
              self.seti_u32(rd, 0);
              self.define_label(end);
            });
          },
          0x2B => {
            //SLTU
            let s = get_rs(op);
            let t = get_rt(op);
            let d = get_rd(op);
            self.reg(d).map(|rd| {
              let skip_set = self.new_label();
              let end = self.new_label();
              let zero = self.new_u32();
              self.seti_u32(zero, 0);
              match (self.reg(t), self.reg(s)) {
                (None, None) => {
                  self.jump(skip_set);
                },
                (Some(rt), None) => {
                  self.cmpv_u32(rt, zero);
                  self.jump_if_not_carry(skip_set);
                },
                (None, Some(rs)) => {
                  self.cmpv_u32(zero, rs);
                  self.jump_if_not_carry(skip_set);
                },
                (Some(rt), Some(rs)) => {
                  self.cmpv_u32(rt, rs);
                  self.jump_if_not_carry(skip_set);
                },
              }
              self.seti_u32(rd, 1);
              self.jump(end);
              self.define_label(skip_set);
              self.seti_u32(rd, 0);
              self.define_label(end);
            });
          },
          _ => todo!("secondary field {:#x}", get_secondary_field(op)),
        }
      },
      0x01 => {
        //BcondZ
        match get_rt(op) {
          0x00 => {
            //BLTZ
            self.emit_branch_gez(insn, initial_pc, true);
          },
          0x01 => {
            //BGEZ
            self.emit_branch_gez(insn, initial_pc, false);
          },
          //0x80 => {
          //  //BLTZAL
          //  log!("> BLTZAL");
          //  call!(rs < 0)
          //},
          //0x81 => {
          //  //BGEZAL
          //  log!("> BGEZAL");
          //  call!(rs >= 0)
          //},
          _ => {
            //invalid opcode
            unreachable!("BcondZ {:#x}", get_rt(op))
          },
        }
      },
      0x02 => {
        //J
        return self.emit_jump_imm26(insn, initial_pc);
      },
      0x03 => {
        //JAL
        let ret = initial_pc.wrapping_add(offset).wrapping_add(4);
        let ra = self.reg(R3000::RA_IDX as u32).expect("");
        self.seti_u32(ra, ret);
        return self.emit_jump_imm26(insn, initial_pc);
      },
      0x04 => {
        //BEQ
        return self.emit_branch_equal(insn, initial_pc, false);
      },
      0x05 => {
        //BNE
        return self.emit_branch_equal(insn, initial_pc, true);
      },
      0x06 => {
        //BLEZ
        return self.emit_branch_gtz(insn, initial_pc, true);
      },
      0x07 => {
        //BGTZ
        return self.emit_branch_gtz(insn, initial_pc, false);
      },
      0x08 => {
        //ADDI
        self.emit_addi(op);
        //return true
      },
      0x09 => {
        //ADDIU
        self.emit_addi(op);
      },
      0x0A => {
        //SLTI
        let s = get_rs(op);
        let t = get_rt(op);
        let imm16 = get_imm16(op).half_sign_extended();
        self.reg(t).map(|rt| {
          let skip_set = self.new_label();
          let end = self.new_label();
          let zero = self.new_u32();
          let temp = self.new_u32();
          self.seti_u32(zero, 0);
          self.seti_u32(temp, imm16);
          match self.reg(s) {
            Some(rs) => {
              self.cmpv_u32(temp, rs);
            },
            None => {
              self.cmpv_u32(temp, zero);
            },
          }
          self.jump_if_not_less(skip_set);
          self.seti_u32(rt, 1);
          self.jump(end);
          self.define_label(skip_set);
          self.seti_u32(rt, 0);
          self.define_label(end);
        });
      },
      0x0B => {
        //SLTIU
        let s = get_rs(op);
        let t = get_rt(op);
        let imm16 = get_imm16(op).half_sign_extended();
        self.reg(t).map(|rt| {
          let skip_set = self.new_label();
          let end = self.new_label();
          let zero = self.new_u32();
          let temp = self.new_u32();
          self.seti_u32(zero, 0);
          self.seti_u32(temp, imm16);
          match self.reg(s) {
            Some(rs) => {
              self.cmpv_u32(temp, rs);
            },
            None => {
              self.cmpv_u32(temp, zero);
            },
          }
          self.jump_if_not_carry(skip_set);
          self.seti_u32(rt, 1);
          self.jump(end);
          self.define_label(skip_set);
          self.seti_u32(rt, 0);
          self.define_label(end);
        });
      },
      0x0C => {
        //ANDI
        let s = get_rs(op);
        let t = get_rt(op);
        let imm16 = get_imm16(op);
        self.reg(t).map(|rt| {
          match self.reg(s) {
            Some(rs) => {
              self.setv_u32(rt, rs);
              self.andi_u32(rt, imm16);
            },
            None => self.seti_u32(rt, 0),
          }
        });
      },
      0x0D => {
        //ORI
        let s = get_rs(op);
        let t = get_rt(op);
        let imm16 = get_imm16(op);
        self.reg(t).map(|rt| {
          if s == t {
            self.ori_u32(rt, imm16);
          } else {
            self.seti_u32(rt, imm16);
            self.reg(s).map(|rs| {
              self.orv_u32(rt, rs);
            });
          }
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
          0x00 => {
            //MFC0
            let t = get_rt(op);
            let d = get_rd(op);
            if d == 12 || d == 13 || d == 14 {
              self.reg(t).map(|rt| {
                let delayed_write = self.new_delayed_write(rt);
                let cop0_rd = self.new_u64();
                self.load_ptr(cop0_rd, Block::COP0_REG_POS);
                self.index_u32(cop0_rd, (d - 12) as i32);
                self.setv_u32(delayed_write, cop0_rd);
              });
            }
          },
          0x04 => {
            //MTC0
            let t = get_rt(op);
            let d = get_rd(op);
            if d == 12 || d == 13 || d == 14 {
              let zero = self.new_u32();
              self.seti_u32(zero, 0);
              let cop0_rd = self.new_u64();
              self.load_ptr(cop0_rd, Block::COP0_REG_POS);
              match self.reg(t) {
                Some(rt) => {
                  self.index_mut_u32(cop0_rd, rt, (d - 12) as i32);
                },
                None => {
                  self.index_mut_u32(cop0_rd, zero, (d - 12) as i32);
                },
              }
            }
          },
          _ => todo!("COP0 {:#x}", get_rs(op)),
        }
      },
      0x20 => {
        //LB
        self.emit_load(op, Block::READ_BYTE_SIGN_EXTENDED_POS);
      },
      0x21 => {
        //LH
        self.emit_load(op, Block::READ_HALF_SIGN_EXTENDED_POS);
      },
      0x23 => {
        //LW
        self.emit_load(op, Block::READ_WORD_POS);
      },
      0x24 => {
        //LBU
        self.emit_load(op, Block::READ_BYTE_POS);
      },
      0x25 => {
        //LHU
        self.emit_load(op, Block::READ_HALF_POS);
      },
      0x28 => {
        //SB
        self.emit_store(op, Block::WRITE_BYTE_POS);
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
    NextOp::Standard
  }
  fn emit_load(&mut self, op: u32, function_ptr: usize) {
    let t = get_rt(op);
    self.reg(t).map(|rt| {
      let s = get_rs(op);
      let imm16 = get_imm16(op).half_sign_extended();

      let end = self.new_label();
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

      self.call_ptr_with_ret(function_ptr);
      self.define_label(end);
    });
  }
  fn emit_store(&mut self, op: u32, function_ptr: usize) {
    let s = get_rs(op);
    let t = get_rt(op);
    let imm16 = get_imm16(op).half_sign_extended();
    let cop0r12 = self.new_u32();

    let end = self.new_label();
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
    self.jump_if_carry(end);
    self.call_ptr(function_ptr);
    self.define_label(end);
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
  fn emit_jump_imm26(&mut self, insn: &Insn, initial_pc: u32) -> NextOp {
    let op = insn.op();
    let offset = insn.offset();
    let imm26 = get_imm26(op);
    let shifted_imm26 = imm26 << 2;
    let pc = initial_pc.wrapping_add(offset);
    let pc_hi_bits = pc & 0xf000_0000;
    let dest = pc_hi_bits.wrapping_add(shifted_imm26);
    let jit_pc = self.reg(R3000::PC_IDX as u32).expect("");
    self.seti_u32(jit_pc, dest);
    self.set_carry();
    NextOp::DelaySlot
  }
  fn emit_jump_reg(&mut self, insn: &Insn, initial_pc: u32) -> NextOp {
    let op = insn.op();
    let jit_pc = self.reg(R3000::PC_IDX as u32).expect("");
    let s = get_rs(op);
    //FIXME: handle case where rs is misaligned
    match self.reg(s) {
      Some(rs) => {
        self.setv_u32(jit_pc, rs);
      },
      None => {
        self.seti_u32(jit_pc, 0);
      },
    }
    self.set_carry();
    NextOp::DelaySlot
  }
  fn emit_branch_equal(&mut self, insn: &Insn, initial_pc: u32, invert: bool) -> NextOp {
    let op = insn.op();
    let offset = insn.offset();
    let imm16 = get_imm16(op);
    let inc = ((imm16.half_sign_extended() as i32) << 2) as u32;
    let pc = initial_pc.wrapping_add(offset);
    let dest = pc.wrapping_add(inc);
    let t = get_rt(op);
    let s = get_rs(op);
    let take_jump = self.new_label();
    let next_op = self.new_label();
    let jit_pc = self.reg(R3000::PC_IDX as u32).expect("");
    match (self.reg(s), self.reg(t)) {
      (None, None) => self.set_zero(),
      (Some(rs), None) => self.testv_u32(rs, rs),
      (None, Some(rt)) => self.testv_u32(rt, rt),
      (Some(rs), Some(rt)) => self.cmpv_u32(rs, rt),
    }
    if invert {
      self.jump_if_not_zero(take_jump);
    } else {
      self.jump_if_zero(take_jump);
    }
    self.clear_carry();
    self.jump(next_op);

    self.define_label(take_jump);
    self.seti_u32(jit_pc, dest);
    self.set_carry();

    self.define_label(next_op);
    NextOp::DelaySlot
  }
  fn emit_branch_gtz(&mut self, insn: &Insn, initial_pc: u32, invert: bool) -> NextOp {
    let op = insn.op();
    let offset = insn.offset();
    let imm16 = get_imm16(op);
    let inc = ((imm16.half_sign_extended() as i32) << 2) as u32;
    let pc = initial_pc.wrapping_add(offset);
    let dest = pc.wrapping_add(inc);
    let s = get_rs(op);
    let skip_jump = self.new_label();
    let take_jump = self.new_label();
    let next_op = self.new_label();
    let jit_pc = self.reg(R3000::PC_IDX as u32).expect("");
    match self.reg(s) {
      None => self.clear_signed(),
      Some(rs) => self.testv_u32(rs, rs),
    }
    if invert {
      self.jump_if_zero(take_jump);
      self.jump_if_signed(take_jump);
      self.jump(skip_jump);
    } else {
      self.jump_if_zero(skip_jump);
      self.jump_if_signed(skip_jump);
    }
    self.define_label(take_jump);
    self.seti_u32(jit_pc, dest);
    self.set_carry();
    self.jump(next_op);

    self.define_label(skip_jump);
    self.clear_carry();

    self.define_label(next_op);
    NextOp::DelaySlot
  }
  fn emit_branch_gez(&mut self, insn: &Insn, initial_pc: u32, invert: bool) -> NextOp {
    let op = insn.op();
    let offset = insn.offset();
    let imm16 = get_imm16(op);
    let inc = ((imm16.half_sign_extended() as i32) << 2) as u32;
    let pc = initial_pc.wrapping_add(offset);
    let dest = pc.wrapping_add(inc);
    let s = get_rs(op);
    let skip_jump = self.new_label();
    let next_op = self.new_label();
    let jit_pc = self.reg(R3000::PC_IDX as u32).expect("");
    match self.reg(s) {
      None => self.clear_signed(),
      Some(rs) => self.testv_u32(rs, rs),
    }
    //self.bind(jit_pc);
    //if op == 0x4410005 {
    //  self.set_arg1(jit_pc);
    //  self.call_ptr(Block::DEBUG_POS);
    //}
    if invert {
      self.jump_if_not_signed(skip_jump);
    } else {
      self.jump_if_signed(skip_jump);
    }
    self.seti_u32(jit_pc, dest);
    self.set_carry();
    self.jump(next_op);

    self.define_label(skip_jump);
    self.clear_carry();

    self.define_label(next_op);
    //if op == 0x4410005 {
    //  self.set_arg1(jit_pc);
    //  self.call_ptr(Block::DEBUG_POS);
    //}
    NextOp::DelaySlot
  }
}
