use jam::recompiler::Recompiler;
use crate::jit::insn::Insn;
use crate::common::*;

pub trait GenerateStubs {
  fn emit_insn(&mut self, insn: &Insn);
}

impl GenerateStubs for Recompiler {
  fn emit_insn(&mut self, insn: &Insn) {
    let op = insn.op();
    match get_primary_field(op) {
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
      _ => todo!("{:#x}", get_primary_field(op)),
    }
  }
}
