#!/usr/bin/env python3
"""Extract an AArch64 ELF's file-backed PT_LOAD bytes; no external objcopy needed."""

import argparse
import pathlib
import struct


def convert(source: pathlib.Path, destination: pathlib.Path) -> None:
    elf = source.read_bytes()
    if elf[:7] != b"\x7fELF\x02\x01\x01":
        raise ValueError("expected a little-endian ELF64 file")
    header = struct.unpack_from("<HHIQQQIHHHHHH", elf, 16)
    elf_type, machine, _, entry, phoff, _, _, _, phsize, phnum, *_ = header
    if elf_type != 2 or machine != 183 or phsize != 56:
        raise ValueError("expected a static AArch64 executable")
    segments = []
    for index in range(phnum):
        kind, _, offset, vaddr, paddr, filesz, memsz, _ = struct.unpack_from(
            "<IIQQQQQQ", elf, phoff + index * phsize
        )
        if kind == 1 and filesz:
            if vaddr != paddr or filesz > memsz or offset + filesz > len(elf):
                raise ValueError("unsupported or truncated load segment")
            segments.append((paddr, elf[offset : offset + filesz]))
    base = min(address for address, _ in segments)
    end = max(address + len(data) for address, data in segments)
    if base != 0x40080000 or entry != base or end - base > 2 * 1024 * 1024:
        raise ValueError("unexpected Forge load layout")
    image = bytearray(end - base)
    for address, data in segments:
        image[address - base : address - base + len(data)] = data
    if image[56:60] != b"ARM\x64":
        raise ValueError("missing ARM64 Image header")
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(image)
    print(f"{destination}: {len(image)} bytes (load address {base:#x})")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("elf", type=pathlib.Path)
    parser.add_argument("image", type=pathlib.Path)
    args = parser.parse_args()
    convert(args.elf, args.image)
