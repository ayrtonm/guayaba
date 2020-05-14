use crate::common::ReadArray;
use crate::common::WriteArray;
use crate::memory::Memory;
use crate::register::Register;
use crate::register::BitBang;
use crate::dma::Transfer;
use crate::dma::Chunks;
use crate::dma::Blocks;
use crate::dma::Direction;
use crate::dma::Step;

macro_rules! identifier_size {
  (read_word) => {
    {
      SizeIdentifier::Word
    }
  };
  (read_half) => {
    {
      SizeIdentifier::Half
    }
  };
  (read_byte) => {
    {
      SizeIdentifier::Byte
    }
  };
  (write_word) => {
    {
      SizeIdentifier::Word
    }
  };
  (write_half) => {
    {
      SizeIdentifier::Half
    }
  };
  (write_byte) => {
    {
      SizeIdentifier::Byte
    }
  };
}

#[macro_export]
macro_rules! get_io_response {
  ($address:expr, $function:ident, $self:expr) => {
    {
      let aligned_address = $address & 0xffff_fffc;
      let aligned_offset = aligned_address - Memory::IO_PORTS;
      let offset = $address - Memory::IO_PORTS;
      let value = $self.io_ports.as_ref().$function(offset);
      match aligned_address {
        Memory::INTERRUPT_STAT => {
          //println!("read interrupt stat");
          MemResponse::Value(value)
        },
        Memory::INTERRUPT_MASK => {
          //println!("read interrupt mask");
          MemResponse::Value(value)
        },
        Memory::TIMER_VALUE_0 => {
          //println!("read timer value 0");
          MemResponse::Value(value)
        },
        Memory::TIMER_MODE_0 => {
          //println!("read timer mode 0");
          MemResponse::Value(value)
        },
        Memory::TIMER_TARGET_0 => {
          //println!("read timer target 0");
          MemResponse::Value(value)
        },
        Memory::TIMER_VALUE_1 => {
          //println!("read timer value 1");
          MemResponse::Value(value)
        },
        Memory::TIMER_MODE_1 => {
          //println!("read timemode r 1");
          MemResponse::Value(value)
        },
        Memory::TIMER_TARGET_1 => {
          //println!("read timer target 1");
          MemResponse::Value(value)
        },
        Memory::TIMER_VALUE_2 => {
          //println!("read timer value 2");
          MemResponse::Value(value)
        },
        Memory::TIMER_MODE_2 => {
          //println!("read timer mode 2");
          MemResponse::Value(value)
        },
        Memory::TIMER_TARGET_2 => {
          //println!("read timer target 2");
          MemResponse::Value(value)
        },
        Memory::CD_PORT => {
          let index = $self.io_ports.as_ref().read_byte(aligned_offset).lowest_bits(2);
          let ret = match $address.lowest_bits(2) {
            //could read word, half, byte
            0 => {
              MemResponse::Value(*value.clone().clear_mask(0x0000_00fc).set(3).set(4))
              //match identifier_size!($function) {
              //  SizeIdentifier::Word => {
              //  },
              //  SizeIdentifier::Half => {
              //  },
              //  SizeIdentifier::Byte => {
              //  },
              //}
            },
            //could read byte
            1 => {
              MemResponse::CDResponse
            },
            //could read half, byte
            2 => {
              MemResponse::Value(value)
            },
            //could read byte
            3 => {
              MemResponse::Value(value)
            },
            _ => {
              unreachable!("");
            },
          };
          //println!("CD {} {:x?} from {:#x}", stringify!($function), ret, $address);
          ret
        },
        Memory::GPU_GP0 => {
          MemResponse::GPUREAD
        },
        Memory::GPU_GP1 => {
          MemResponse::GPUSTAT
        },
        _ => {
          MemResponse::Value(value)
        },
      }
    }
  };
}

