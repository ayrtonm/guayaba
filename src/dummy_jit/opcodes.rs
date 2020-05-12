use std::ops::Add;
use std::ops::Shl;
use std::ops::Shr;
use std::ops::Sub;
use crate::register::BitBang;
use crate::r3000::MaybeSet;
use crate::r3000::DelayedWrite;
use crate::r3000::Name;
use crate::cop0::Cop0Exception;
use crate::dummy_jit::JIT;
use crate::console::Console;
use crate::common::*;

impl JIT {
  //if program counter should incremented normally, return None
  //otherwise return Some(new program counter)
  pub(super) fn compile_opcode(&mut self, op: u32, logging: bool) -> Option<Box<dyn Fn(&mut Console)>> {
    macro_rules! log {
      () => ($crate::print!("\n"));
      ($($arg:tt)*) => ({
        if logging {
          println!($($arg)*);
        };
      })
    }
    //loading a value from memory is a delayed operation (i.e. the updated register
    //is not visible to the next opcode). Note that the rs + imm16 in parentheses is
    //symbolic and only used to improve readability. This macro should be able to
    //handle all loads in the MIPS instructions set so there's no point to generalizing it
    macro_rules! mov {
      (rt = [rs + imm16] left) => {
        {
          mov!(rt = [rs + imm16] 24_u32, sub lowest_bits shl)
        }
      };
      (rt = [rs + imm16] right) => {
        {
          mov!(rt = [rs + imm16] 0_u32, add upper_bits_in_place shr)
        }
      };
      (rt = [rs + imm16] $offset:expr, $operator:ident $mask:ident $shift:ident) => {
        {
          Some(Box::new(move |vm| {
            let rs = vm.r3000.nth_reg(get_rs(op));
            let imm16 = get_imm16(op).half_sign_extended();
            let rt_idx = get_rt(op);
            let rt = vm.delayed_writes.iter()
                                        .rev()
                                        .find(|write| *write.name() == Name::Rn(rt_idx))
                                        .map_or(vm.r3000.nth_reg(rt_idx),|write| write.value());
            let address = rs.wrapping_add(imm16);
            let aligned_address = *address.clone().clear_mask(3);
            let aligned_word = vm.resolve_memresponse(vm.memory.read_word(aligned_address));
            let num_bits = $offset.$operator(8*address.lowest_bits(2));
            let result = rt.$mask(num_bits) | aligned_word.$shift(num_bits);
            vm.delayed_writes.push_back(DelayedWrite::new(Name::Rn(rt_idx), result));
          }))
        }
      };
      //delayed aligned reads
      (rt = [rs + imm16] $method:ident) => {
        {
          Some(Box::new(move |vm| {
            let rs = vm.r3000.nth_reg(get_rs(op));
            let imm16 = get_imm16(op).half_sign_extended();
            let rt = get_rt(op);
            let result = vm.resolve_memresponse(vm.memory.$method(rs.wrapping_add(imm16)));
            vm.delayed_writes.push_back(DelayedWrite::new(Name::Rn(rt), result));
            log!("R{} = [{:#x} + {:#x}] \n  = [{:#x}] \n  = {:#x} {}",
                      rt, rs, imm16, rs.wrapping_add(imm16), result, stringify!($method));
          }))
        }
      };
      ([rs + imm16] = rt left) => {
        {
          mov!([rs + imm16] = rt 24_u32, sub upper_bits_in_place shr)
        }
      };
      ([rs + imm16] = rt right) => {
        {
          mov!([rs + imm16] = rt 0_u32, add lowest_bits shl)
        }
      };
      ([rs + imm16] = rt $offset:expr, $operator:ident $mask:ident $shift:ident) => {
        {
          Some(Box::new(move |vm| {
            let rs = vm.r3000.nth_reg(get_rs(op));
            let rt = vm.r3000.nth_reg(get_rt(op));
            let imm16 = get_imm16(op).half_sign_extended();
            if !vm.cop0.cache_isolated() {
              let address = rs.wrapping_add(imm16);
              let aligned_address = *address.clone().clear_mask(3);
              let aligned_word = vm.resolve_memresponse(vm.memory.read_word(aligned_address));
              let num_bits = $offset.$operator(8*address.lowest_bits(2));
              let result = rt.$shift(num_bits) | aligned_word.$mask(num_bits);
              let maybe_action = vm.write_word(aligned_address, result);
              vm.resolve_memactions(maybe_action);
            } else {
              log!("ignoring write while cache is isolated");
            }
          }))
        }
      };
      //aligned writes
      ([rs + imm16] = rt $method:ident) => {
        {
          Some(Box::new(move |vm| {
            let rs = vm.r3000.nth_reg(get_rs(op));
            let rt = vm.r3000.nth_reg(get_rt(op));
            let imm16 = get_imm16(op).half_sign_extended();
            log!("[{:#x} + {:#x}] = [{:#x}] \n  = R{}\n  = {:#x} {}",
                      rs, imm16, rs.wrapping_add(imm16), get_rt(op), rt, stringify!($method));
            if !vm.cop0.cache_isolated() {
              let maybe_action = vm.$method(rs.wrapping_add(imm16), rt);
              vm.resolve_memactions(maybe_action);
            } else {
              log!("ignoring write while cache is isolated");
            }
          }))
        }
      };
      (lo = rs) => {
        {
          Some(Box::new(move |vm| {
            let rs = vm.r3000.nth_reg(get_rs(op));
            let lo = vm.r3000.lo_mut();
            *lo = rs;
            log!("op1");
          }))
        }
      };
      (hi = rs) => {
        {
          Some(Box::new(move |vm| {
            let rs = vm.r3000.nth_reg(get_rs(op));
            let hi = vm.r3000.hi_mut();
            *hi = rs;
            log!("op2");
          }))
        }
      };
      (rd = lo) => {
        {
          Some(Box::new(move |vm| {
            let lo = vm.r3000.lo();
            let rd_idx = get_rd(op);
            let rd = vm.r3000.nth_reg_mut(rd_idx);
            vm.modified_register = rd.maybe_set(lo);
            log!("op3");
          }))
        }
      };
      (rd = hi) => {
        {
          Some(Box::new(move |vm| {
            let hi = vm.r3000.hi();
            let rd_idx = get_rd(op);
            let rd = vm.r3000.nth_reg_mut(rd_idx);
            vm.modified_register = rd.maybe_set(hi);
            log!("op4");
          }))
        }
      };
    }
    //since vm.r3000 is borrowed mutably on the lhs, the rhs must be
    //computed from the immutable references before assigning its value
    //to the lhs
    macro_rules! compute {
      //ALU instructions with two general purpose registers
      (rd = rs $method:ident rt) => {
        {
          Some(Box::new(move |vm| {
            let rs = vm.r3000.nth_reg(get_rs(op));
            let rt = vm.r3000.nth_reg(get_rt(op));
            let rd = vm.r3000.nth_reg_mut(get_rd(op));
            vm.modified_register = rd.maybe_set(rs.$method(rt));
            log!("R{} = R{} {} R{}\n  = {:#x} {} {:#x}\n  = {:#x}",
                      get_rd(op), get_rs(op), stringify!($method), get_rt(op),
                      rs, stringify!($method), rt, vm.r3000.nth_reg(get_rd(op)));
          }))
        }
      };
      //ALU instructions with two general purpose registers that trap overflow
      (rd = rs $method:ident rt trap) => {
        {
          Some(Box::new(move |vm| {
            let rs = vm.r3000.nth_reg(get_rs(op)) as u64;
            let rt = vm.r3000.nth_reg(get_rt(op)) as u64;
            let rd = vm.r3000.nth_reg_mut(get_rd(op));
            let result = rs.$method(rt);
            match result {
              Some(result) => {
                vm.modified_register = rd.maybe_set(result as u32);
              },
              None => {
                let pc = vm.r3000.pc_mut();
                *pc = vm.cop0.generate_exception(Cop0Exception::Overflow, *pc);
              },
            }
            log!("R{} = R{} {} R{} trap overflow\n  = {:#x} {} {:#x}\n  = {:#x}",
                      get_rd(op), get_rs(op), stringify!($method), get_rt(op),
                      rs, stringify!($method), rt, vm.r3000.nth_reg(get_rd(op)));
          }))
        }
      };
      //ALU instructions with a register and immediate 16-bit data that trap overflow
      (rt = rs $method:ident signed imm16 trap) => {
        {
          Some(Box::new(move |vm| {
            let rs = vm.r3000.nth_reg(get_rs(op)) as i32;
            let imm16 = get_imm16(op).half_sign_extended() as i32;
            let rt = vm.r3000.nth_reg_mut(get_rt(op));
            let result = rs.$method(imm16);
            match result {
              Some(result) => {
                vm.modified_register = rt.maybe_set(result as u32);
              },
              None => {
                let pc = vm.r3000.pc_mut();
                *pc = vm.cop0.generate_exception(Cop0Exception::Overflow, *pc);
              },
            }
            log!("R{} = R{} {} {:#x} trap overflow\n  = {:#x} {} {:#x}\n  = {:#x}",
                      get_rt(op), get_rs(op), stringify!($method), imm16,
                      rs, stringify!($method), imm16, vm.r3000.nth_reg(get_rt(op)));
          }))
        }
      };
      //ALU instructions with a register and immediate 16-bit data
      (rt = rs $method:tt imm16) => {
        {
          Some(Box::new(move |vm| {
            let rs = vm.r3000.nth_reg(get_rs(op));
            let imm16 = get_imm16(op);
            let rt = vm.r3000.nth_reg_mut(get_rt(op));
            vm.modified_register = rt.maybe_set(rs.$method(imm16));
            log!("R{} = R{} {} {:#x}\n  = {:#x} {} {:#x}\n  = {:#x}",
                      get_rt(op), get_rs(op), stringify!($method), imm16,
                      rs, stringify!($method), imm16, vm.r3000.nth_reg(get_rt(op)));
          }))
        }
      };
      //ALU instructions with a register and immediate 16-bit data
      (rt = rs $method:tt signed imm16) => {
        {
          Some(Box::new(move |vm| {
            let rs = vm.r3000.nth_reg(get_rs(op));
            let imm16 = get_imm16(op).half_sign_extended();
            let rt = vm.r3000.nth_reg_mut(get_rt(op));
            vm.modified_register = rt.maybe_set(rs.$method(imm16));
            log!("R{} = R{} {} {:#x}\n  = {:#x} {} {:#x}\n  = {:#x}",
                      get_rt(op), get_rs(op), stringify!($method), imm16,
                      rs, stringify!($method), imm16, vm.r3000.nth_reg(get_rt(op)));
          }))
        }
      };
      //shifts a register based on immediate 5 bits
      (rd = rt $method:tt imm5) => {
        {
          Some(Box::new(move |vm| {
            let rt = vm.r3000.nth_reg(get_rt(op));
            let imm5 = get_imm5(op);
            let rd = vm.r3000.nth_reg_mut(get_rd(op));
            vm.modified_register = rd.maybe_set(rt.$method(imm5));
            log!("R{} = R{} {} {:#x}\n  = {:#x} {} {:#x}\n  = {:#x}",
                      get_rd(op), get_rt(op), stringify!($method), imm5,
                      rt, stringify!($method), imm5, vm.r3000.nth_reg(get_rd(op)));
          }))
        }
      };
      //shifts a register based on the lowest 5 bits of another register
      (rd = rt $method:tt (rs and 0x1F)) => {
        {
          Some(Box::new(move |vm| {
            let rt = vm.r3000.nth_reg(get_rt(op));
            let rs = vm.r3000.nth_reg(get_rs(op));
            let rd = vm.r3000.nth_reg_mut(get_rd(op));
            vm.modified_register = rd.maybe_set(rt.$method(rs & 0x1F));
            log!("op9");
          }))
        }
      };
      (rt = imm16 shl 16) => {
        {
          Some(Box::new(move |vm| {
            let rt = vm.r3000.nth_reg_mut(get_rt(op));
            let imm16 = get_imm16(op);
            vm.modified_register = rt.maybe_set(imm16 << 16);
            log!("R{} = {:#x} << 16 \n  = {:#x}", get_rt(op), imm16, vm.r3000.nth_reg(get_rt(op)));
          }))
        }
      };
      (hi:lo = rs * rt) => {
        {
          Some(Box::new(move |vm| {
            let rs = vm.r3000.nth_reg(get_rs(op));
            let rt = vm.r3000.nth_reg(get_rt(op));
            let result = (rs as u64) * (rt as u64);
            let hi_res = (result >> 32) as u32;
            let lo_res = (result & 0x0000_0000_ffff_ffff) as u32;
            let delay = match rs {
              0x0000_0000..=0x0000_07ff => {
                6
              },
              0x0000_0800..=0x000f_ffff => {
                9
              },
              0x0010_0000..=0xffff_ffff => {
                13
              },
            };
            //TODO: add delay back in
            //vm.delayed_writes.push(DelayedWrite::new(Name::Hi, hi_res, delay));
            //vm.delayed_writes.push(DelayedWrite::new(Name::Lo, lo_res, delay));
            *vm.r3000.hi_mut() = hi_res;
            *vm.r3000.lo_mut() = lo_res;
            log!("op11");
          }))
        }
      };
      (hi:lo = rs * rt signed) => {
        {
          Some(Box::new(move |vm| {
            let rs = vm.r3000.nth_reg(get_rs(op)) as i32;
            let rt = vm.r3000.nth_reg(get_rt(op)) as i32;
            let result = (rs as i64) * (rt as i64);
            let hi_res = (result >> 32) as u32;
            let lo_res = (result & 0x0000_0000_ffff_ffff) as u32;
            let delay = match rs as u32 {
              0x0000_0000..=0x0000_07ff | 0xffff_f800..=0xffff_ffff => {
                6
              },
              0x0000_0800..=0x000f_ffff | 0xfff0_0000..=0xffff_f801 => {
                9
              },
              0x0010_0000..=0x7fff_ffff | 0x8000_0000..=0xfff0_0001 => {
                13
              },
            };
            //TODO: add delay back in
            //vm.delayed_writes.push(DelayedWrite::new(Name::Hi, hi_res, delay));
            //vm.delayed_writes.push(DelayedWrite::new(Name::Lo, lo_res, delay));
            *vm.r3000.hi_mut() = hi_res;
            *vm.r3000.lo_mut() = lo_res;
            log!("op11");
          }))
        }
      };
      (hi:lo = rs / rt) => {
        {
          Some(Box::new(move |vm| {
            let rs = vm.r3000.nth_reg(get_rs(op));
            let rt = vm.r3000.nth_reg(get_rt(op));
            let lo_res = match rt {
              0 => {
                0xffff_ffff
              },
              _ => {
                rs / rt
              },
            };
            let hi_res = rs % rt;
            //TODO: add delay back in
            //vm.delayed_writes.push(DelayedWrite::new(Name::Hi, hi_res, 36));
            //vm.delayed_writes.push(DelayedWrite::new(Name::Lo, lo_res, 36));
            *vm.r3000.hi_mut() = hi_res;
            *vm.r3000.lo_mut() = lo_res;
            log!("op12");
          }))
        }
      };
      (hi:lo = rs / rt signed) => {
        {
          Some(Box::new(move |vm| {
            let rs = vm.r3000.nth_reg(get_rs(op)) as i32;
            let rt = vm.r3000.nth_reg(get_rt(op)) as i32;
            let lo_res = match rt {
              0 => {
                match rs {
                  0x0000_0000..=0x7fff_ffff => {
                    -1
                  },
                  -0x8000_0000..=-1 => {
                    1
                  },
                }
              },
              -1 => {
                match rs {
                  -0x8000_0000..=-1 => {
                    1
                  },
                  _ => {
                    rs / rt
                  },
                }
              }
              _ => {
                rs / rt
              },
            } as u32;
            let hi_res = (rs % rt) as u32;
            //TODO: add delay back in
            //vm.delayed_writes.push(DelayedWrite::new(Name::Hi, hi_res, 36));
            //vm.delayed_writes.push(DelayedWrite::new(Name::Lo, lo_res, 36));
            *vm.r3000.hi_mut() = hi_res;
            *vm.r3000.lo_mut() = lo_res;
            log!("op12");
          }))
        }
      };
    }
    macro_rules! cop {
      ($copn:ident) => {
        {
          match get_rs(op) {
            0x00 => {
              //MFCn
              Some(Box::new(move |vm| {
                let rt = get_rt(op);
                let rd_data = vm.$copn.nth_data_reg(get_rd(op));
                vm.delayed_writes.push_back(DelayedWrite::new(Name::Rn(rt), rd_data));
                log!("R{} = {}R{}\n  = {:#x} after the delay slot",
                          rt, stringify!($copn), get_rd(op), rd_data);
              }))
            },
            0x02 => {
              //CFCn
              Some(Box::new(move |vm| {
                let rt = get_rt(op);
                let rd_ctrl = vm.$copn.nth_ctrl_reg(get_rd(op));
                vm.delayed_writes.push_back(DelayedWrite::new(Name::Rn(rt), rd_ctrl));
              }))
            },
            0x04 => {
              //MTCn
              Some(Box::new(move |vm| {
                let rt = vm.r3000.nth_reg(get_rt(op));
                let rd = vm.$copn.nth_data_reg_mut(get_rd(op));
                vm.modified_register = rd.maybe_set(rt);
                log!("{}R{} = R{}\n  = {:#x}",
                          stringify!($copn), get_rd(op), get_rt(op),
                          vm.$copn.nth_data_reg(get_rd(op)));
              }))
            },
            0x06 => {
              //CTCn
              Some(Box::new(move |vm| {
                let rt = vm.r3000.nth_reg(get_rt(op));
                let rd = vm.$copn.nth_ctrl_reg_mut(get_rd(op));
                vm.modified_register = rd.maybe_set(rt);
              }))
            },
            0x08 => {
              match get_rt(op) {
                0x00 => {
                  //BCnF
                  Some(Box::new(move |vm| {
                    vm.$copn.bcnf(get_imm16(op));
                  }))
                },
                0x01 => {
                  //BCnT
                  //technically we're implementing one illegal instruction here
                  //since BCnT is not implemented for COP0
                  //however, GTE (i.e. COP2) does implement it
                  Some(Box::new(move |vm| {
                  }))
                },
                _ => {
                  unreachable!("ran into invalid opcode")
                },
              }
            },
            0x10..=0x1F => {
              //COPn imm25
              Some(Box::new(move |vm| {
                vm.$copn.execute_command(get_imm25(op));
              }))
            },
            _ => {
              unreachable!("ran into invalid opcode")
            },
          }
        }
      }
    }
    //after executing an opcode, complete the loads from the previous opcode
    //this match statement optionally returns the next program counter
    //if the return value is None, then we increment pc as normal
    match get_primary_field(op) {
      0x00 => {
        //SPECIAL
        match get_secondary_field(op) {
          0x00 => {
            //SLL
            log!("> SLL");
            compute!(rd = rt shl imm5)
          },
          0x02 => {
            //SRL
            log!("> SRL");
            compute!(rd = rt shr imm5)
          },
          0x03 => {
            //SRA
            log!("> SRA");
            compute!(rd = rt sra imm5)
          },
          0x04 => {
            //SLLV
            log!("> SLLV");
            compute!(rd = rt shl (rs and 0x1F))
          },
          0x06 => {
            //SRLV
            log!("> SRLV");
            compute!(rd = rt shr (rs and 0x1F))
          },
          0x07 => {
            //SRAV
            log!("> SRAV");
            compute!(rd = rt sra (rs and 0x1F))
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
            mov!(rd = hi)
          },
          0x11 => {
            //MTHI
            log!("> MTHI");
            mov!(hi = rs)
          },
          0x12 => {
            //MFLO
            log!("> MFLO");
            mov!(rd = lo)
          },
          0x13 => {
            //MTLO
            log!("> MTLO");
            mov!(lo = rs)
          },
          0x18 => {
            //MULT
            log!("> MULT");
            compute!(hi:lo = rs * rt signed)
          },
          0x19 => {
            //MULTU
            log!("> MULTU");
            compute!(hi:lo = rs * rt)
          },
          0x1A => {
            //DIV
            log!("> DIV");
            compute!(hi:lo = rs / rt signed)
          },
          0x1B => {
            //DIVU
            log!("> DIVU");
            compute!(hi:lo = rs / rt)
          },
          0x20 => {
            //ADD
            log!("> ADD");
            compute!(rd = rs checked_add rt trap)
          },
          0x21 => {
            //ADDU
            log!("> ADDU");
            compute!(rd = rs wrapping_add rt)
          },
          0x22 => {
            //SUB
            log!("> SUB");
            compute!(rd = rs checked_sub rt trap)
          },
          0x23 => {
            //SUBU
            log!("> SUBU");
            compute!(rd = rs wrapping_sub rt)
          },
          0x24 => {
            //AND
            log!("> AND");
            compute!(rd = rs and rt)
          },
          0x25 => {
            //OR
            log!("> OR");
            compute!(rd = rs or rt)
          },
          0x26 => {
            //XOR
            log!("> XOR");
            compute!(rd = rs xor rt)
          },
          0x27 => {
            //NOR
            log!("> NOR");
            compute!(rd = rs nor rt)
          },
          0x2A => {
            //SLT
            log!("> SLT");
            compute!(rd = rs signed_compare rt)
          },
          0x2B => {
            //SLTU
            log!("> SLTU");
            compute!(rd = rs compare rt)
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
        compute!(rt = rs checked_add signed imm16 trap)
      },
      0x09 => {
        //ADDIU
        log!("> ADDIU");
        compute!(rt = rs wrapping_add signed imm16)
      },
      0x0A => {
        //SLTI
        log!("> SLTI");
        compute!(rt = rs signed_compare imm16)
      },
      0x0B => {
        //SLTIU
        log!("> SLTIU");
        compute!(rt = rs compare imm16)
      },
      0x0C => {
        //ANDI
        log!("> ANDI");
        compute!(rt = rs and imm16)
      },
      0x0D => {
        //ORI
        log!("> ORI");
        compute!(rt = rs or imm16)
      },
      0x0E => {
        //XORI
        log!("> XORI");
        compute!(rt = rs xor imm16)
      },
      0x0F => {
        //LUI
        log!("> LUI");
        compute!(rt = imm16 shl 16)
      },
      0x10 => {
        //COP0
        log!("> COP0");
        cop!(cop0)
      },
      0x11 => {
        //COP1
        unreachable!("COP1 is not implemented on the PSX")
      },
      0x12 => {
        //COP2
        log!("> COP2");
        cop!(gte)
      },
      0x13 => {
        //COP3
        unreachable!("COP3 is not implemented on the PSX")
      },
      0x20 => {
        //LB
        log!("> LB");
        mov!(rt = [rs + imm16] read_byte_sign_extended)
      },
      0x21 => {
        //LH
        log!("> LH");
        mov!(rt = [rs + imm16] read_half_sign_extended)
      },
      0x22 => {
        //LWL
        log!("> LWL");
        mov!(rt = [rs + imm16] left)
      },
      0x23 => {
        //LW
        log!("> LW");
        mov!(rt = [rs + imm16] read_word)
      },
      0x24 => {
        //LBU
        log!("> LBU");
        mov!(rt = [rs + imm16] read_byte)
      },
      0x25 => {
        //LHU
        log!("> LHU");
        mov!(rt = [rs + imm16] read_half)
      },
      0x26 => {
        //LWR
        log!("> LWR");
        mov!(rt = [rs + imm16] right)
      },
      0x28 => {
        //SB
        log!("> SB");
        mov!([rs + imm16] = rt write_byte)
      },
      0x29 => {
        //SH
        log!("> SH");
        mov!([rs + imm16] = rt write_half)
      },
      0x2A => {
        //SWL
        log!("> SWL");
        mov!([rs + imm16] = rt left)
      },
      0x2B => {
        //SW
        log!("> SW");
        mov!([rs + imm16] = rt write_word)
      },
      0x2E => {
        //SWR
        log!("> SWR");
        mov!([rs + imm16] = rt right)
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

