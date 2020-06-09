#![feature(llvm_asm)]

fn main() {
  unsafe {
    llvm_asm!("
      condition:
        jmp branch_delay_slot

      jump:
        jne to_end
        ret

      to_end:
        addq $$8, %rsp
        jmp end

      branch_delay_slot:
        pushfq
        nop
        popfq
        nop
        nop
        nop
        call jump
        nop
        nop
        nop

      next_opcode:
        nop

      end:
        nop
        ret");
  }
}
