use crate::gpu::GPU;
use crate::gpu::Command;
use crate::register::Register;
use crate::register::BitManipulation;
use crate::screen::Drawable;

impl GPU {
  pub fn exec_next_gp0_command(&mut self) -> Option<Drawable> {
    let command = self.command_buffer.pop_front();
    match command {
      Some(command) => {
        match command.id() {
          0x00 => {
          },
          0x01 => {
          },
          0x04..=0x1e | 0xe0 | 0xe7..=0xef => {
          },
          0x28 => {
            if self.logging {
              println!("rendered an opaque monochrome four-point polygon");
            }
            let xpos = command.as_ref()
                              .iter()
                              .skip(1)
                              .map(|yx| yx.lowest_bits(16) as i16)
                              .collect::<Vec<i16>>();
            let ypos = command.as_ref()
                              .iter()
                              .skip(1)
                              .map(|yx| (yx >> 16) as i16)
                              .collect::<Vec<i16>>();
            let xmax = 1023;
            let ymax = 511;
            let xdiff = xpos.iter().max().unwrap() - xpos.iter().min().unwrap();
            let ydiff = ypos.iter().max().unwrap() - ypos.iter().min().unwrap();
            if xdiff < xmax && ydiff < ymax {
              let monochrome = command.as_ref()
                                      .iter()
                                      .take(1)
                                      .enumerate()
                                      .map(|(i, v)| v >> (8 * i))
                                      .map(|c| c.lowest_bits(8) as i16)
                                      .cycle()
                                      .take(4)
                                      .collect::<Vec<i16>>();
              let positions = xpos.into_iter()
                                  .zip(ypos.into_iter())
                                  .map(|(a,b)| vec![a,b])
                                  .flatten()
                                  .collect();
              return Some(Drawable::new(positions, monochrome));
            }
          },
          0x30 => {
          },
          0x38 => {
            if self.logging {
              println!("rendered an shaded four-point polygon");
            }
            let xpos = command.as_ref()
                              .iter()
                              .skip(1)
                              .step_by(2)
                              .map(|yx| yx.lowest_bits(16) as i16)
                              .collect::<Vec<i16>>();
            let ypos = command.as_ref()
                              .iter()
                              .skip(1)
                              .step_by(2)
                              .map(|yx| (yx >> 16) as i16)
                              .collect::<Vec<i16>>();
            let xmax = 1023;
            let ymax = 511;
            let xdiff = xpos.iter().max().unwrap() - xpos.iter().min().unwrap();
            let ydiff = ypos.iter().max().unwrap() - ypos.iter().min().unwrap();
            if xdiff < xmax && ydiff < ymax {
              let cols = command.as_ref()
                                .iter()
                                .step_by(2)
                                .map(|col| {
                                  vec![col].into_iter()
                                           .cycle()
                                           .take(3)
                                           .enumerate()
                                           .map(|(i, c)| c >> (8 * i))
                                           .map(|c| c.lowest_bits(8) as i16)
                                           .collect::<Vec<i16>>()
                                })
                                .flatten()
                                .collect::<Vec<i16>>();
              let positions = xpos.into_iter()
                                  .zip(ypos.into_iter())
                                  .map(|(a,b)| vec![a,b])
                                  .flatten()
                                  .collect();
              return Some(Drawable::new(positions, cols));
            }
          },
          0xa0 => {
            if self.logging {
              println!("copy rectangle to VRAM");
            }
          },
          0xc0 => {
            if self.logging {
              println!("copy rectangle from VRAM");
            }
          },
          0xe1 => {
            let mask = 0x0000_83ff;
            let command = command.idx(0) & mask;
            self.gpustat.as_mut().clear_mask(mask).set_mask(command);
          },
          0xe2 => {
            let command = command.idx(0);
            self.texture_mask_x = command.lowest_bits(5);
            self.texture_mask_y = (command >> 5).lowest_bits(5);
            self.texture_offset_x = (command >> 10).lowest_bits(5);
            self.texture_offset_y = (command >> 15).lowest_bits(5);
          },
          0xe3 => {
            let command = command.idx(0);
            self.drawing_min_x = command.lowest_bits(10);
            self.drawing_min_y = (command >> 10).lowest_bits(9);
          },
          0xe4 => {
            let command = command.idx(0);
            self.drawing_max_x = command.lowest_bits(10);
            self.drawing_max_y = (command >> 10).lowest_bits(9);
          },
          0xe5 => {
            let command = command.idx(0);
            self.drawing_offset_x = command.lowest_bits(11);
            self.drawing_offset_y = (command >> 11).lowest_bits(11);
          },
          0xe6 => {
            let command = command.idx(0);
            let mask = command.lowest_bits(2) << 11;
            self.gpustat.as_mut().clear(11).clear(12).set_mask(mask);
          },
          _ => {
            todo!("implement this GP0 command {:#x}", command.id());
          },
        }
      },
      None => {
      },
    }
    None
  }
  pub fn write_to_gp0(&mut self, value: Register) {
    //println!("GP0 received {:#x}", value);
    let cmd = match self.waiting_for_parameters {
      true => {
        let mut cmd = self.partial_command.take()
                                          .expect("Expected a partial command in the GPU");
        cmd.append_parameters(value);
        cmd
      },
      false => {
        Command::new(value)
      },
    };
    self.try_push_command(cmd);
  }
  fn try_push_command(&mut self, cmd: Command) {
    match cmd.completed() {
      true => {
        if self.logging {
          println!("GP0 received command {:#x?}", cmd);
        }
        self.command_buffer.push_back(cmd);
        self.waiting_for_parameters = false;
      },
      false => {
        self.partial_command = Some(cmd);
        self.waiting_for_parameters = true;
      },
    }
  }
  fn filled_buffer(&self) -> usize {
    self.command_buffer.iter().fold(0, |acc, command| acc + command.num_words())
  }
}
