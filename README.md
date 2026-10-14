# BiosBench Failure Examples

## Moved to BiosBench Evidence

Development continues in
[BiosBench Evidence: `examples/failure-examples/`](https://github.com/Topazoo/biosbench-evidence/tree/main/examples/failure-examples).
Use the [unified developer guide](https://github.com/Topazoo/biosbench-evidence#rust-failure-examples)
for current commands and the
[migration record](https://github.com/Topazoo/biosbench-evidence/blob/main/examples/failure-examples/docs/SCOPE.md#public-repository-consolidation)
for source provenance. The MIT license, attribution, and original source history
remain available here so existing links keep working.

This repository is retained as an archival reference. Please use the unified
repository for new work. The documentation below describes this historical copy.

Small host-only Rust examples for save failures, terminal sessions, and one-use
operations. MIT licensed, copyright 2026 Peter Swanson. No third-party crates,
firmware images, emulators, collectors, drivers, or privileged operations.

Source: https://github.com/Topazoo/biosbench-failure-examples

Companion: [BiosBench Evidence](https://github.com/Topazoo/biosbench-evidence).

## Run

With Rust 1.80+ and a host linker installed:

```console
cargo test --locked --offline
python tools/check_package.py
python tools/build_archive.py
```

The optional Python packaging checks require Python 3.12+. All tests operate on
in-memory synthetic data. Cargo writes only normal local build output. The save
harness serializes entire scenarios, so the default parallel test runner is safe.
Alternatively `python tools/run_tests.py --rustc /path/to/rustc` builds and runs
the two test executables directly, with output confined to this package's target/.

## What is actually tested

- **Production save control flow:** `src/report.rs` is the project save routine,
  compiled directly into `tests/save_faults.rs` against an in-memory UEFI API
  double. Cases cover each I/O failure stage, preservation of the original error
  when cleanup also fails, complete/partial leftover files, numbered-name
  preservation, exhaustion, overflow and non-regular files.
- **One-use operation:** `one_use::run` consumes a supplied permission before
  invoking fallible code. Failure, success, and reentry cannot reuse it.
- **Session guard:** `session::Session` binds opaque owner/epoch tokens, rejects
  drift, and makes close, failed operations, invalid binding and panic terminal.

The save routine is a test fixture here, not an exported production filesystem
API. Its name-availability check is not an atomic create-if-absent guarantee.
The in-memory double cannot model concurrent filesystem writers, real durability,
firmware bugs, power loss, or an actual UEFI implementation. A complete readable
file after a simulated flush failure does not turn the failed save into success.

Session token equality is not authentication, freshness or exclusive ownership.
The one-use atomic is not a security boundary against a caller that can reset it.
These mechanisms neither authorize motherboard writes nor provide a hardware
control backend. See [provenance and scope](docs/SCOPE.md).
