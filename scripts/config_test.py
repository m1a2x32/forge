#!/usr/bin/env python3
"""Check that Cargo reads configuration files and rebuilds when they change."""

import os
from pathlib import Path
import subprocess
import tempfile

from smoke import ROOT, check


def build(env: dict, expected_error: str | None = None) -> None:
    result = subprocess.run(["bash", "scripts/build.sh"], cwd=ROOT, env=env,
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    if expected_error is None:
        assert result.returncode == 0, result.stdout
    else:
        assert result.returncode != 0, "invalid configuration was accepted"
        assert expected_error in result.stdout, result.stdout


def main() -> None:
    (ROOT / "build").mkdir(exist_ok=True)
    original_env = os.environ.copy()
    try:
        with tempfile.TemporaryDirectory(prefix="config-check-", dir=ROOT / "build") as directory:
            config = Path(directory) / "test.cfg"
            env = original_env | {"FORGE_CONFIG": str(config.relative_to(ROOT))}
            for size in (4096, 16384):
                # Reuse the same filename to check rerun-if-changed behavior.
                config.write_text(f"boot_frame_size = {size}\n")
                build(env)
                for el in (1, 2):
                    check("normal", el, expected_frame_size=size)
            config.write_text("boot_frame_size = 6144\n")
            build(env, "nonzero power of two")
            env["FORGE_CONFIG"] = str(config.with_name("missing.cfg"))
            build(env, "cannot read Forge config")
        print("PASS configuration file changes and build errors")
    finally:
        # Restore the caller's selected image after the temporary file is removed.
        build(original_env)


if __name__ == "__main__":
    main()
