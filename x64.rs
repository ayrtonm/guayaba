#![feature(asm)]
use std::io;
use memmap::{Mmap, MmapMut};

struct Toy {
  f: fn(u32) -> u32,
}

impl Toy {
  pub fn new(f: fn(u32) -> u32) -> Self {
    Toy {
      f
    }
  }
  pub fn run(&self, x: u32) -> u32 {
    (self.f)(x)
  }
}

fn main() -> io::Result<()> {
  let mut buffer = Vec::new();
  buffer.push(0x48);
  buffer.push(0x8b);
  buffer.push(0xc7);
  buffer.push(0xc3);
  let mut mmap = MmapMut::map_anon(buffer.len())?;
  mmap.copy_from_slice(&buffer);
  let mmap: Mmap = mmap.make_exec()?;
  let addr = mmap.as_ptr();
  unsafe {
    let f = std::mem::transmute::<*const u8, fn(u32) -> u32>(addr);
    let toy = Toy::new(f);
    for i in 0..=10 {
      println!("{}", toy.run(i));
    }
  };
  Ok(())
}
