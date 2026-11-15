#!/usr/bin/env python3
"""Boot real AArch64 images and assert UART diagnostics; timeout alone never passes."""

import os
from pathlib import Path
import re
import select
import subprocess
import time

ROOT = Path(__file__).resolve().parent.parent
QEMU = os.environ.get("QEMU_SYSTEM_AARCH64", "qemu-system-aarch64")


def check(mode: str, el: int, cpus: int = 1, ram: int = 128,
          expected_frame_size: int | None = None) -> None:
    name = f"{mode}-el{el}-{cpus}cpu-{ram}M"
    log_path = ROOT / "build" / "logs" / f"{name}.log"
    log_path.parent.mkdir(parents=True, exist_ok=True)
    command = [
        QEMU, "-machine", f"virt,virtualization={'on' if el == 2 else 'off'}",
        "-cpu", "cortex-a53", "-m", f"{ram}M", "-smp", str(cpus),
        "-display", "none", "-monitor", "none", "-serial", "stdio",
        "-nic", "none", "-no-reboot", "-kernel", str(ROOT / f"build/{mode}/forge.img"),
    ]
    expected = {
        "normal": "FORGE_BOOT_OK",
        "exception": "FORGE_EXCEPTION_HALTED",
        "panic": "FORGE_PANIC_HALTED",
    }[mode]
    output = bytearray()
    try:
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    except FileNotFoundError as error:
        raise SystemExit(f"QEMU missing: set QEMU_SYSTEM_AARCH64 or install qemu-system-arm ({error})")
    try:
        deadline = time.monotonic() + 10
        completed_at = None
        while time.monotonic() < deadline:
            if select.select([process.stdout], [], [], 0.1)[0]:
                chunk = os.read(process.stdout.fileno(), 65536)
                if not chunk:
                    break
                output.extend(chunk)
            if expected.encode() in output and completed_at is None:
                completed_at = time.monotonic()
            # Capture any trailing output, including accidental secondary boots.
            if completed_at is not None and time.monotonic() - completed_at >= 0.2:
                break
            if process.poll() is not None:
                break
        text = output.decode(errors="replace").replace("\r", "")
        log_path.write_text(text)
        assert process.poll() is None, f"QEMU exited unexpectedly ({process.returncode})"
        assert expected in text, f"missing {expected} (boot timeout or incomplete output)"
        assert f"entry=EL{el} current=EL1" in text, "wrong exception-level transition"
        assert text.count("Forge OS 0.1.0") == 1, "kernel must initialize exactly once"
        assert "BSS/data/stack checks passed; MMU/caches off" in text
        allocation = re.search(r"memory: (\d+) frames of (\d+) bytes, allocation check passed", text)
        assert allocation, "missing boot allocation check"
        frames, frame_size = map(int, allocation.groups())
        assert frames >= 2 and frame_size > 0
        assert frame_size & (frame_size - 1) == 0, "invalid frame size"
        if expected_frame_size is not None:
            assert frame_size == expected_frame_size, "image uses the wrong configuration"
        assert "kernel: initialized" in text
        if mode == "exception":
            assert "FORGE_EXCEPTION slot=4 ec=0x3c" in text
            assert "ESR=0x00000000f2000046" in text, "wrong breakpoint syndrome"
            assert len(re.findall(r"^x\d{2}=", text, re.MULTILINE)) == 31
            for register in (0, 1, 2, 16, 30):
                assert f"x{register:02}={0x100 + register:#018x}" in text, "register capture corrupted"
            assert "FORGE_PANIC" not in text
        elif mode == "panic":
            assert "intentional panic test" in text
            assert "FORGE_EXCEPTION" not in text
        else:
            assert "FORGE_PANIC" not in text and "FORGE_EXCEPTION" not in text
        if mode != "normal":
            assert "FORGE_BOOT_OK" not in text, "fault path incorrectly returned"
    except AssertionError as error:
        raise SystemExit(f"FAIL {name}: {error}\nSee {log_path}\n{output.decode(errors='replace')}")
    finally:
        if process.poll() is None:
            process.terminate()
        try:
            process.wait(timeout=2)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
    print(f"PASS {name}")


def main() -> None:
    for mode in ("normal", "exception", "panic"):
        subprocess.run(["bash", "scripts/build.sh", mode], cwd=ROOT, check=True)
    for el in (1, 2):
        check("normal", el)
        check("normal", el, cpus=4, ram=256)
        check("exception", el)
        check("panic", el)
    print("All 8 QEMU boot/fault checks passed.")


if __name__ == "__main__":
    main()
