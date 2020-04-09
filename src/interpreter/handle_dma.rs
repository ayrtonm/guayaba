use crate::interpreter::Interpreter;
use crate::dma::Transfer;
use crate::dma::Direction;
use crate::dma::Chunks;
use crate::dma::Blocks;
use crate::dma::Step;
use crate::dma::DMAChannel;

impl Interpreter{
  pub(super) fn handle_dma(&mut self, transfers: Vec<Transfer>) {
    transfers.iter().for_each(
      |transfer| {
        match transfer.chunks() {
          Chunks::NumWords(num) => {
          },
          Chunks::Blocks(blocks) => {
          },
          Chunks::LinkedList => {
            let mut buffer = Vec::new();
            let mut current_address = transfer.start_address();
            loop {
              let header = self.resolve_memresponse(self.memory.read_word(current_address));
              let packet_size = header >> 24;
              let operation = |addr| match transfer.step() {
                Step::Forward => current_address.wrapping_add(addr),
                Step::Backward => current_address.wrapping_sub(addr),
              };
              for i in 1..=packet_size {
                let next_addr = operation(i *  4) & 0x001f_fffc;
                let data = self.resolve_memresponse(self.memory.read_word(next_addr));
                buffer.push(data);
              }
              let next_header_addr = header & 0x00ff_ffff;
              if next_header_addr == 0x00ff_ffff {
                break
              } else {
                current_address = next_header_addr & 0x001f_fffc;
              }
            }
            let mut channel = self.get_dma_channel(transfer.channel());
            channel.map(|channel| channel.send(buffer));
            println!("sent data on DMA channel {}", transfer.channel());
          },
          _ => {
          },
        }
      }
    )
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
