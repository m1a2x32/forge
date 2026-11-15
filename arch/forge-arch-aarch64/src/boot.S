.section .text.boot, "ax"
.global _start
.type _start, %function
_start:
    // ARM64 Image header lets QEMU use its direct-kernel boot protocol (DTB in x0).
    b .Lstart
    .long 0
    .quad 0x80000
    .quad __kernel_end - _start
    .quad 0
    .quad 0, 0, 0
    .byte 0x41, 0x52, 0x4d, 0x64
    .long 0
.Lstart:
    msr daifset, #0xf
    // Park secondary CPUs before touching a shared stack or BSS. QEMU normally
    // holds these in PSCI; this is also defensive for direct secondary entry.
    mrs x2, mpidr_el1
    and x2, x2, #0xffffff
    cbnz x2, .Lpark
    mov x20, x0
    mrs x21, CurrentEL
    lsr x21, x21, #2
    cmp x21, #2
    b.eq .Lfrom_el2
    cmp x21, #1
    b.eq .Lel1
    b .Lpark
.Lfrom_el2:
    // EL1 executes AArch64; disable optional EL2 traps for this direct boot path.
    mov x2, #(1 << 31)
    msr hcr_el2, x2
    msr cptr_el2, xzr
    mov x2, #3
    msr cnthctl_el2, x2
    msr cntvoff_el2, xzr
    adrp x2, __stack_top
    add x2, x2, :lo12:__stack_top
    msr sp_el1, x2
    // ARMv8-A RES1 bits; MMU, data cache and instruction cache remain disabled.
    mov x2, #0x0800
    movk x2, #0x30d0, lsl #16
    msr sctlr_el1, x2
    adr x2, .Lel1
    msr elr_el2, x2
    mov x2, #0x3c5 // DAIF masked, EL1h (SP_EL1).
    msr spsr_el2, x2
    isb
    eret
.Lel1:
    msr spsel, #1
    adrp x2, __stack_top
    add x2, x2, :lo12:__stack_top
    mov sp, x2
    // QEMU's direct boot contract supplies M=C=I=0. No firmware/cache teardown
    // is implemented: do not use this entry for arbitrary bootloaders yet.
    mov x2, #0x0800
    movk x2, #0x30d0, lsl #16
    msr sctlr_el1, x2
    isb
    adrp x2, __bss_start
    add x2, x2, :lo12:__bss_start
    adrp x3, __bss_end
    add x3, x3, :lo12:__bss_end
.Lzero_bss:
    cmp x2, x3
    b.hs .Lbss_done
    str xzr, [x2], #8
    b .Lzero_bss
.Lbss_done:
    dsb sy
    isb
    mov x0, x20
    mov x1, x21
    bl forge_entry
.Lpark:
    wfe
    b .Lpark
.size _start, . - _start
