use crate::register::Register;

#[derive(Debug)]
pub struct Command {
  id: u8,
  parameters: Vec<u8>,
}

impl Command {
  pub fn new(cmd: u32) -> Self {
    let param1 = ((cmd >> 16) & 0x0000_00ff) as u8;
    let param2 = ((cmd >> 8) & 0x0000_00ff) as u8;
    let param3 = (cmd & 0x0000_00ff) as u8;
    Command {
      id: (cmd >> 24) as u8,
      parameters: vec![param1, param2, param3],
    }
  }
  pub fn serialize(&self) -> Register {
    assert!(self.parameters.len() == 3, "{:#x?}", self);
    ((self.id as Register) << 24) |
    ((self.parameters[0] as Register) << 16) |
    ((self.parameters[1] as Register) << 8) |
    (self.parameters[2] as Register)
  }
  pub fn id(&self) -> u8 {
    self.id
  }
  pub fn append_parameters(&mut self, param: u32) {
    let param1 = (param >> 24) as u8;
    let param2 = ((param >> 16) & 0x0000_00ff) as u8;
    let param3 = ((param >> 8) & 0x0000_00ff) as u8;
    let param4 = (param & 0x0000_00ff) as u8;
    self.parameters.push(param1);
    self.parameters.push(param2);
    self.parameters.push(param3);
    self.parameters.push(param4);
  }
  pub fn num_bytes(&self) -> usize {
    1 + self.parameters.len()
  }
  pub fn completed(&self) -> bool {
    match self.id {
      0x20 | 0x22 => {
        self.parameters.len() == 15
      },
      0x28 | 0x2a => {
        self.parameters.len() == 19
      },
      0x24 | 0x25 | 0x26 | 0x27 => {
        self.parameters.len() == 27
      },
      0x2C | 0x2D | 0x2E | 0x2F => {
        self.parameters.len() == 35
      },
      0x30 | 0x32 => {
        self.parameters.len() == 23
      },
      0x38 | 0x3a => {
        self.parameters.len() == 31
      },
      0x34 | 0x36 => {
        self.parameters.len() == 35
      },
      0x3c | 0x3e => {
        self.parameters.len() == 47
      },
      0x40 | 0x42 => {
        self.parameters.len() == 11
      },
      0x48 | 0x4a | 0x58 | 0x5a => {
        (self.parameters.len() >= 15) &&
        self.parameters.iter()
                       .rev()
                       .take(4)
                       .all(|&p| p == 0x55)
      },
      0x60 | 0x62 => {
        self.parameters.len() == 11
      },
      0x68 | 0x6a | 0x70 | 0x72 | 0x78 | 0x7a => {
        self.parameters.len() == 7
      },
      0x64 | 0x65 | 0x66 | 0x67 => {
        self.parameters.len() == 15
      },
      0x6c | 0x6d | 0x6e | 0x6f |
      0x74 | 0x75 | 0x76 | 0x77 |
      0x7c | 0x7d | 0x7e | 0x7f => {
        self.parameters.len() == 11
      },
      0xe1 | 0xe2 | 0xe3 | 0xe4 | 0xe5 | 0xe6 | 0x01 => {
        self.parameters.len() == 3
      },
      0x02 => {
        self.parameters.len() == 11
      },
      0x80..=0x9f => {
        self.parameters.len() == 15
      },
      0xa0..=0xbf => {
        //this is some function of the parameters
        if self.parameters.len() < 11 {
          false
        } else {
          //xsize and ysize are measured in halfwords
          let xsize = (self.parameters[10] as u32) + ((self.parameters[9] as u32) << 8);
          let ysize = (self.parameters[8] as u32) + ((self.parameters[7] as u32) << 8);
          //paramter length is in bytes
          let num_words = ((xsize as u64) * (ysize as u64)) << 1;
          self.parameters.len() >= num_words as usize
        }
      },
      0xc0..=0xdf => {
        //this is some function of the parameters
        todo!("implement this GPU command {:x}", self.id)
      },
      0x1f => {
        self.parameters.len() == 3
      },
      0x00 | 04..=0x1e | 0xe0 | 0xe7..=0xef => {
        true
      },
      0x50 | 0x52 => {
        self.parameters.len() == 15
      },
      _ => {
        todo!("implement this GPU command {:x}", self.id)
      },
    }
  }
}


