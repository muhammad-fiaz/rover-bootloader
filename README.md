<div align="center">

# Rover Bootloader

<a href="https://github.com/muhammad-fiaz/rover-bootloader"><img src="https://img.shields.io/badge/Rust-2024%20Edition-orange.svg?logo=rust" alt="Rust Edition"></a>
<a href="https://github.com/muhammad-fiaz/rover-bootloader"><img src="https://img.shields.io/github/stars/muhammad-fiaz/rover-bootloader" alt="GitHub stars"></a>
<a href="https://github.com/muhammad-fiaz/rover-bootloader/issues"><img src="https://img.shields.io/github/issues/muhammad-fiaz/rover-bootloader" alt="GitHub issues"></a>
<a href="https://github.com/muhammad-fiaz/rover-bootloader/pulls"><img src="https://img.shields.io/github/issues-pr/muhammad-fiaz/rover-bootloader" alt="GitHub pull requests"></a>
<a href="https://github.com/muhammad-fiaz/rover-bootloader"><img src="https://img.shields.io/github/last-commit/muhammad-fiaz/rover-bootloader" alt="GitHub last commit"></a>
<a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue.svg" alt="License"></a>
<a href="https://github.com/muhammad-fiaz/rover-bootloader/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/muhammad-fiaz/rover-bootloader/ci.yml?branch=main&label=CI" alt="CI"></a>
<a href="https://github.com/muhammad-fiaz/rover-bootloader/actions/workflows/github-code-scanning/codeql"><img src="https://github.com/muhammad-fiaz/rover-bootloader/actions/workflows/github-code-scanning/codeql/badge.svg" alt="CodeQL"></a>
<img src="https://img.shields.io/badge/platforms-x86__64--uefi%20%7C%20aarch64--uefi%20%7C%20i686--uefi-blue" alt="Supported Platforms">
<a href="https://pay.muhammadfiaz.com"><img src="https://img.shields.io/badge/Sponsor-pay.muhammadfiaz.com-ff69b4?style=flat&logo=heart" alt="Sponsor"></a>
<a href="https://github.com/sponsors/muhammad-fiaz"><img src="https://img.shields.io/badge/Sponsor-GitHub-pink?style=social&logo=github" alt="GitHub Sponsors"></a>
<a href="https://hits.sh/github.com/muhammad-fiaz/rover-bootloader/"><img alt="Hits" src="https://hits.sh/github.com/muhammad-fiaz/rover-bootloader.svg"/></a>

<p><em>Modern UEFI Bootloader Built in Rust.</em></p>

</div>

`rover-bootloader` is an independent, modular, high-assurance UEFI bootloader implemented natively in `#![no_std]` Rust. Designed as a standalone bootloader, it leverages `rover-uefi` as its authoritative firmware interface library to discover system resources, configure display adapters, parse kernel executables, terminate firmware boot services safely, and transfer execution across multiple processor architectures.

> [!TIP]
> If you build with `rover-bootloader`, make sure give it a star ⭐ on Github!

---

<details>
<summary><strong>Features</strong> (click to expand)</summary>

| Feature Category | Description |
|---|---|
| **Zero-Dependency Core** | Pure `#![no_std]` Rust implementation depending exclusively on `rover-uefi`. Zero external third-party crates, ensuring maximum supply-chain security and predictable firmware behavior. |
| **Safe ExitBootServices** | Iterative `GetMemoryMap` and `ExitBootServices` transition loop adhering to Section 7.2 of the UEFI Specification. Automatically retries upon firmware memory allocation events. |
| **Defensive Binary Parsing** | Native zero-allocation ELF64, ELF32, and PE32+ parsers with strict bounds checking, machine architecture verification (`EM_X86_64`, `EM_AARCH64`, `EM_RISCV`), and BSS zero-filling (`p_memsz > p_filesz`). |
| **Rover Boot Protocol v1** | Standardized, 64-bit aligned handoff structure passing normalized/coalesced physical memory maps, linear framebuffer metadata, ACPI 2.0/1.0 RSDP, SMBIOS 3.x/2.x entry points, ramdisk modules, command lines, and architecture IDs. |
| **Hardware Graphics & Console** | `EFI_GRAPHICS_OUTPUT_PROTOCOL` (GOP) discovery, mode switching, pixel format mapping (RGB, BGR, Bitmask), and color-enabled interactive text console with keyboard navigation and timeout countdown. |
| **Multi-Architecture Handoff** | System V AMD64 ABI kernel transition for `x86_64` (`RDI` = `RoverBootInfo`, 16-byte aligned stack, `cli`), DAIF interrupt masking for `AArch64`, supervisor interrupt disabling for `riscv64`, and IA-32 stack setup. |
| **Cryptographic Integrity** | 100% native FIPS 180-4 SHA-256 implementation to verify kernel image checksums prior to execution. |
| **Dual Configuration Syntax** | Zero-allocation parser for `rover.cfg` supporting both block-based syntax (`entry "Name" { ... }`) and key-value INI syntax. |