#[macro_export]
macro_rules! get_io_action {
  ($address:expr, $value:expr, $function:ident, $self:expr) => {
    {
      let aligned_address = $address & 0xffff_fffc;
      let aligned_offset = aligned_address - Memory::IO_PORTS;
      match aligned_address {
        //Memory::INTERRUPT_STAT => {
        //  //check for interrupt requests
        //  let mut irq = Vec::new();
        //  let new_interrupt_stat = $self.io_ports.as_ref().read_half(aligned_offset);
        //  let interrupt_mask = $self.io_ports.as_ref().read_half(Memory::INTERRUPT_MASK - Memory::IO_PORTS);
        //  for i in 0..=10 {
        //    if !$self.old_interrupt_stat.nth_bit_bool(i) &&
        //      new_interrupt_stat.nth_bit_bool(i) &&
        //      interrupt_mask.nth_bit_bool(i) {
        //      irq.push(MemAction::Interrupt(i));
        //    }
        //  }
        //  $self.old_interrupt_stat = $self.io_ports.as_ref().read_half(aligned_offset);
        //  //println!("wrote {:#x} to interrupt stat", $value);
        //  match irq.len() {
        //    0 => {
        //      None
        //    },
        //    _ => {
        //      Some(irq)
        //    },
        //  }
        //},
        Memory::INTERRUPT_MASK => {
          $self.io_ports.as_mut()
                        .write_word(aligned_offset,
                                    *$value.clone()
                                           .clear(11)
                                           .clear(12)
                                           .clear(13)
                                           .clear(14)
                                           .clear(15));
          //println!("wrote {:#x} to interrupt mask", $value);
          MemAction::None
        },
        Memory::TIMER_VALUE_0 => {
          //println!("wrote {:#x} to timer value 0", $value);
          MemAction::None
        },
        Memory::TIMER_MODE_0 => {
          //println!("wrote {:#x} to timer mode 0", $value);
          MemAction::None
        },
        Memory::TIMER_TARGET_0 => {
          //println!("wrote {:#x} to timer target 0", $value);
          MemAction::None
        },
        Memory::TIMER_VALUE_1 => {
          //println!("wrote {:#x} to timer value 1", $value);
          MemAction::None
        },
        Memory::TIMER_MODE_1 => {
          //println!("wrote {:#x} to timemode r 1", $value);
          MemAction::None
        },
        Memory::TIMER_TARGET_1 => {
          //println!("wrote {:#x} to timer target 1", $value);
          MemAction::None
        },
        Memory::TIMER_VALUE_2 => {
          //println!("wrote {:#x} to timer value 2", $value);
          MemAction::None
        },
        Memory::TIMER_MODE_2 => {
          //println!("wrote {:#x} to timer mode 2", $value);
          MemAction::None
        },
        Memory::TIMER_TARGET_2 => {
          //println!("wrote {:#x} to timer target 2", $value);
          MemAction::None
        },
        Memory::CD_PORT => {
          //println!("CD {} {:#x} to {:#x}", stringify!($function), $value, $address);
          let index = $self.io_ports.as_ref().read_byte(aligned_offset).lowest_bits(2);
          match $address.lowest_bits(2) {
            //could write word, half or byte
            //1800, 1801, 1802, 1803
            0 => {
              match identifier_size!($function) {
                SizeIdentifier::Word => {
                  MemAction::None
                  //Some(vec![
                  //  MemAction::CDCmd(
                  //    $self.io_ports.as_ref().read_byte(aligned_offset + 1) as u8),
                  //  MemAction::CDParam(
                  //    $self.io_ports.as_ref().read_byte(aligned_offset + 2) as u8)
                  //  ]
                  //)
                },
                SizeIdentifier::Half => {
                    MemAction::CDCmd(
                      $self.io_ports.as_ref().read_byte(aligned_offset + 1) as u8)
                },
                SizeIdentifier::Byte => {
                  MemAction::None
                },
              }
            },
            //could write byte
            1 => {
              match index {
                0 => {
                    MemAction::CDCmd(
                      $self.io_ports.as_ref().read_byte(aligned_offset + 1) as u8)
                },
                1 => {
                  MemAction::None
                },
                2 => {
                  MemAction::None
                },
                3 => {
                  MemAction::None
                },
                _ => {
                  unreachable!("");
                },
              }
            },
            //could write half or byte
            //1802, 1803
            2 => {
              match index {
                0 => {
                    MemAction::CDParam(
                      $self.io_ports.as_ref().read_byte(aligned_offset + 2) as u8)
                },
                1 => {
                  MemAction::None
                },
                2 => {
                  MemAction::None
                },
                3 => {
                  MemAction::None
                },
                _ => {
                  unreachable!("");
                },
              }
            },
            //could write byte
            //1803
            3 => {
              match index {
                0 => {
                  MemAction::None
                },
                1 => {
                  MemAction::None
                },
                2 => {
                  MemAction::None
                },
                3 => {
                  MemAction::None
                },
                _ => {
                  unreachable!("");
                },
              }
            },
            _ => {
              unreachable!("");
            },
          }
        },
        Memory::GPU_GP0 => {
            MemAction::GpuGp0(
              $self.io_ports.as_ref().read_word(aligned_offset))
        },
        Memory::GPU_GP1 => {
            MemAction::GpuGp1(
              $self.io_ports.as_ref().read_word(aligned_offset))
        },
        Memory::DMA_CHANNEL_0 | Memory::DMA_CHANNEL_1 | Memory::DMA_CHANNEL_2 |
        Memory::DMA_CHANNEL_3 | Memory::DMA_CHANNEL_4 | Memory::DMA_CHANNEL_5 |
        Memory::DMA_CHANNEL_6 => {
          let channel_num = (aligned_address - Memory::DMA_CHANNEL_0) as u8 >> 4;
          let control_register = $self.io_ports.as_ref()
                                               .read_word(aligned_offset);
          let sync_mode = control_register.sync_mode();
          if control_register.nth_bit_bool(24) {
            match sync_mode {
              0 => {
                if control_register.nth_bit_bool(28) {
                    MemAction::DMA($self.create_dma_transfer(channel_num))
                } else {
                  MemAction::None
                }
              },
              1 | 2 => {
                  MemAction::DMA($self.create_dma_transfer(channel_num))
              },
              _ => unreachable!("DMA channel {} is not configured properly", channel_num),
            }
          } else {
            MemAction::None
          }
        },
        //DMA interrupt register
        0x1f80_10f4 => {
          MemAction::None
        },
        _ => {
          //println!("unhandled IO port {} {:#x} at {:#x}", stringify!($function), $value, $address);
          MemAction::None
        },
      }
    }
  };
}

