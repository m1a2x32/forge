.section .text.vectors, "ax"
.balign 2048
.global __exception_vectors
__exception_vectors:
// 16 architected slots, each 128 bytes. Save x0/x1 before carrying the slot index
// to common code; all original registers are captured before entering Rust.
.macro VECTOR slot
    .balign 128
    sub sp, sp, #288
    stp x0, x1, [sp, #0]
    mov x1, #\slot
    b .Lexception_common
.endm
VECTOR 0
VECTOR 1
VECTOR 2
VECTOR 3
VECTOR 4
VECTOR 5
VECTOR 6
VECTOR 7
VECTOR 8
VECTOR 9
VECTOR 10
VECTOR 11
VECTOR 12
VECTOR 13
VECTOR 14
VECTOR 15
.balign 128
.Lexception_common:
    stp x2, x3, [sp, #16]
    stp x4, x5, [sp, #32]
    stp x6, x7, [sp, #48]
    stp x8, x9, [sp, #64]
    stp x10, x11, [sp, #80]
    stp x12, x13, [sp, #96]
    stp x14, x15, [sp, #112]
    stp x16, x17, [sp, #128]
    stp x18, x19, [sp, #144]
    stp x20, x21, [sp, #160]
    stp x22, x23, [sp, #176]
    stp x24, x25, [sp, #192]
    stp x26, x27, [sp, #208]
    stp x28, x29, [sp, #224]
    str x30, [sp, #240]
    add x0, sp, #288
    str x0, [sp, #248]
    mrs x0, elr_el1
    str x0, [sp, #256]
    mrs x0, esr_el1
    str x0, [sp, #264]
    mrs x0, far_el1
    str x0, [sp, #272]
    mrs x0, spsr_el1
    str x0, [sp, #280]
    mov x0, sp
    bl forge_exception
1:  wfe
    b 1b