</details>

---

<details>
<summary><strong>Prerequisites and Supported Platforms</strong> (click to expand)</summary>

<br>

## Prerequisites

| Requirement | Version | Notes |
|---|---|---|
| **Rust Toolchain** | Nightly or Stable 2024 Edition (1.85+) | Compatible with standard Rust installations |
| **UEFI Targets** | Target support via `rustup target add` | Bare-metal firmware cross-compilation |

---

## Supported Targets

| Architecture | Target Triple | Supported |
|---|---|:---:|
| **x86_64 (64-bit)** | `x86_64-unknown-uefi` | ✅ |
| **AArch64 (ARM64)** | `aarch64-unknown-uefi` | ✅ |
| **x86 (32-bit)** | `i686-unknown-uefi` | ✅ |
| **Host Test Runner** | `x86_64-pc-windows-msvc` / `x86_64-unknown-linux-gnu` | ✅ |

### Cross-Compilation

```bash
# Add bare-metal UEFI compilation targets
rustup target add x86_64-unknown-uefi
rustup target add aarch64-unknown-uefi
rustup target add i686-unknown-uefi

# Check compilation for x86_64 UEFI
cargo check --target x86_64-unknown-uefi

# Check compilation for ARM64 UEFI
cargo check --target aarch64-unknown-uefi

# Check compilation for 32-bit x86 UEFI
cargo check --target i686-unknown-uefi
```

</details>

---

## Configuration (`rover.cfg`)

The bootloader searches for `rover.cfg` in `\`, `\boot\`, and `\EFI\BOOT\`.

### Block Syntax Example

```ini
# Global settings
default = 0
timeout = 5

entry "Rover OS Primary" {
    kernel = /boot/kernel.elf
    initrd = /boot/initrd.img
    cmdline = console=tty0 root=/dev/sda1
    resolution = 1920x1080
    sha256 = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
}

entry "Rover OS Fallback" {
    kernel = /boot/rescue.elf
    cmdline = single
}
```

### INI Syntax Example

```ini
default = 0
timeout = 3

[Rover OS Standard]
kernel = \boot\kernel.elf
initrd = \boot\initrd.img
cmdline = quiet splash
```

---

## Quick Start

### Building the UEFI Executable

Build the optimized UEFI binary for `x86_64`:

```bash
cargo build --target x86_64-unknown-uefi --release
```

The resulting binary is located at:

```text
target/x86_64-unknown-uefi/release/rover-bootloader.efi
```

### Running Tests Locally

The complete unit and integration test suite runs natively on the host:

```bash
cargo test --all-features
```

---

## QEMU Validation

To validate `rover-bootloader.efi` under QEMU with OVMF firmware:

1. Prepare an EFI System Partition (ESP) directory:
   ```text
   esp/
   ├── EFI/
   │   └── BOOT/
   │       └── BOOTX64.EFI   # (copy of rover-bootloader.efi)
   ├── rover.cfg
   └── kernel.elf
   ```

2. Execute QEMU:
   ```bash
   qemu-system-x86_64 \
       -machine q35 \
       -m 512M \
       -cpu qemu64 \
       -drive if=pflash,format=raw,readonly=on,file=path/to/OVMF_CODE.fd \
       -drive if=pflash,format=raw,file=path/to/OVMF_VARS.fd \
       -drive format=raw,file=fat:rw:esp \
       -serial stdio \
       -net none
   ```

---

<details>
<summary><strong>Validation Matrix</strong> (click to expand)</summary>

<br>

Run the comprehensive test suite and linters locally:

```bash
# Run unit and integration tests (26 test suites)
cargo test --all-features

# Strict lint verification
cargo clippy --all-targets --all-features -- -D warnings

# Code style checking
cargo fmt --all -- --check

