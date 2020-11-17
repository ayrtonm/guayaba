use crate::register::BitTwiddle;

#[derive(Debug)]
pub struct Command(Vec<u32>);

impl Command {
    pub fn new(cmd: u32) -> Self {
        Command(vec![cmd])
    }

    pub fn id(&self) -> u8 {
        self.0[0].upper_bits(8) as u8
    }

    pub fn idx(&self, n: usize) -> u32 {
        self.0[n]
    }

    pub fn as_ref(&self) -> &Vec<u32> {
        &self.0
    }

    pub fn num_words(&self) -> usize {
        self.0.len()
    }

    pub fn append_parameters(&mut self, parameters: u32) {
        self.0.push(parameters);
    }

    pub fn get_monochrome(&self) -> Vec<i16> {
        self.as_ref()
            .iter()
            .take(1)
            .enumerate()
            .map(|(i, v)| v >> (8 * i))
            .map(|c| c.lowest_bits(8) as i16)
            .cycle()
            .take(4)
            .collect()
    }

    pub fn get_colors(&self) -> Vec<i16> {
        self.as_ref()
            .iter()
            .step_by(2)
            .map(|col| {
                vec![col]
                    .into_iter()
                    .cycle()
                    .take(3)
                    .enumerate()
                    .map(|(i, c)| c >> (8 * i))
                    .map(|c| c.lowest_bits(8) as i16)
                    .collect::<Vec<i16>>()
            })
            .flatten()
            .collect()
    }

    pub fn get_xpos_consecutive(&self) -> Vec<i16> {
        self.as_ref()
            .iter()
            .skip(1)
            .map(|yx| yx.half() as i16)
            .collect()
    }

    pub fn get_ypos_consecutive(&self) -> Vec<i16> {
        self.as_ref()
            .iter()
            .skip(1)
            .map(|yx| yx.upper_bits(16) as i16)
            .collect()
    }

    pub fn get_xpos_every_other(&self) -> Vec<i16> {
        self.as_ref()
            .iter()
            .skip(1)
            .step_by(2)
            .map(|yx| yx.half() as i16)
            .collect()
    }

    pub fn get_ypos_every_other(&self) -> Vec<i16> {
        self.as_ref()
            .iter()
            .skip(1)
            .step_by(2)
            .map(|yx| yx.upper_bits(16) as i16)
            .collect()
    }

    pub fn get_xpos_copy(&self, idx: usize) -> u32 {
        self.idx(idx).half() & 0x3ff
    }

    pub fn get_ypos_copy(&self, idx: usize) -> u32 {
        self.idx(idx).upper_bits(16) & 0x1ff
    }

    pub fn get_xsize_copy(&self, idx: usize) -> u32 {
        ((self.idx(idx).half() - 1) & 0x3ff) + 1
    }

    pub fn get_ysize_copy(&self, idx: usize) -> u32 {
        (((self.idx(idx) >> 16) - 1) & 0x3ff) + 1
    }

    pub fn completed(&self) -> bool {
        match self.id() {
            0xe1 | 0xe2 | 0xe3 | 0xe4 | 0xe5 | 0xe6 | 0x01 | 0x1f => self.num_words() == 1,
            0x68 | 0x6a | 0x70 | 0x72 | 0x78 | 0x7a => self.num_words() == 2,
            0x6c | 0x6d | 0x6e | 0x6f | 0x74 | 0x75 | 0x76 | 0x77 | 0x7c | 0x7d | 0x7e | 0x7f |
            0x60 | 0x62 | 0x40 | 0x42 | 0x02 => self.num_words() == 3,
            0x20 | 0x22 | 0x64 | 0x65 | 0x66 | 0x67 | 0x80..=0x9f | 0x50 | 0x52 => {
                self.num_words() == 4
            },
            0x28 | 0x2a => self.num_words() == 5,
            0x30 | 0x32 => self.num_words() == 6,
            0x24 | 0x25 | 0x26 | 0x27 => self.num_words() == 7,
            0x38 | 0x3a => self.num_words() == 8,
            0x2C | 0x2D | 0x2E | 0x2F | 0x34 | 0x36 => self.num_words() == 9,
            0x3c | 0x3e => self.num_words() == 12,
            0x48 | 0x4a | 0x58 | 0x5a => {
                (self.num_words() >= 4) && self.0.iter().rev().take(1).all(|&p| p == 0x55555555)
            },
            0xa0..=0xbf => {
                if self.num_words() < 3 {
                    false
                } else {
                    //xsize and ysize are measured in halfwords
                    let ysize = self.0[2].upper_bits(16);
                    let xsize = self.0[2].half();
                    //paramter length is in bytes
                    let num_words = *((xsize * ysize) + 1).clear(0) >> 1;
                    self.num_words() == 3 + num_words as usize
                }
            },
            0xc0..=0xdf => self.num_words() == 3,
            0x00 | 04..=0x1e | 0xe0 | 0xe7..=0xef => true,
            _ => todo!("implement this GP0 command {:x}", self.id()),
        }
    }
}
