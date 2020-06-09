#![feature(llvm_asm)]

fn main() {
  unsafe {
    llvm_asm!("
      jmp branch_delay_slot

      jump:
        nop
        addq $$8, %rsp
        jne end
        addq $$-8, %rsp
        ret

      branch_delay_slot:
        nop
        call jump

      next_opcode:
        nop

      end:
        nop
        ret");
  }
}
