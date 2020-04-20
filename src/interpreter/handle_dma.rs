use crate::interpreter::Interpreter;
use crate::dma::Transfer;
use crate::dma::Direction;
use crate::dma::Chunks;
use crate::dma::Blocks;
use crate::dma::Step;
use crate::dma::DMAChannel;

impl Interpreter{
  pub(super) fn handle_dma(&mut self, transfer: Transfer) {
    let next_address = |increment| match transfer.step() {
      Step::Forward => transfer.start_address().wrapping_add(increment * 4),
      Step::Backward => transfer.start_address().wrapping_sub(increment * 4),
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
                  self.memory.write_word(addr, data);
                  addr = next_address(i) & 0x001f_fffc;
                }
                self.memory.reset_dma_channel(6);
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
        todo!("implement DMA from RAM {:#x?}", transfer)
      },
    }
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
