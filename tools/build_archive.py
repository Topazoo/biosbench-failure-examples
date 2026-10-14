"""Deterministic source archive, excluding test executables and private audit."""

import hashlib
import zipfile

from check_package import ROOT, FILES, check


if __name__ == "__main__":
    check()
    folder = ROOT / "dist"
    folder.mkdir(exist_ok=True)
    output = folder / "biosbench-failure-examples-review.zip"
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_STORED) as archive:
        for name in FILES:
            info = zipfile.ZipInfo("biosbench-failure-examples/" + name, (2026, 1, 1, 0, 0, 0))
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            archive.writestr(info, (ROOT / name).read_bytes())
    digest = hashlib.sha256(output.read_bytes()).hexdigest()
    output.with_suffix(".zip.sha256").write_text(
        digest + "  " + output.name + "\n", encoding="ascii", newline="\n")
    print(digest + "  " + output.name)
