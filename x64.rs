#![feature(asm)]
use std::io;
use memmap::{Mmap, MmapMut};

fn main() -> io::Result<()> {
  let mut buffer = Vec::new();
  let func_addr: u64 = g as extern "C" fn() as u64;
  //nop
  buffer.push(0x90);

  //mov rcx, func
  buffer.push(0x48); //REX.W prefix
  buffer.push(0xb9);
  buffer.push((func_addr & 0xff) as u8);
  buffer.push(((func_addr >> 8) & 0xff) as u8);
  buffer.push(((func_addr >> 16) & 0xff) as u8);
  buffer.push(((func_addr >> 24) & 0xff) as u8);
  buffer.push(((func_addr >> 32) & 0xff) as u8);
  buffer.push(((func_addr >> 40) & 0xff) as u8);
  buffer.push(((func_addr >> 48) & 0xff) as u8);
  buffer.push(((func_addr >> 56) & 0xff) as u8);
  //call rcx
  buffer.push(0x48); //REX.W prefix
  buffer.push(0xff); //call
  buffer.push(0xd1);

  buffer.push(0x48); //REX.W prefix
  buffer.push(0x8b); //MOV r r
  buffer.push(0xc7); //MOD/RM byte for %rdi -> %rax
  buffer.push(0xc3); //ret
  let mut mmap = MmapMut::map_anon(buffer.len())?;
  mmap.copy_from_slice(&buffer);
  let mmap: Mmap = mmap.make_exec()?;
  let addr = mmap.as_ptr();
  unsafe {
    let f = std::mem::transmute::<*const u8, fn(u64) -> u64>(addr);
    for i in 0..=10 {
      println!("{:#x?}", f(i));
    }
    asm!("call $0"
         :
         : "i"(g as extern "C" fn())::"intel");
  };
  println!("{:#x?}", func_addr);
  println!("{:x?}", g as extern "C" fn());
  Ok(())
}

pub extern fn g() {
  let x = 1 + 2;
  return;
}
