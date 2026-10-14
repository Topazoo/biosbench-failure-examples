# Extraction record and limits

Prepared from generic project sources. The existing project MIT license is retained. No third-party
implementation or asset is included; the Rust/Python standard libraries are
prerequisites, not redistributed. The UEFI test double was project test code.
It mimics a small API surface; no uefi-rs crate source or vendor firmware is copied.

| Component | Original source SHA-256 | Adaptation |
| --- | --- | --- |
| Production `report.rs` | `29f5db9e11617141ca518aef9590517fd84f09e03161291d49ea28df9fbdc58f` | Retained production logic; formatting only. Compiled solely by the host test double. |
| `test_report_save_faults.rs` | `80ce40c58c026343f7d35b2f558ddf225ecb25c1608958966b449a4ce9aa0dec` | Relative module path, explicit transparent string-wrapper layout, and whole-scenario serialization. |
| `route_attempt.rs` | `677f821af2ba363dc9ac4dc4d03dd2a60b2e2d58b20e171579d7c222fb824189` | Renamed `one_use.rs`; mechanism retained. |
| Session example | Project `synthetic-profile-adapter-v4/src/session.rs` | Generic owner/epoch tokens and generic payload replace fixture-specific profile/browser identities; panic is terminal. |

The session example is an adaptation, not a claim that the full emulator adapter
was extracted or validated. No original profile-format mappings, board semantics,
addresses, calibration captures, QEMU requirements, fonts, or binaries are shipped.

The save double substitutes memory for file writes. Test artifacts are ordinary
host executables generated in target/, excluded from the source archive. Package
checks use an explicit file allowlist and basic private-path/key detection;
they do not claim universal secret detection. Archive order/metadata are fixed.

The repository remains `publish = false` to prevent accidental registry release.


Local Linux validation passed with Rust 1.99.0: eight one-use/session tests and
six production-save fault tests, through both the direct runner and offline Cargo.
Three unused-production-API warnings are expected from compiling the full save
routine in its smaller host harness. No test failed. The local linker required
an existing C runtime sysroot supplied read-only; all output/cache state stayed
inside this package. Hosts with a normal Rust/C linker setup need no custom flags.
The Linux/Windows hosted CI results are available in the repository Actions tab. The minimum
declared Rust version and Windows Rust execution have not been independently tested.
