# Changelog

## [1.0.0] – 2025-06-16
### Added
* Initial public release of BST-SEAL codec.
* Rust crates: `bstseal-core`, `bstseal-cli`, `bstseal-ffi`.
* CLI subcommands: `encode`, `decode`, `pack`, `unpack`, `list`, `cat`, `fsck`, `bench`.
* Integrity layer with Blake3 footer.
* Archive container format (`.bsa`).
* Multithreaded encode/decode with SIMD-optimised Huffman.
* Cross-language bindings: C header, Node.js (ffi-napi), Unity C# wrapper.
* GitHub Actions CI (build, tests, lints, Node smoke test).
* Documentation: `README.md`, `SPEC.md`.