# Cross-compilation target checks
cargo check --target x86_64-unknown-uefi
cargo check --target aarch64-unknown-uefi
cargo check --target i686-unknown-uefi
```

</details>

---

<details>
<summary><strong>Project Structure</strong> (click to expand)</summary>

```text
rover-bootloader/
├── .github/                    # CI workflows, Dependabot & issue templates
│   ├── ISSUE_TEMPLATE/         # GitHub issue templates
│   ├── workflows/              # GitHub Actions CI matrix
│   ├── FUNDING.yml             # Project sponsor links
│   └── pull_request_template.md# PR review checklist
├── .gitignore                  # Git exclusion rules
├── Cargo.lock                  # Dependency lockfile
├── Cargo.toml                  # Package manifest, metadata & strict compiler lints
├── CODE_OF_CONDUCT.md          # Contributor Covenant Code of Conduct
├── CONTRIBUTING.md             # Contribution guidelines & development workflow
├── LICENSE                     # Apache-2.0 license text
├── README.md                   # Project documentation
├── SECURITY.md                 # Security & vulnerability reporting policy
├── src/
│   ├── lib.rs                  # Crate root, no_std configuration & top-level exports
│   ├── main.rs                 # UEFI binary entry point (efi_main) and host CLI fallback
│   ├── error.rs                # Strongly typed BootError enumeration
│   ├── abi/                    # Calling conventions and type aliases
│   ├── arch/                   # Architecture-specific kernel handoff thunks
│   │   ├── aarch64.rs          # ARM64 register setup, DAIF interrupt masking, and branch
│   │   ├── ia32.rs             # 32-bit x86 stack frame setup and jump
│   │   ├── riscv64.rs          # RISC-V 64 supervisor interrupt disabling and jump
│   │   └── x86_64.rs           # System V AMD64 ABI transition thunk (RDI, RSP, cli)
│   ├── boot/                   # Boot sequence orchestration and user interface
│   │   ├── menu.rs             # Interactive text console selection menu with countdown timer
│   │   └── pipeline.rs         # End-to-end boot pipeline orchestration
│   ├── config/                 # Configuration file parsing
│   │   ├── entry.rs            # BootEntry data structure
│   │   └── parser.rs           # Zero-allocation parser for rover.cfg (block & INI)
│   ├── formats/                # Native binary parsers
│   │   ├── elf.rs              # ELF64 and ELF32 executable parser, segment loader, BSS zeroing
│   │   └── pe.rs               # PE32+ and PE32 header and metadata parser
│   ├── fs/                     # Storage and filesystem abstraction
│   │   ├── path.rs             # UEFI path normalization (/ -> \) and UTF-16 encoding
│   │   └── uefi_fs.rs          # File reading via EFI_SIMPLE_FILE_SYSTEM_PROTOCOL
│   ├── graphics/               # Visual output interfaces
│   │   ├── console.rs          # EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL writer with color support
│   │   └── gop.rs              # EFI_GRAPHICS_OUTPUT_PROTOCOL discovery and mode resolution
│   ├── memory/                 # Physical memory management
│   │   ├── allocator.rs        # UEFI page and pool allocation wrappers
│   │   ├── exit_boot.rs        # Safe ExitBootServices retry loop
│   │   └── map.rs              # Memory descriptor normalization, sorting, and coalescing
│   ├── protocol/               # Rover Boot Protocol v1 definition
│   │   ├── framebuffer.rs      # Framebuffer geometry, stride, and RoverPixelFormat
│   │   ├── info.rs             # Root RoverBootInfo structure and ROVER_BOOT_MAGIC
│   │   ├── memory.rs           # RoverMemoryDescriptor and type classifications
│   │   └── module.rs           # Kernel image, ramdisk module, command line, and RoverSystemInfo
│   └── security/               # Cryptographic verification and firmware policy
│       ├── secure_boot.rs      # UEFI SecureBoot variable detection
│       └── sha256.rs           # Native FIPS 180-4 SHA-256 implementation
└── tests/                      # Comprehensive unit and integration test suite
    ├── config_tests.rs         # Configuration parser unit tests
    ├── crypto_tests.rs         # SHA-256 test vector validation
    ├── elf_parser_tests.rs     # ELF64 parsing, bounds validation, loading, and truncation rejection
    ├── path_tests.rs           # Path conversion and overflow protection tests
    ├── pe_parser_tests.rs      # PE32+ and PE32 parsing and rejection tests
    └── protocol_tests.rs       # Memory map parsing, coalescing, and protocol structure tests
```

</details>

---

## Acknowledgements

`rover-bootloader` is designed and engineered completely from the ground up natively in pure Rust. Its architectural interfaces, protocol structures, and binary loaders are built upon the foundational industry standards and technical literature:

- [UEFI Specification](https://uefi.org/specifications) (Versions 2.10 and 2.11), UEFI Forum.
- [UEFI Platform Initialization (PI) Specification](https://uefi.org/specifications) (Version 1.8), UEFI Forum.
- [System V Application Binary Interface: AMD64 Architecture Processor Supplement](https://gitlab.com/x86-psABIs/x86-64-ABI), Michael Matz et al.
- [Executable and Linking Format (ELF) Specification](https://refspecs.linuxfoundation.org/elf/elf.pdf), Tool Interface Standard (TIS) Portable Formats Specification Version 1.2.

---

## Contributing

Contributions are welcome! Please refer to the [CONTRIBUTING.md](CONTRIBUTING.md) guide for details on our development workflow, coding conventions, `#![no_std]` constraints, and pull request process.

---

## License

This project is licensed under the **Apache License 2.0** (`Apache-2.0`) - see the [LICENSE](LICENSE) file for details.