impl Memory {
  pub fn reset_dma_channel(&mut self, channel: u8) {
    let address = Memory::DMA_CHANNEL_0 + ((channel as u32) * 0x10) - Memory::IO_PORTS;
    let mut control_register = self.io_ports.as_ref().read_word(address);
    let new_register = *control_register.clear(28).clear(24);
    self.io_ports.as_mut().write_word(address, new_register);
  }
  //pack the current state of I/O ports into a Transfer struct for a given channel
  pub(super) fn create_dma_transfer(&mut self, channel: u8) -> Transfer {
    assert!(channel < 7);
    //these are addresses to locations in memory
    let base_addr = 0x0000_0080 + ((channel as u32) * 0x0000_0010);
    let block_control = base_addr + 4;
    let channel_control = block_control + 4;
  
    //these are the values of locations in memory
    let start_address = self.io_ports.as_ref().read_word(base_addr) & 0x00ff_fffc;
    let block_control = self.io_ports.as_ref().read_word(block_control);
    let control_register = self.io_ports.as_ref().read_word(channel_control);
    let sync_mode = control_register.sync_mode();
    let direction = match control_register.nth_bit_bool(0) {
      false => Direction::ToRAM,
      true => Direction::FromRAM,
    };
    let step = match control_register.nth_bit_bool(1) {
      false => Step::Forward,
      true => Step::Backward,
    };
    let chunks = match sync_mode {
      0 => {
        let words = block_control & 0x0000_ffff;
        Chunks::NumWords(match words {
          0 => 0x0001_0000,
          _ => words,
        })
      },
      1 => {
        let size = block_control & 0x0000_ffff;
        let amount = block_control >> 16;
        let max_size = match channel {
          0 => 0x20,
          1 => 0x20,
          2 => 0x10,
          4 => 0x10,
          _ => unreachable!("DMA channel {} is not configured properly", channel),
        };
        Chunks::Blocks(
          Blocks::new(
            if size < max_size {
              size
            } else {
              max_size
            } as u16,
            amount as u16
          )
        )
      },
      2 => Chunks::LinkedList,
      3 => unreachable!("DMA channel {} is not configured properly", channel),
      _ => unreachable!("DMA channel {} is not configured properly", channel),
    };
    Transfer::new(channel, start_address, chunks, direction, step, sync_mode)
  }
}

pub trait DMAControl {
  fn sync_mode(&self) -> u8;
}

impl DMAControl for Register {
  fn sync_mode(&self) -> u8 {
    self.upper_bits(23) as u8 & 3
  }
}
