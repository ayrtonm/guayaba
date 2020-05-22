use crate::common::*;

//this tags each opcode with its input and output registers
pub struct Insn {
  opcode: u32,
  //offset into the nominal block
  offset: u32,
  //registers which are used directly, i.e. not as an index into memory
  inputs: Vec<u32>,
  //sometimes an input register is used as an index into memory
  indices: Option<u32>,
  //the modified register if any
  output: Option<u32>,
}

impl Insn {
  pub fn new(op: u32, offset: u32) -> Self {
    let (inputs, indices, output) =
      match get_primary_field(op) {
        0x00 => {
          match get_secondary_field(op) {
            0x00 | 0x02 | 0x03 => {
              //SLL, SRL, SRA
              (vec![get_rt(op)], None, Some(get_rd(op)))
            },
            0x04 | 0x06 | 0x07 => {
              //SLLV, SRLV, SRAV
              (vec![get_rt(op), get_rs(op)], None, Some(get_rd(op)))
            },
            0x08 => {
              //JR
              (vec![get_rs(op)], None, None)
            },
            0x09 => {
              //JALR
              (vec![get_rs(op)], None, Some(get_rd(op)))
            },
            0x0C => {
              //SYSCALL
              (vec![], None, None)
            },
            0x0D => {
              //BREAK
              (vec![], None, None)
            },
            0x10 | 0x12 => {
              //MFHI, MFLO
              (vec![], None, Some(get_rd(op)))
            },
            0x11 | 0x13 => {
              //MTHI, MTLO
              (vec![get_rs(op)], None, None)
            },
            0x18 | 0x19 | 0x1A | 0x1B => {
              //MULT, MULTU, DIV, DIVU
              (vec![get_rs(op), get_rt(op)], None, None)
            },
            0x20 | 0x21 | 0x22 | 0x23 | 0x24 |
            0x25 | 0x26 | 0x27 | 0x2A | 0x2B => {
              (vec![get_rs(op), get_rt(op)], None, Some(get_rd(op)))
            },
            _ => {
              unreachable!("Invalid opcode {:#x}", op);
            },
          }
        },
        0x01 => {
          //BcondZ
          match get_rt(op) {
            0x00 | 0x01 => {
              //BLTZ, BGEZ
              (vec![get_rs(op)], None, None)
            },
            0x80 | 0x81 => {
              //BLTZAL, BGEZAL
              (vec![get_rs(op)], None, Some(31))
            },
            _ => {
              unreachable!("Invalid opcode {:#x}", op);
            },
          }
        },
        0x02 => {
          //J
          (vec![], None, None)
        },
        0x03 => {
          //JAL
          (vec![], None, Some(31))
        },
        0x04..=0x07 => {
          //BEQ, BNE, BLEZ, BGTZ
          (vec![get_rs(op), get_rt(op)], None, None)
        },
        0x08..=0x0E => {
          //ADDI
          (vec![get_rs(op)], None, Some(get_rt(op)))
        },
        0x0F => {
          //LUI
          (vec![], None, Some(get_rt(op)))
        },
        0x10 | 0x12 => {
          //COPn for COP0 and COP2 (GTE)
          match get_rs(op) {
            0x00 | 0x02 => {
              //MFCn, CFCn
              (vec![], None, Some(get_rt(op)))
            },
            0x04 | 0x06 => {
              //MTCn, CTCn
              (vec![get_rt(op)], None, None)
            },
            0x08 => {
              match get_rt(op) {
                0x00 => {
                  (vec![], None, None)
                },
                0x01 => {
                  (vec![], None, None)
                },
                _ => {
                  unreachable!("Invalid opcode {:#x}", op);
                },
              }
            },
            0x10..=0x1F => {
              (vec![], None, None)
            },
            _ => {
              unreachable!("Invalid opcode {:#x}", op);
            },
          }
        },
        0x20..=0x26 => {
          //LB, LH, LWL, LW, LBU, LHU, LWR
          (vec![], Some(get_rs(op)), Some(get_rt(op)))
        },
        0x28..=0x2B | 0x2E => {
          //SB, SH, SWL, SW, SWR
          (vec![get_rt(op)], Some(get_rs(op)), None)
        },
        _ => {
          unreachable!("Invalid opcode {:#x}", op);
        },
      };
    Insn {
      opcode: op,
      offset,
      inputs,
      indices,
      output,
    }
  }
  pub fn op(&self) -> u32 {
    self.opcode
  }
  pub fn offset(&self) -> u32 {
    self.offset
  }
  pub fn has_branch_delay_slot(op: u32) -> bool {
    match get_primary_field(op) {
      0x00 => {
        match get_secondary_field(op) {
          0x08 | 0x09 => true,
          _ => false,
        }
      },
      0x02 | 0x03 => true,
      _ => false,
    }
  }
  pub fn is_unconditional_jump(op: u32) -> bool {
    match get_primary_field(op) {
      0x00 => {
        match get_secondary_field(op) {
          0x08 | 0x09 | 0x0c | 0x0d => true,
          _ => false,
        }
      },
      0x02 | 0x03 => true,
      _ => false,
    }
  }
}
