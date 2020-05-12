use crate::register::Register;
use crate::register::BitBang;
use crate::r3000::MaybeSet;
use crate::cop0::Cop0Exception;
use crate::dummy_jit::Dummy_JIT;
use crate::console::Console;
use crate::common::*;

impl Dummy_JIT {
  pub(super) fn compile_jump(&self, op: u32, logging: bool) -> Box<dyn Fn(&mut Console)> {
    macro_rules! log {
      () => ($crate::print!("\n"));
      ($($arg:tt)*) => ({
        if logging {
          println!($($arg)*);
        };
      })
    }
    macro_rules! jump {
      (imm26) => {
        {
          Box::new(move |vm| {
            vm.next_pc = {
              let imm26 = get_imm26(op);
              let pc_hi_bits = vm.r3000.pc() & 0xf000_0000;
              let shifted_imm26 = imm26 * 4;
              let dest = pc_hi_bits + shifted_imm26;
              log!("jumping to (PC & 0xf0000000) + ({:#x} * 4)\n  = {:#x} + {:#x}\n  = {:#x} after the delay slot",
                        imm26, pc_hi_bits, shifted_imm26, dest);
              Some(dest)
            }
          })
        }
      };
      (rs) => {
        {
          Box::new(move |vm| {
            vm.next_pc = {
              let rs = vm.r3000.nth_reg(get_rs(op));
              if rs & 0x0000_0003 != 0 {
                let pc = vm.r3000.pc_mut();
                *pc = vm.cop0.generate_exception(Cop0Exception::LoadAddress, *pc);
                log!("ignoring jumping to R{} = {:#x} and generating an exception", get_rs(op), rs);
                None
              } else {
                log!("jumping to R{} = {:#x} after the delay slot", get_rs(op), rs);
                Some(rs)
              }
            }
          })
        }
      };
      (rs $cmp:tt rt) => {
        {
          Box::new(move |vm| {
            vm.next_pc = {
              let rt = vm.r3000.nth_reg(get_rt(op));
              let rs = vm.r3000.nth_reg(get_rs(op));
              if rs $cmp rt {
                let imm16 = get_imm16(op);
                let pc = vm.r3000.pc();
                let inc = ((imm16.half_sign_extended() as i32) * 4) as u32;
                let dest = pc.wrapping_add(inc);
                log!("jumping to PC + ({:#x} * 4) = {:#x} + {:#x} = {:#x} after the delay slot\n  since R{} {} R{} -> {:#x} {} {:#x}",
                          imm16, pc, inc, dest, get_rs(op), stringify!($cmp), get_rt(op), rs, stringify!($cmp), rt);
                Some(dest)
              } else {
                log!("skipping jump since R{} {} R{} -> {:#x} {} {:#x} is false",
                          get_rs(op), stringify!($cmp), get_rt(op), rs, stringify!($cmp), rt);
                None
              }
            }
          })
        }
      };
      (rs $cmp:tt 0) => {
        {
          Box::new(move |vm| {
            vm.next_pc = {
              let rs = vm.r3000.nth_reg(get_rs(op));
              log!("op16");
              if (rs as i32) $cmp 0 {
                let imm16 = get_imm16(op);
                let pc = vm.r3000.pc();
                let inc = ((imm16 as i16) * 4) as u32;
                let dest = pc.wrapping_add(inc);
                Some(dest)
              } else {
                None
              }
            }
          })
        }
      };
    }
    macro_rules! call {
      (imm26) => {
        {
          Box::new(move |vm| {
            vm.next_pc = {
              let ret = vm.r3000.pc().wrapping_add(4);
              vm.modified_register = vm.r3000.ra_mut().maybe_set(ret);
              log!("R31 = {:#x}", ret);
              let imm26 = get_imm26(op);
              let pc_hi_bits = vm.r3000.pc() & 0xf000_0000;
              let shifted_imm26 = imm26 * 4;
              let dest = pc_hi_bits + shifted_imm26;
              log!("jumping to (PC & 0xf0000000) + ({:#x} * 4)\n  = {:#x} + {:#x}\n  = {:#x} after the delay slot",
                        imm26, pc_hi_bits, shifted_imm26, dest);
              Some(dest)
            }
          })
        }
      };
      (rs) => {
        {
          Box::new(move |vm| {
            vm.next_pc = {
              let result = vm.r3000.pc().wrapping_add(4);
              let rd = vm.r3000.nth_reg_mut(get_rd(op));
              vm.modified_register = rd.maybe_set(result);
              log!("op18");
              let rs = vm.r3000.nth_reg(get_rs(op));
              if rs & 0x0000_0003 != 0 {
                let pc = vm.r3000.pc_mut();
                *pc = vm.cop0.generate_exception(Cop0Exception::LoadAddress, *pc);
                log!("ignoring jumping to R{} = {:#x} and generating an exception", get_rs(op), rs);
                None
              } else {
                log!("jumping to R{} = {:#x} after the delay slot", get_rs(op), rs);
                Some(rs)
              }
            }
          })
        }
      };
      (rs $cmp:tt rt) => {
        {
          Box::new(move |vm| {
            vm.next_pc = {
              let rt = vm.r3000.nth_reg(get_rt(op));
              let rs = vm.r3000.nth_reg(get_rs(op));
              log!("op19");
              if *rs $cmp *rt {
                let ret = vm.r3000.pc().wrapping_add(4);
                vm.modified_register = vm.r3000.ra_mut().maybe_set(ret);
                let imm16 = get_imm16(op);
                let pc = vm.r3000.pc();
                let inc = ((imm16 as i16) * 4) as u32;
                let dest = pc.wrapping_add(inc);
                Some(dest)
              } else {
                None
              }
            }
          })
        }
      };
      (rs $cmp:tt 0) => {
        {
          Box::new(move |vm| {
            vm.next_pc = {
              let rs = vm.r3000.nth_reg(get_rs(op));
              log!("op20");
              if (rs as i32) $cmp 0 {
                let ret = vm.r3000.pc().wrapping_add(4);
                vm.modified_register = vm.r3000.ra_mut().maybe_set(ret);
                let imm16 = get_imm16(op);
                let pc = vm.r3000.pc();
                let dest = pc + (imm16 * 4);
                Some(dest)
              } else {
                None
              }
            }
          })
        }
      };
    }
    match get_primary_field(op) {
      0x00 => {
        //SPECIAL
        match get_secondary_field(op) {
          0x08 => {
            //JR
            log!("> JR");
            jump!(rs)
          },
          0x09 => {
            //JALR
            log!("> JALR");
            call!(rs)
          },
          0x0C => {
            //SYSCALL
            log!("> SYSCALL");
            Box::new(move |vm| {
              vm.next_pc = {
                Some(vm.cop0.generate_exception(Cop0Exception::Syscall, vm.r3000.pc()))
              }
            })
          },
          _ => {
            unreachable!("");
          },
        }
      },
      0x01 => {
        //BcondZ
        match get_rt(op) {
          0x00 => {
            //BLTZ
            log!("> BLTZ");
            jump!(rs < 0)
          },
          0x01 => {
            //BGEZ
            log!("> BGEZ");
            jump!(rs >= 0)
          },
          0x80 => {
            //BLTZAL
            log!("> BLTZAL");
            call!(rs < 0)
          },
          0x81 => {
            //BGEZAL
            log!("> BGEZAL");
            call!(rs >= 0)
          },
          _ => {
            //invalid opcode
            unreachable!("ran into invalid opcode")
          },
        }
      },
      0x02 => {
        //J
        log!("> J");
        jump!(imm26)
      },
      0x03 => {
        //JAL
        log!("> JAL");
        call!(imm26)
      },
      0x04 => {
        //BEQ
        log!("> BEQ");
        jump!(rs == rt)
      },
      0x05 => {
        //BNE
        log!("> BNE");
        jump!(rs != rt)
      },
      0x06 => {
        //BLEZ
        log!("> BLEZ");
        jump!(rs <= 0)
      },
      0x07 => {
        //BGTZ
        log!("> BGTZ");
        jump!(rs > 0)
      },
      _ => {
        unreachable!("");
      },
    }
  }
}
