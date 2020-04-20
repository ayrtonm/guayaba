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
    let step = |address: Register| {
      match transfer.step() {
        Step::Forward => address.wrapping_add(4) & 0x001f_fffc,
        Step::Backward => address.wrapping_sub(4) & 0x001f_fffc,
      }
    };
    let mut addr = transfer.start_address() & 0x001f_fffc;
    match transfer.direction() {
      Direction::ToRAM => {
        match transfer.channel() {
          6 => {
            match transfer.chunks() {
              Chunks::NumWords(num) => {
                for i in 1..=*num {
                  let remaining = *num - i;
                  let data = match remaining {
                    0 => 0x00ff_ffff,
                    _ => addr.wrapping_sub(4) & 0x001f_fffc,
                  };
                  let action = self.memory.write_word(addr, data);
                  self.resolve_memaction(action);
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
        match transfer.channel() {
          2 => {
            match transfer.chunks() {
              Chunks::LinkedList => {
                let mut buffer = Vec::new();
                let mut header_address = addr;
                loop {
                  let header = self.resolve_memresponse(self.memory.read_word(header_address));
                  let packet_size = header >> 24;
                  for i in 1..=packet_size {
                    addr = step(addr);
                    let data = self.resolve_memresponse(self.memory.read_word(addr));
                    buffer.push(data);
                  }
                  let next_packet = header & 0x00ff_ffff;
                  if next_packet == 0x00ff_ffff {
                    break
                  } else {
                    header_address = next_packet & 0x001f_fffc;
                  }
                }
                self.memory.write_word(0x1f80_1080 + (transfer.channel() * 0x10), 0x00ff_ffff);
                self.get_dma_channel(transfer.channel())
                    .map(|channel| channel.send(buffer));
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
    }
    self.memory.reset_dma_channel(transfer.channel());
  }
  fn get_dma_channel(&mut self, channel: u32) -> Option<&mut dyn DMAChannel> {
    match channel {
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
        todo!("implement get_dma_channel() for channel {}", channel)
      },
    }
  }
}
