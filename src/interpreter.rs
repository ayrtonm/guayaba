use std::io;
use crate::register::Register;
use crate::r3000::R3000;
use crate::memory::Memory;
use crate::cd::CD;

pub struct Interpreter {
  r3000: R3000,
  memory: Memory,
  cd: Option<CD>,
  next_pc: Option<Register>,
}

impl Interpreter {
  pub fn new(bios_filename: &String, infile: Option<&String>) -> io::Result<Self> {
    let r3000 = R3000::new();
    let memory = Memory::new(bios_filename)?;
    let cd = infile.and_then(|f| CD::new(f).ok());
    Ok(Interpreter {
      r3000,
      memory,
      cd,
      next_pc: None,
    })
  }
  fn step(&mut self) {
    //println!("attempting to read opcode at address {:#x}", self.r3000.pc().get_value());
    let op = self.memory.read_word(self.r3000.pc().get_value());
    //println!("decoding opcode {:#x}", op);
    println!("decoding opcode {:#x} from address {:#x}", op, self.r3000.pc().get_value());
    match &mut self.next_pc {
      Some(next_pc) => {
        *self.r3000.pc() = self.next_pc.take().unwrap();
      },
      None => {
        *self.r3000.pc() += 4;
      },
    };
    self.next_pc = self.decode_opcode(op);
  }
  fn decode_opcode(&mut self, op: u32) -> Option<Register> {
    let a = ((op & 0xfb00_0000) >> 26) as u8;
    match a {
      0x00 => {
        //SPECIAL
        let b = (op & 0x0000_003f) as u8;
        match b {
          0x00 => {
            //SLL
            None
          },
          0x02 => {
            //SRL
            None
          },
          0x03 => {
            //SRA
            None
          },
          0x04 => {
            //SLLV
            None
          },
          0x06 => {
            //SRLV
            None
          },
          0x07 => {
            //SRAV
            None
          },
          0x08 => {
            //JR
            let rs = Register::new((op & 0x03e0_0000) >> 21);
            println!("jumping to {:#x}", rs.get_value());
            Some(rs)
          },
          0x09 => {
            //JALR
            None
          },
          0x0C => {
            //SYSCALL
            None
          },
          0x0D => {
            //BREAK
            None
          },
          0x10 => {
            //MFHI
            None
          },
          0x11 => {
            //MTHI
            None
          },
          0x12 => {
            //MFLO
            None
          },
          0x13 => {
            //MTLO
            None
          },
          0x18 => {
            //MULT
            None
          },
          0x19 => {
            //MULTU
            None
          },
          0x1A => {
            //DIV
            None
          },
          0x1B => {
            //DIVU
            None
          },
          0x20 => {
            //ADD
            None
          },
          0x21 => {
            //ADDU
            None
          },
          0x22 => {
            //SUB
            None
          },
          0x23 => {
            //SUBU
            None
          },
          0x24 => {
            //AND
            None
          },
          0x25 => {
            //OR
            None
          },
          0x26 => {
            //XOR
            None
          },
          0x27 => {
            //NOR
            None
          },
          0x2A => {
            //SLT
            None
          },
          0x2B => {
            //SLTU
            None
          },
          _ => {
            //invalid opcode
            panic!("ran into invalid opcode")
          }
        }
      },
      0x01 => {
        //BcondZ
        None
      },
      0x02 => {
        //J
        let imm = op & 0x03ff_ffff;
        let dest = (self.r3000.pc() & 0xf000_0000) + (imm * 4);
        println!("jumping to {:#x}", dest.get_value());
        Some(dest)
      },
      0x03 => {
        //JAL
        let imm = op & 0x03ff_ffff;
        let dest = (self.r3000.pc() & 0xf000_0000) + (imm * 4);
        *self.r3000.ra() += 8;
        println!("jumping to {:#x}", dest.get_value());
        Some(dest)
      },
      0x04 => {
        //BEQ
        None
      },
      0x05 => {
        //BNE
        None
      },
      0x06 => {
        //BLEZ
        None
      },
      0x07 => {
        //BGTZ
        None
      },
      0x08 => {
        //ADDI
        None
      },
      0x09 => {
        //ADDIU
        None
      },
      0x0A => {
        //SLTI
        None
      },
      0x0B => {
        //SLTIU
        None
      },
      0x0C => {
        //ANDI
        None
      },
      0x0D => {
        //ORI
        None
      },
      0x0E => {
        //XORI
        None
      },
      0x0F => {
        //LUI
        None
      },
      0x10 => {
        //COP0
        None
      },
      0x11 => {
        //COP1
        None
      },
      0x12 => {
        //COP2
        None
      },
      0x13 => {
        //COP3
        None
      },
      0x20 => {
        //LB
        None
      },
      0x21 => {
        //LH
        None
      },
      0x22 => {
        //LWL
        None
      },
      0x23 => {
        //LW
        None
      },
      0x24 => {
        //LBU
        None
      },
      0x25 => {
        //LHU
        None
      },
      0x26 => {
        //LWR
        None
      },
      0x28 => {
        //SB
        None
      },
      0x29 => {
        //SH
        None
      },
      0x2A => {
        //SWL
        None
      },
      0x2B => {
        //SW
        None
      },
      0x2E => {
        //SWR
        None
      },
      0x30 => {
        //LWC0
        None
      },
      0x31 => {
        //LWC1
        None
      },
      0x32 => {
        //LWC2
        None
      },
      0x33 => {
        //LWC3
        None
      },
      0x38 => {
        //SWC0
        let rs = ((op & 0x03e0_0000) >> 21) as u8;
        let rt = ((op & 0x001f_0000) >> 16) as u8;
        let imm = (op & 0x0000_ffff) as u16;
        None
      },
      0x39 => {
        //SWC1
        None
      },
      0x3A => {
        //SWC2
        None
      },
      0x3B => {
        //SWC3
        None
      },
      _ => {
        //invalid opcode
        panic!("ran into invalid opcode")
      }
    }
  }
  pub fn run(&mut self) {
    let n = 50;
    for i in 0..n {
      self.step();
    }
    if self.cd.is_some() {
      self.cd.as_ref().unwrap().preview(10);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy_bios() {
    const BIOS: u32 = 0x1fc0_0000;
    let mut vm = Interpreter::new(&"/home/ayrton/dev/rps/scph1001.bin".to_string(), None).unwrap();
    vm.memory.write_word(BIOS, 0x0000_0002);
    let dest: u32 = 0x0bf0_0000;
    let instr: u32 = (2 << 26) | (dest & 0x03ff_ffff);
    vm.memory.write_word(BIOS + 4, 0x0000_0003);
    vm.memory.write_word(BIOS + 8, 0x0000_0004);
    vm.memory.write_word(BIOS + 12, instr);
    vm.memory.write_word(BIOS + 16, 0x0000_0006);
    vm.run();
  }
}
