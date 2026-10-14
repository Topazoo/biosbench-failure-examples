"""Build/run host-only Rust tests directly with a caller-selected compiler."""

import argparse
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rustc", default="rustc")
    parser.add_argument("--rustc-arg", action="append", default=[],
                        help="Optional compiler argument for an explicitly configured host linker")
    args = parser.parse_args()
    destination = ROOT / "target" / "direct-tests"
    destination.mkdir(parents=True, exist_ok=True)
    suffix = ".exe" if sys.platform == "win32" else ""
    for name, source in (("core", "src/lib.rs"), ("save_faults", "tests/save_faults.rs")):
        output = destination / (name + suffix)
        subprocess.run([args.rustc, *args.rustc_arg, "--edition=2021", "--test", str(ROOT / source),
                        "-o", str(output)], cwd=ROOT, check=True)
        subprocess.run([str(output)], cwd=ROOT, check=True)


if __name__ == "__main__":
    main()
