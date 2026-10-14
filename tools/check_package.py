"""Check only the explicit source package. No owner-workspace discovery."""

import ast
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
FILES = sorted((
    "Cargo.toml", "Cargo.lock", "LICENSE", "README.md", ".editorconfig",
    ".gitattributes", ".gitignore", "src/lib.rs", "src/report.rs", "src/one_use.rs",
    "src/session.rs", "tests/save_faults.rs", "docs/SCOPE.md",
    "tools/run_tests.py", "tools/check_package.py", "tools/build_archive.py",
    ".github/workflows/check.yml",
))


def check():
    actual = {p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*") if p.is_file()
              and not any(part in {".git", "target", "dist", "__pycache__"}
                          for part in p.relative_to(ROOT).parts)}
    if actual != set(FILES):
        raise ValueError(f"Unexpected file set: {actual ^ set(FILES)}")
    for name in FILES:
        data = (ROOT / name).read_bytes()
        text = data.decode("utf-8")
        if b"\r" in data or b"\t" in data or not data.endswith(b"\n"):
            raise ValueError(f"Line formatting: {name}")
        if any(line.rstrip() != line for line in text.splitlines()):
            raise ValueError(f"Trailing whitespace: {name}")
        if name.endswith(".py"):
            ast.parse(text, filename=name)
        if name != "tools/check_package.py" and re.search(
                r"[A-Za-z]:\\|/Users/|/home/|app\.slack\.com|BEGIN .*PRIVATE KEY", text):
            raise ValueError(f"Potential private material: {name}")
    print(f"PASS: {len(FILES)} allowlisted text files; no bundled dependencies or binaries")


if __name__ == "__main__":
    check()
