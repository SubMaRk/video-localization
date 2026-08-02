# VID-IMPL-P00-001 Scaffold Execution Guide

This directory contains the phase 00 scaffold entrypoint scripts for implementation work.

## Scope

- Repository bootstrap validation
- Toolchain and dependency discoverability checks
- Repo test-harness smoke checks for scaffold-only scaffolding

No product behavior is implemented in this issue.

## Execution

- `pwsh .\eng\build.ps1`
- `pwsh .\eng\test.ps1`
- `pwsh .\eng\print-versions.ps1`
- `pwsh .\eng\verify-toolchain.ps1`

Both scripts write machine-readable evidence under:

- `Video Localization/specifications/evidence/PHASE-00-IMPLEMENTATION/VID-IMPL-P00-001/`

## Assumptions documented during execution

1. Windows App SDK is pinned centrally; WinUI application composition and MSIX packaging are tracked after this scaffold gate.
2. This scaffold records only repository, toolchain, and test-harness readiness for Phase 00.
3. The proof of platform compatibility will be produced when behavior is added in later issues.
