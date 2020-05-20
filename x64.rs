#![feature(asm)]
use std::io;
use memmap::{Mmap, MmapMut};

fn main() -> io::Result<()> {
  let mut buffer = Vec::new();
  let toy = Toy::new();
  let toy2 = Toy::new();
  let func_addr: u64 = Toy::g as fn(&mut Toy) as u64;
  let toy_addr: u64 = &toy as *const Toy as u64;
  let toy2_addr: u64 = &toy2 as *const Toy as u64;
  ////mov rcx, func
  buffer.push(0x48); //REX.W prefix
  buffer.push(0xb8);
  buffer.push((func_addr & 0xff) as u8);
  buffer.push(((func_addr >> 8) & 0xff) as u8);
  buffer.push(((func_addr >> 16) & 0xff) as u8);
  buffer.push(((func_addr >> 24) & 0xff) as u8);
  buffer.push(((func_addr >> 32) & 0xff) as u8);
  buffer.push(((func_addr >> 40) & 0xff) as u8);
  buffer.push(((func_addr >> 48) & 0xff) as u8);
  buffer.push(((func_addr >> 56) & 0xff) as u8);
  //mov rdi, toy
  buffer.push(0x48); //REX.W prefix
  buffer.push(0xbf);
  buffer.push((toy_addr & 0xff) as u8);
  buffer.push(((toy_addr >> 8) & 0xff) as u8);
  buffer.push(((toy_addr >> 16) & 0xff) as u8);
  buffer.push(((toy_addr >> 24) & 0xff) as u8);
  buffer.push(((toy_addr >> 32) & 0xff) as u8);
  buffer.push(((toy_addr >> 40) & 0xff) as u8);
  buffer.push(((toy_addr >> 48) & 0xff) as u8);
  buffer.push(((toy_addr >> 56) & 0xff) as u8);
  ////call rcx
  //buffer.push(0x48); //REX.W prefix
  buffer.push(0xff); //call
  buffer.push(0xd0);

  buffer.push(0x48); //REX.W prefix
  buffer.push(0x8b); //MOV r r
  buffer.push(0xc7); //MOD/RM byte for %rdi -> %rax
  buffer.push(0xc3); //ret
  let mut mmap = MmapMut::map_anon(buffer.len())?;
  mmap.copy_from_slice(&buffer);
  let mmap: Mmap = mmap.make_exec()?;
  let addr = mmap.as_ptr();
  println!("{}", toy.x());
  println!("{}", toy2.x());
  println!("{}", Toy::new().compare(&toy));
  unsafe {
    let f = std::mem::transmute::<*const u8, fn(u64) -> u64>(addr);
    for i in 0..10 {
      f(i);
    }
    asm!("mov rdi, $0
          call $1"
         :
         : "r"(toy2_addr), "r"(func_addr)
         :
         : "intel");
  };
  println!("{}", toy.x());
  println!("{}", toy2.x());
  println!("{}", Toy::new().compare(&toy));
  Ok(())
}

struct Toy {
  x: u32,
}

impl Toy {
  fn new() -> Toy {
    Toy { x: 0 }
  }
  fn x(&self) -> u32 {
    self.x
  }
  fn g(&mut self) {
    self.x += 1;
    return;
  }
  fn compare(&self, other: &Toy) -> bool {
    self.x == other.x()
  }
}
