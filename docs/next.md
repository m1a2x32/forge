# Next increment: discovering RAM before using it

The immediate next subsystem is a trustworthy physical memory map. MMU tables
depend on that map; scheduling can follow once interrupts and a timer can return
from their handlers.

Implement a small bounds-checked flattened-device-tree reader or evaluate a
focused `no_std` library. Keep the parser independent of MMIO. Test truncated
headers, structure/string bounds, malformed cell counts and arithmetic overflow
on the host before passing it a slice of guest physical memory.

On QEMU, read `/memory` address/size cells, the header reservation map and applicable
`reserved-memory` nodes. Reserve the complete kernel including BSS, boot arena and
stack; the device tree itself; and all firmware-declared reservations. Do not
quietly assume every address above the kernel is free. Establish the safety
argument for mapping the DTB pointer to a readable bounded slice at the platform
boundary, where the QEMU boot contract is known.

Create a portable representation of disjoint usable/reserved physical intervals.
Feed free intervals to a real frame allocator, keeping permanent reservations
separate from transient boot allocations. Keep this independent of virtual memory
so a future MCU port can use memory mechanisms without page-table machinery.

Verify the reported RAM changes when QEMU runs with 128 and 256 MiB, that no frame
overlaps any reservation, and that exhaustion fails cleanly. Only then build the
first AArch64 translation tables: normal RAM vs device attributes, cache policy,
kernel W^X, identity-mapped transition code and stack, and the required barriers
and TLB maintenance. Enable the MMU as a separate, reviewable milestone.

Later increments add a timer and GIC driver, a returning exception path, cooperative
task contexts, and then preemption. No shell, network stack or process loader is
needed to validate those kernel mechanisms.
