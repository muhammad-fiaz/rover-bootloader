# Security Policy

The `rover-bootloader` project takes security, memory safety, and pre-boot integrity seriously. This document outlines our vulnerability reporting process and security posture.

---

## Supported Versions

Security updates and vulnerability patches are maintained for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1.0 | :x:                |

---

## Reporting a Vulnerability

If you discover a security vulnerability or potential defect in `rover-bootloader`, please report it responsibly so that we can address it before public disclosure.

**Please DO NOT report security issues via public GitHub issues, discussions, or pull requests.**

### Reporting Channels

You can report vulnerabilities securely via:

1. **Email:** Send your report directly to **[contact@muhammadfiaz.com](mailto:contact@muhammadfiaz.com)** with the subject line `[SECURITY] rover-bootloader: <Brief Summary>`.
2. **GitHub Security Advisory:** Submit a private vulnerability report via [GitHub Private Vulnerability Reporting](https://github.com/muhammad-fiaz/rover/security/advisories/new).

### Information to Include

To assist in timely triage and resolution, please provide:

- **Component:** The affected module (e.g., `formats::elf`, `fs::uefi_fs`, `memory::exit_boot`, `security::sha256`).
- **Severity & Impact:** Assessment of impact (e.g., arbitrary code execution, denial of service, memory corruption before kernel transition).
- **Reproduction Steps:** Minimal proof-of-concept file, image, or configuration demonstrating the issue.
- **Environment:** Target architecture (`x86_64-unknown-uefi`, `aarch64-unknown-uefi`, `i686-unknown-uefi`), Rust toolchain version, and QEMU/OVMF configuration.

---

## Security Architecture

`rover-bootloader` is designed with the following security properties:

- **Strict `#![no_std]`:** Eliminates runtime attack surfaces associated with host OS environments.
- **Zero Panic Architecture:** Parsers use checked math and bounded slicing. Compiler lints deny unwrap and expect usage.
- **SHA-256 Pre-Execution Verification:** Kernel images can be verified against expected cryptographic hashes before memory placement and execution.
- **Clean Handoff:** Interrupts are masked, stacks are isolated, and memory maps are acquired immediately prior to terminating boot services.
