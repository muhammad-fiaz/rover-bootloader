# Contributing to Rover Bootloader

Thank you for your interest in contributing to `rover-bootloader`! This document outlines the engineering standards, architecture rules, testing criteria, and development workflows required for this project.

---

## Core Principles

`rover-bootloader` is engineered as an independent, high-assurance, bare-metal UEFI bootloader in native Rust. Every contribution must adhere to the following principles:

1. **Pure `#![no_std]` Execution:** The bootloader operates in a pre-boot environment before any kernel or standard library exists. It must remain `#![no_std]`.
2. **Strict Dependency Isolation:** The crate must not introduce third-party runtime dependencies. All UEFI specifications, protocols, tables, and GUIDs must come exclusively from `rover-uefi`.
3. **Specification-Accurate ABIs:** Kernel handoff thunks and protocol structures must conform to target processor ABIs (such as System V AMD64 ABI on `x86_64`) and the Rover Boot Protocol v1 specification with `#[repr(C)]` layouts.
4. **Defensive Bounded Parsing:** Parsers (ELF, PE, configuration files) must never panic on malformed or malicious inputs. All indexing must use checked slicing and bounds verification.
5. **No Compiler Warnings:** All code must compile cleanly under `cargo clippy --all-targets --all-features -- -D warnings` and format cleanly under `cargo fmt --all -- --check`.

---

## Prerequisites

Install a modern Rust toolchain (Rust 1.85+ recommended) along with the bare-metal UEFI target triples:

```bash
rustup target add x86_64-unknown-uefi
rustup target add aarch64-unknown-uefi
rustup target add i686-unknown-uefi
rustup component add clippy rustfmt
```

---

## Development Workflow

### 1. Host Tests

Run host unit and integration tests:

```bash
cargo test --all-features
```

### 2. Cross-Compilation Checks

Verify compilation across all supported UEFI target architectures:

```bash
cargo check --target x86_64-unknown-uefi
cargo check --target aarch64-unknown-uefi
cargo check --target i686-unknown-uefi
```

### 3. Linting and Formatting

Ensure full compliance with formatting and linter requirements:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

---

## Pull Request Checklist

Before submitting a pull request, ensure:

- [ ] All unit and integration tests pass cleanly (`cargo test --all-features`).
- [ ] Cross-compilation targets compile without errors.
- [ ] `cargo clippy` emits zero warnings across all targets.
- [ ] `cargo fmt` produces zero diffs.
- [ ] Documentation comments include `# Safety` and `# Errors` sections where applicable.
- [ ] No third-party runtime crates are added to `Cargo.toml`.

---

## License

By contributing to `rover-bootloader`, you agree that your contributions will be licensed under the Apache License, Version 2.0.
