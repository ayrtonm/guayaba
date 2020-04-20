use crate::interpreter::Interpreter;
use crate::register::Register;
use crate::dma::Transfer;
use crate::dma::Direction;
use crate::dma::Chunks;
use crate::dma::Blocks;
use crate::dma::Step;
use crate::dma::DMAChannel;

impl Interpreter{
  pub(super) fn handle_dma(&mut self, transfer: Transfer) {
    let addr_mask = 0x00ff_fffc;
    let step = |address: Register| {
      match transfer.step() {
        Step::Forward => address.wrapping_add(4) & addr_mask,
        Step::Backward => address.wrapping_sub(4) & addr_mask,
      }
    };
    let undo_step = |address: Register| {
      match transfer.step() {
        Step::Forward => address.wrapping_sub(4) & addr_mask,
        Step::Backward => address.wrapping_add(4) & addr_mask,
      }
    };
    let mut addr = transfer.start_address() & addr_mask;
    match transfer.direction() {
      Direction::ToRAM => {
        match transfer.channel_num() {
          6 => {
            match transfer.chunks() {
              Chunks::NumWords(num) => {
                for i in 1..=*num {
                  let remaining = *num - i;
                  let data = match remaining {
                    0 => 0x00ff_ffff,
                    _ => addr.wrapping_sub(4) & addr_mask,
                  };
                  self.memory.write_word(addr, data);
                  addr = step(addr);
                }
              },
              _ => {
                todo!("implement DMA {:#x?}", transfer)
              },
            }
          },
          _ => {
            todo!("implement DMA {:#x?}", transfer)
          },
        }
      },
      Direction::FromRAM => {
        let channel_available = self.get_dma_channel(transfer.channel_num()).is_some();
        if channel_available {
          let mut buffer = Vec::new();
          match transfer.chunks() {
            Chunks::NumWords(num) => {
              for _ in 1..=*num {
                let data = self.resolve_memresponse(self.memory.read_word(addr));
                buffer.push(data);
                addr = step(addr);
              }
            },
            Chunks::Blocks(blocks) => {
              let packet_size = blocks.num_blocks() *  blocks.block_size();
              for _ in 1..=packet_size {
                let data = self.resolve_memresponse(self.memory.read_word(addr));
                buffer.push(data);
                addr = step(addr);
              }
              addr = undo_step(addr);
              self.memory.write_word(0x1f80_1080 + (transfer.channel_num() * 0x10), addr);
            },
            Chunks::LinkedList => {
              let mut header_address = addr;
              loop {
                let header = self.resolve_memresponse(self.memory.read_word(header_address));
                let packet_size = header >> 24;
                for _ in 1..=packet_size {
                  addr = step(addr);
                  let data = self.resolve_memresponse(self.memory.read_word(addr));
                  buffer.push(data);
                }
                let next_packet = header & 0x00ff_ffff;
                if next_packet == 0x00ff_ffff {
                  break
                } else {
                  header_address = next_packet & addr_mask;
                }
              }
              self.memory.write_word(0x1f80_1080 + (transfer.channel_num() * 0x10), 0x00ff_ffff);
            },
          }
          println!("sending {:#x?} through a DMA", buffer);
          self.get_dma_channel(transfer.channel_num())
              .map(|channel| channel.send(buffer));
        }
      },
    }
    self.memory.reset_dma_channel(transfer.channel_num());
  }
  fn get_dma_channel(&mut self, channel_num: u32) -> Option<&mut dyn DMAChannel> {
    match channel_num {
      2 => {
        Some(&mut self.gpu)
      },
      3 => {
        self.cd.as_mut().map(|cd| cd as &mut dyn DMAChannel)
      },
      6 => {
        Some(&mut self.memory)
      },
      _ => {
        todo!("implement get_dma_channel() for channel num {}", channel_num)
      },
    }
  }
}
