use std::io;
use crate::common::get_rs;
use crate::common::get_rt;
use crate::common::get_rd;
use crate::common::get_imm16;
use crate::common::get_imm26;
use crate::common::get_primary_field;
use crate::common::get_secondary_field;
use crate::register::Register;
use crate::r3000::R3000;
use crate::r3000::Write;
use crate::r3000::Name;
use crate::r3000::General;
use crate::r3000::idx_to_name;
use crate::memory::Memory;
use crate::cd::CD;

macro_rules! delayed_load {
  (rt = $rhs:ident, $self:expr, $new_writes:expr, $op: expr) => {
    let rs = $self.r3000.nth_reg(get_rs($op));
    let imm = get_imm16($op);
    let rt = get_rt($op);
    $new_writes.push(Write::new(Name::gpr(idx_to_name(rt)),
                     $self.memory.read_word(&(rs + imm)).$rhs()));
  };
}

macro_rules! compute_then_assign {
  (rd = rs $operator:tt rt, $self:expr, $op:expr) => {
    let rs = $self.r3000.nth_reg(get_rs($op));
    let rt = $self.r3000.nth_reg(get_rt($op));
    let result = rs $operator rt;
    let rd = $self.r3000.nth_reg_mut(get_rd($op));
    *rd = result;
  };
  (rt = rs $operator:tt imm16, $self:expr, $op:expr) => {
    let rs = $self.r3000.nth_reg(get_rs($op));
    let imm = get_imm16($op);
    let result = rs $operator imm;
    let rt = $self.r3000.nth_reg_mut(get_rt($op));
    *rt = result;
  };
}

pub struct Interpreter {
  r3000: R3000,
  memory: Memory,
  cd: Option<CD>,
  next_pc: Option<Register>,
  delayed_writes: Option<Vec<Write>>,
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
      delayed_writes: None,
    })
  }
  pub fn run(&mut self) {
    let n = 10;
    for _ in 0..n {
      self.step();
    }
    self.cd.as_ref().map(|cd| cd.preview(10));
  }
  fn step(&mut self) {
    //get opcode from memory at program counter
    let op = self.memory.read_word(self.r3000.pc());
    println!("decoding opcode {:#x} from address {:#x}", op.get_value(), self.r3000.pc().get_value());
    //the instruction following each jump is always executed before updating the pc
    *self.r3000.pc() = self.next_pc
                           .take()
                           .map_or_else(|| self.r3000.pc() + 4, |next_pc| next_pc);
    self.next_pc = self.execute_opcode(op.get_value());
  }
  //if program counter should incremented normally, return None
  //otherwise return Some(new program counter)
  fn execute_opcode(&mut self, op: u32) -> Option<Register> {
    let mut new_writes = Vec::new();
    let a = get_primary_field(op);
    println!("primary field is {:#x?}", a);
    let next_pc = match a {
      0x00 => {
        //SPECIAL
        let b = get_secondary_field(op);
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
            let rs = self.r3000.nth_reg(get_rs(op));
            println!("jumping to {:#x}", rs.get_value());
            Some(rs.clone());
            None
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
            //since self.r3000 is borrowed mutably on the lhs, the rhs must be
            //computed from the immutable references before assigning its value
            //to the lhs
            compute_then_assign!(rd = rs + rt, self, op);
            None
          },
          0x22 => {
            //SUB
            None
          },
          0x23 => {
            //SUBU
            compute_then_assign!(rd = rs - rt, self, op);
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
        let imm = get_imm26(op);
        let dest = (self.r3000.pc() & 0xf000_0000) + (imm * 4);
        Some(dest)
      },
      0x03 => {
        //JAL
        let imm = get_imm26(op);
        let dest = (self.r3000.pc() & 0xf000_0000) + (imm * 4);
        *self.r3000.ra() += 8;
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
        compute_then_assign!(rt = rs + imm16, self, op);
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
        compute_then_assign!(rt = rs & imm16, self, op);
        None
      },
      0x0D => {
        //ORI
        compute_then_assign!(rt = rs | imm16, self, op);
        None
      },
      0x0E => {
        //XORI
        compute_then_assign!(rt = rs ^ imm16, self, op);
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
        delayed_load!(rt = lowest_byte, self, new_writes, op);
        None
      },
      0x21 => {
        //LH
        delayed_load!(rt = lower_half, self, new_writes, op);
        None
      },
      0x22 => {
        //LWL
        None
      },
      0x23 => {
        //LW
        delayed_load!(rt = word, self, new_writes, op);
        None
      },
      0x24 => {
        //LBU
        //loading the value from memory is a delayed operation (i.e. the updated
        //register is not visible to the next opcode). this would work if the
        //first argument to Write::new were a Name, but I need for this to work
        //with register indices as well
        delayed_load!(rt = lowest_byte, self, new_writes, op);
        None
      },
      0x25 => {
        //LHU
        delayed_load!(rt = lower_half, self, new_writes, op);
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
    };
    //after executing an opcode, complete the loads from the previous opcode
    self.delayed_writes.take().map(|writes| self.r3000.flush_write_cache(writes));
    //put the loads from the current opcode next in line
    if new_writes.len() > 0 {
      self.delayed_writes = Some(new_writes);
    }
    next_pc
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn f() {
    toy_macro!(rt = 0);
  }
  //this is the entry point in case we want to test some dummy instructions
  const BIOS: u32 = 0x1fc0_0000;
  #[test]
  fn dummy_bios() {
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
