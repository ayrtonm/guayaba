use memmap::Mmap;

pub struct JIT_Fn {
  mmap: Mmap,
  #[cfg(not(test))]
  name: fn(),
  #[cfg(test)]
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
