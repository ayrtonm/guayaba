use std::ops::Add;
use std::ops::Shl;
use std::ops::Shr;
use std::ops::Sub;
use crate::register::BitBang;
use crate::r3000::MaybeSet;
use crate::r3000::DelayedWrite;
use crate::r3000::Name;
use crate::cop0::Cop0Exception;
use crate::dummy_jit::Dummy_JIT;
use crate::console::Console;
use crate::common::*;

enum Kind {
  Immediate,
  Register,
  Jump,
}

pub struct Insn {
  kind: Kind,
  inputs: Vec<u32>,
  output: Option<u32>,
}

impl Dummy_JIT {
  //outer option determines if it's an insn that ends a block
  //inner option determines if there's if the insn is tagged
  pub(super) fn tag_insn(&mut self, op: u32, logging: bool) -> Option<Option<Insn>> {
    macro_rules! log {
      () => ($crate::print!("\n"));
      ($($arg:tt)*) => ({
        if logging {
          println!($($arg)*);
        };
      })
    }
    match get_primary_field(op) {
      0x00 => {
        //SPECIAL
        match get_secondary_field(op) {
          0x00 => {
            //SLL
            log!("> SLL");
            //compute!(rd = rt shl imm5)
            Some(Some(Insn {
              kind: Kind::Immediate,
              inputs: vec![get_rt(op)],
              output: Some(get_rd(op)),
            }))
          },
          0x02 => {
            //SRL
            log!("> SRL");
            //compute!(rd = rt shr imm5)
            Some(Some(Insn {
              kind: Kind::Immediate,
              inputs: vec![get_rt(op)],
              output: Some(get_rd(op)),
            }))
          },
          0x03 => {
            //SRA
            log!("> SRA");
            //compute!(rd = rt sra imm5)
            Some(Some(Insn {
              kind: Kind::Immediate,
              inputs: vec![get_rt(op)],
              output: Some(get_rd(op)),
            }))
          },
          0x04 => {
            //SLLV
            log!("> SLLV");
            //compute!(rd = rt shl (rs and 0x1F))
            Some(Some(Insn {
              kind: Kind::Immediate,
              inputs: vec![get_rt(op), get_rs(op)],
              output: Some(get_rd(op)),
            }))
          },
          0x06 => {
            //SRLV
            log!("> SRLV");
            //compute!(rd = rt shr (rs and 0x1F))
            Some(Some(Insn {
              kind: Kind::Immediate,
              inputs: vec![get_rt(op), get_rs(op)],
              output: Some(get_rd(op)),
            }))
          },
          0x07 => {
            //SRAV
            log!("> SRAV");
            //compute!(rd = rt sra (rs and 0x1F))
            Some(Some(Insn {
              kind: Kind::Immediate,
              inputs: vec![get_rt(op), get_rs(op)],
              output: Some(get_rd(op)),
            }))
          },
          0x08 => {
            //JR
            /*jump!(rs);*/
            None
          },
          0x09 => {
            //JALR
            /*call!(rs);*/
            None
          },
          0x0C => {
            //SYSCALL
            //Some(Box::new(move |vm| {
            //  let pc = vm.r3000.pc_mut();
            //  *pc = vm.cop0.generate_exception(Cop0Exception::Syscall, *pc);
            //}))
            None
          },
          0x0D => {
            //BREAK
            log!("> BREAK");
            todo!("break")
          },
          0x10 => {
            //MFHI
            log!("> MFHI");
            //mov!(rd = hi)
            Some(None)
          },
          0x11 => {
            //MTHI
            log!("> MTHI");
            //mov!(hi = rs)
            Some(None)
          },
          0x12 => {
            //MFLO
            log!("> MFLO");
            //mov!(rd = lo)
            Some(None)
          },
          0x13 => {
            //MTLO
            log!("> MTLO");
            //mov!(lo = rs)
            Some(None)
          },
          0x18 => {
            //MULT
            log!("> MULT");
            //compute!(hi:lo = rs * rt signed)
            Some(None)
          },
          0x19 => {
            //MULTU
            log!("> MULTU");
            //compute!(hi:lo = rs * rt)
            Some(None)
          },
          0x1A => {
            //DIV
            log!("> DIV");
            //compute!(hi:lo = rs / rt signed)
            Some(None)
          },
          0x1B => {
            //DIVU
            log!("> DIVU");
            //compute!(hi:lo = rs / rt)
            Some(None)
          },
          0x20 => {
            //ADD
            log!("> ADD");
            //compute!(rd = rs checked_add rt trap)
            Some(Some(Insn {
              kind: Kind::Register,
              inputs: vec![get_rs(op), get_rt(op)],
              output: Some(get_rd(op)),
            }))
          },
          0x21 => {
            //ADDU
            log!("> ADDU");
            //compute!(rd = rs wrapping_add rt)
            Some(Some(Insn {
              kind: Kind::Register,
              inputs: vec![get_rs(op), get_rt(op)],
              output: Some(get_rd(op)),
            }))
          },
          0x22 => {
            //SUB
            log!("> SUB");
            //compute!(rd = rs checked_sub rt trap)
            Some(Some(Insn {
              kind: Kind::Register,
              inputs: vec![get_rs(op), get_rt(op)],
              output: Some(get_rd(op)),
            }))
          },
          0x23 => {
            //SUBU
            log!("> SUBU");
            //compute!(rd = rs wrapping_sub rt)
            Some(Some(Insn {
              kind: Kind::Register,
              inputs: vec![get_rs(op), get_rt(op)],
              output: Some(get_rd(op)),
            }))
          },
          0x24 => {
            //AND
            log!("> AND");
            //compute!(rd = rs and rt)
            Some(Some(Insn {
              kind: Kind::Register,
              inputs: vec![get_rs(op), get_rt(op)],
              output: Some(get_rd(op)),
            }))
          },
          0x25 => {
            //OR
            log!("> OR");
            //compute!(rd = rs or rt)
            Some(Some(Insn {
              kind: Kind::Register,
              inputs: vec![get_rs(op), get_rt(op)],
              output: Some(get_rd(op)),
            }))
          },
          0x26 => {
            //XOR
            log!("> XOR");
            //compute!(rd = rs xor rt)
            Some(Some(Insn {
              kind: Kind::Register,
              inputs: vec![get_rs(op), get_rt(op)],
              output: Some(get_rd(op)),
            }))
          },
          0x27 => {
            //NOR
            log!("> NOR");
            //compute!(rd = rs nor rt)
            Some(Some(Insn {
              kind: Kind::Register,
              inputs: vec![get_rs(op), get_rt(op)],
              output: Some(get_rd(op)),
            }))
          },
          0x2A => {
            //SLT
            log!("> SLT");
            //compute!(rd = rs signed_compare rt)
            Some(Some(Insn {
              kind: Kind::Register,
              inputs: vec![get_rs(op), get_rt(op)],
              output: Some(get_rd(op)),
            }))
          },
          0x2B => {
            //SLTU
            log!("> SLTU");
            //compute!(rd = rs compare rt)
            Some(Some(Insn {
              kind: Kind::Register,
              inputs: vec![get_rs(op), get_rt(op)],
              output: Some(get_rd(op)),
            }))
          },
          _ => {
            //invalid opcode
            unreachable!("ran into invalid opcode")
          }
        }
      },
      0x01 => {
        //BcondZ
        match get_rt(op) {
          0x00 => {
            //BLTZ
            /*jump!(rs < 0);*/
            None
          },
          0x01 => {
            //BGEZ
            /*jump!(rs >= 0);*/
            None
          },
          0x80 => {
            //BLTZAL
            /*call!(rs < 0);*/
            None
          },
          0x81 => {
            //BGEZAL
            /*call!(rs >= 0);*/
            None
          },
          _ => {
            //invalid opcode
            unreachable!("ran into invalid opcode")
          },
        }
      },
      0x02 => {
        //J
        /*jump!(imm26);*/
        None
      },
      0x03 => {
        //JAL
        /*call!(imm26);*/
        None
      },
      0x04 => {
        //BEQ
        /*jump!(rs == rt);*/
        None
      },
      0x05 => {
        //BNE
        /*jump!(rs != rt);*/
        None
      },
      0x06 => {
        //BLEZ
        /*jump!(rs <= 0);*/
        None
      },
      0x07 => {
        //BGTZ
        /*jump!(rs > 0);*/
        None
      },
      0x08 => {
        //ADDI
        log!("> ADDI");
        //compute!(rt = rs checked_add signed imm16 trap)
        Some(Some(Insn {
          kind: Kind::Immediate,
          inputs: vec![get_rs(op)],
          output: Some(get_rt(op)),
        }))
      },
      0x09 => {
        //ADDIU
        log!("> ADDIU");
        //compute!(rt = rs wrapping_add signed imm16)
        Some(Some(Insn {
          kind: Kind::Immediate,
          inputs: vec![get_rs(op)],
          output: Some(get_rt(op)),
        }))
      },
      0x0A => {
        //SLTI
        log!("> SLTI");
        //compute!(rt = rs signed_compare imm16)
        Some(Some(Insn {
          kind: Kind::Immediate,
          inputs: vec![get_rs(op)],
          output: Some(get_rt(op)),
        }))
      },
      0x0B => {
        //SLTIU
        log!("> SLTIU");
        //compute!(rt = rs compare imm16)
        Some(Some(Insn {
          kind: Kind::Immediate,
          inputs: vec![get_rs(op)],
          output: Some(get_rt(op)),
        }))
      },
      0x0C => {
        //ANDI
        log!("> ANDI");
        //compute!(rt = rs and imm16)
        Some(Some(Insn {
          kind: Kind::Immediate,
          inputs: vec![get_rs(op)],
          output: Some(get_rt(op)),
        }))
      },
      0x0D => {
        //ORI
        log!("> ORI");
        //compute!(rt = rs or imm16)
        Some(Some(Insn {
          kind: Kind::Immediate,
          inputs: vec![get_rs(op)],
          output: Some(get_rt(op)),
        }))
      },
      0x0E => {
        //XORI
        log!("> XORI");
        //compute!(rt = rs xor imm16)
        Some(Some(Insn {
          kind: Kind::Immediate,
          inputs: vec![get_rs(op)],
          output: Some(get_rt(op)),
        }))
      },
      0x0F => {
        //LUI
        log!("> LUI");
        //compute!(rt = imm16 shl 16)
        Some(Some(Insn {
          kind: Kind::Immediate,
          inputs: vec![],
          output: Some(get_rt(op)),
        }))
      },
      0x10 => {
        //COP0
        log!("> COP0");
        //cop!(cop0)
        Some(None)
      },
      0x11 => {
        //COP1
        unreachable!("COP1 is not implemented on the PSX")
      },
      0x12 => {
        //COP2
        log!("> COP2");
        //cop!(gte)
        Some(None)
      },
      0x13 => {
        //COP3
        unreachable!("COP3 is not implemented on the PSX")
      },
      0x20 => {
        //LB
        log!("> LB");
        //mov!(rt = [rs + imm16] read_byte_sign_extended)
        Some(None)
      },
      0x21 => {
        //LH
        log!("> LH");
        //mov!(rt = [rs + imm16] read_half_sign_extended)
        Some(None)
      },
      0x22 => {
        //LWL
        log!("> LWL");
        //mov!(rt = [rs + imm16] left)
        Some(None)
      },
      0x23 => {
        //LW
        log!("> LW");
        //mov!(rt = [rs + imm16] read_word)
        Some(None)
      },
      0x24 => {
        //LBU
        log!("> LBU");
        //mov!(rt = [rs + imm16] read_byte)
        Some(None)
      },
      0x25 => {
        //LHU
        log!("> LHU");
        //mov!(rt = [rs + imm16] read_half)
        Some(None)
      },
      0x26 => {
        //LWR
        log!("> LWR");
        //mov!(rt = [rs + imm16] right)
        Some(None)
      },
      0x28 => {
        //SB
        log!("> SB");
        //mov!([rs + imm16] = rt write_byte)
        Some(None)
      },
      0x29 => {
        //SH
        log!("> SH");
        //mov!([rs + imm16] = rt write_half)
        Some(None)
      },
      0x2A => {
        //SWL
        log!("> SWL");
        //mov!([rs + imm16] = rt left)
        Some(None)
      },
      0x2B => {
        //SW
        log!("> SW");
        //mov!([rs + imm16] = rt write_word)
        Some(None)
      },
      0x2E => {
        //SWR
        log!("> SWR");
        //mov!([rs + imm16] = rt right)
        Some(None)
      },
      0x30 => {
        //LWC0
        unreachable!("LWC0 is not implemented on the PSX")
      },
      0x31 => {
        //LWC1
        unreachable!("LWC1 is not implemented on the PSX")
      },
      0x32 => {
        //LWC2
        todo!("lwc2")
      },
      0x33 => {
        //LWC3
        unreachable!("LWC3 is not implemented on the PSX")
      },
      0x38 => {
        //SWC0
        unreachable!("SWC0 is not implemented on the PSX")
      },
      0x39 => {
        //SWC1
        unreachable!("SWC1 is not implemented on the PSX")
      },
      0x3A => {
        //SWC2
        todo!("swc2")
      },
      0x3B => {
        //SWC3
        unreachable!("SWC3 is not implemented on the PSX")
      },
      _ => {
        //invalid opcode
        unreachable!("ran into invalid opcode")
      }
    }
  }
}

