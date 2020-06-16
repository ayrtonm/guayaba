use jam::recompiler::Recompiler;
use jam::ArgNumber;
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

  pub(super) fn emit_insn(rc: &mut Recompiler, insn: &Insn) {
    let op = insn.op();
    match get_primary_field(op) {
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
      0x2B => {
        //SW
        let s = get_rs(op);
        let t = get_rt(op);
        let imm16 = get_imm16(op);
        let rs = rc.reg(s).unwrap();
        let rt = rc.reg(t).unwrap();
        let cop0r12 = rc.new_u32();
        rc.load_ptr(cop0r12, Block::COP0_REG_POS);
        rc.deref_u32(cop0r12);
        rc.bti_u32(cop0r12, 16);

        let label = rc.new_label();
        let console = rc.new_u64();
        let address = rc.new_u32();
        rc.jump_if_no_carry(label);
        rc.load_ptr(console, Block::CONSOLE_POS);
        rc.setv_u32(address, rs);
        rc.addi_u32(address, imm16 as i32);
        rc.set_argn(console, ArgNumber::Arg1);
        rc.set_argn(address, ArgNumber::Arg2);
        rc.set_argn(rt, ArgNumber::Arg3);
        rc.call_ptr(Block::WRITE_WORD_POS);
        rc.define_label(label);
      },
      _ => todo!("{:#x}", get_primary_field(op)),
    }
  }
}
