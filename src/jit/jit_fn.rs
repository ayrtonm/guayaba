use memmap::Mmap;

pub struct JIT_Fn {
  mmap: Mmap,
  pub name: fn(),
}

impl JIT_Fn {
  pub fn new(mmap: Mmap, name: fn()) -> Self {
    JIT_Fn {
      mmap,
      name,
    }
  }
  pub fn execute(&self) {
    (self.name)();
  }
}
