# VID-DEC-001 - Video Implementation Stack

| Field | Value |
| --- | --- |
| Decision ID | `VID-DEC-001` |
| Status | Accepted |
| Revision | `1.0` |
| Decision owner | Product Lead: Video (`primary`) |
| Required reviewers | Domain Architect: Video (`terra`); Suite Architect (`luna`); Security Lead (`sonnet`); QA and Compatibility Lead (`deepseek`); Release Engineering Lead (`haiku`) |
| Authority | `OWNER-AUTH-V0` |
| Blocker | `P0:V0` |
| Depends on | `SUI-DEC-008@1.0` Accepted |
| Informs | `SUI-DEC-003`; `VID-DEC-002`; `VID-DEC-003`; `VID-SPEC-002`; `VID-SPEC-003`; `VID-SPEC-004`; `VID-SPEC-015` |

## Decision Question

Which implementation language, runtime, Windows desktop UI framework, media/playback process boundary, project database, IPC binding, build composition, and package model should support the Phase 00 and Phase 01 offline subtitle workflow without constraining later product semantics or importing deferred scope?

## Proposed Decision

### Desktop Application and Video Domain

- Use C# on the current supported `.NET 10` LTS patch line for the Windows desktop application, Video application services, command handlers, projections, and product-owned domain logic.
- Use WinUI 3 from the current Stable Windows App SDK line for the Windows 11 x64 application shell and production UI. Preview and Experimental channels are prohibited in release-bearing builds.
- Pin the exact .NET SDK/runtime, Windows App SDK, Windows SDK, compiler, generated-binding runtime, and package-tool revisions in machine-readable lock and build records.
- Keep product-domain contracts implementation-neutral. A future macOS or Linux client may use another UI stack while preserving the same project semantics and conformance behavior.

`.NET 10` is an active LTS release through November 2028. Microsoft identifies WinUI 3 with the Windows App SDK as the recommended native platform for new Windows applications. The Windows App SDK Stable channel is supported for production use but has a shorter servicing lifecycle than .NET, so the application MUST treat its exact patch line as a serviced dependency rather than a permanent file-format identity.

### Native Workers and Shared Boundary

- Use stable Rust for new isolated media probe, decode, waveform, thumbnail, conversion, export, and hostile-ingest worker hosts where memory safety, deterministic cleanup, bounded resource ownership, and native interoperability are primary.
- Consume Shared broker and worker contracts through generated bindings and authenticated local IPC. Video MUST NOT link Shared product-neutral services into an unrestricted UI-process mutation path.
- Use Protocol Buffers for bounded control messages when `SUI-DEC-003` is Accepted. Use Windows named pipes behind the Shared transport abstraction and out-of-band files, read-only mappings, shared memory, or negotiated streams for large immutable payloads.
- Do not carry video frames, waveform tiles, thumbnails, model payloads, or encoded media as base64 in the ordinary command envelope.
- Rust `unsafe` and native foreign-function interfaces MUST be isolated behind reviewed adapters with exact upstream identity, bounded inputs, structured errors, fuzz coverage, and crash containment.

This decision does not select a decoder, demuxer, subtitle renderer, converter, codec build, or redistributed binary. `VID-DEC-003` and `VID-SPEC-006` own exact promoted components, profiles, rights, and operation-level support claims.

### Media and Playback Boundary

- Video owns source timebase, presentation timestamps, source/proxy maps, frame/sample identity, subtitle rendering semantics, seek correctness, QC, and delivery behavior.
- Complex probe, demux, decode, subtitle/font parsing, proxy, mux, burn-in, and export work runs outside the UI process in bounded workers.
- The UI receives typed metadata and negotiated presentation surfaces. It hosts GPU presentation through a WinUI-compatible Direct3D swap-chain surface when available, with a CPU-capable functional path retained for the complete core workflow.
- Playback state and durable timing commands remain product-owned. A media library or platform player MUST NOT become the authority for durable cue time, VFR frame identity, source/proxy mapping, or project mutation.
- The Phase 00 spike MUST prove accurate presentation, seek, cancellation, worker restart, device-loss fallback, and non-base64 frame transport before this boundary is considered Implemented.

### Project Storage

- Use SQLite as the native per-project transactional metadata store, accessed only through one authoritative Video core writer boundary.
- UI views, workers, plugins, and future Teams clients MUST use commands, events, projections, and artifact references; they MUST NOT open the authoritative database directly.
- Keep media, waveform tiles, thumbnails, proxies, model payloads, and other large or rebuildable artifacts outside SQLite with content identity and lineage records in the project store.
- Active project databases MUST remain on supported local filesystems. Unmanaged network-share access is rejected or opened through an explicit safe import/copy workflow, never as concurrent authoritative SQLite access.
- WAL, checkpoint, backup, integrity checking, migration, sidecar files, forced-close recovery, disk-full behavior, and antivirus interference MUST be frozen and tested in `VID-SPEC-002` and `VID-SPEC-015`.
- Pin a security-fixed SQLite build and update it through dependency change control. Database format and project format versions remain separate identities.

### Build and Generated Contracts

- Compose the workspace from a pinned .NET solution and Cargo workspace; keep generated Protobuf bindings in deterministic generated-output paths with provenance and no hand edits.
- Use Buf locally and in CI for schema formatting, linting, generation, and breaking-change checks after `SUI-DEC-003` accepts the exact schema toolchain.
- Build and test from command-line entry points; an IDE MAY improve development but MUST NOT be required to reproduce an official build.
- Generate SBOM, notice, dependency, license, source, hash, compiler, and build-option records per artifact.
- Keep C# domain tests, Rust unit/property/fuzz tests, cross-language round trips, IPC fault tests, migration tests, and product conformance tests independently runnable.

### Windows Packaging

- Use packaged MSIX with stable package identity as the target Windows distribution model. Development and CI MAY also produce an unpackaged self-contained diagnostic build, but it is not the official package identity or update channel.
- Use x64 as the only Phase 00 and first-release architecture. ARM64 and other operating systems remain Later until separately promoted.
- Use self-contained runtime packaging for bounded alpha/offline evidence unless `VID-SPEC-015` proves that framework-dependent deployment better satisfies offline installation, servicing, rollback, footprint, and dependency-isolation gates.
- Developer/test packages use non-production identity and signing. Official signing, update, rollback, uninstall, and publication remain owner-reserved release operations.
- `VID-DEC-002` owns the exact project extension, application identity, package family naming, protocol/file association, and modified-build namespace rules.

Microsoft documents packaged WinUI 3 applications as the default project model and supports both framework-dependent and self-contained Windows App SDK deployment. The exact deployment mode therefore remains evidence-selected within this decision's MSIX boundary rather than an unsupported blanket claim.

## Required Architecture Spike

This proposal becomes Accepted only after a bounded spike records:

1. A C# command round trip through the proposed Protobuf binding and authenticated named-pipe transport to a Rust worker.
2. A large immutable payload transfer that avoids ordinary control-message amplification and validates identity, size, ownership, cleanup, cancellation, and stale-handle behavior.
3. A WinUI 3 shell presenting a generated or decoded test frame through the selected presentation surface while English/Thai and System/Light/Dark state changes preserve shell state.
4. SQLite create, transaction, forced-close, WAL recovery, backup, migration rollback, disk-full, and damaged-cache evidence under one authoritative writer.
5. Worker crash, hang, malformed response, cancellation, restart, and partial-artifact quarantine evidence.
6. A developer-signed x64 MSIX install, launch, update/rollback rehearsal, and uninstall test that preserves projects and separately retained resources.
7. Exact dependency, license, redistribution, SBOM, and security records for every component used by the spike.
8. Independent `deepseek` review finding no unresolved P0 correctness, recovery, compatibility, or test-gap blocker.

Spike code and fixtures are evidence only. They MUST NOT be represented as Phase 00 implementation until every P0 gate is open.

## Alternatives Considered

| Alternative | Disposition | Reason |
| --- | --- | --- |
| WPF on modern .NET | Rejected for the new primary UI | Mature Windows desktop option, but WinUI 3 is Microsoft's recommended new native Windows platform and provides the selected modern composition/deployment direction. |
| Avalonia shared UI | Deferred | Cross-platform UI reuse is not a first-release requirement and MUST NOT weaken Windows media, IME, accessibility, composition, or packaging evidence. |
| Electron or browser-first desktop shell | Rejected | Adds a web runtime and memory/process surface without solving native media timing, worker isolation, or offline package constraints. |
| Tauri or WebView UI with Rust backend | Rejected for the primary shell | A smaller web shell still introduces browser rendering semantics and does not provide the selected native WinUI component/accessibility path. |
| All-Rust desktop UI | Rejected | Would couple product UI risk to a less established Windows desktop accessibility/localization component ecosystem and reduce generated-binding separation from Shared. |
| In-process media engine | Rejected | Hostile input, codec failure, cancellation, resource exhaustion, and native crashes would share the UI failure boundary. |
| Direct SQLite access by UI or workers | Rejected | Breaks command authority, transaction ownership, recovery, future Teams semantics, and migration control. |
| JSON-only IPC or embedded binary payloads | Rejected | Does not satisfy bounded high-frequency control, schema compatibility, or large-payload transport requirements. |
| Unpackaged official Windows distribution | Rejected | Omits the selected stable package identity and predictable install/update/uninstall boundary. Diagnostic unpackaged builds remain allowed. |

## Consequences

Positive consequences:

- The Windows UI uses a current native platform while product semantics remain portable.
- C# supports productive desktop/domain development; Rust isolates native and hostile media work.
- Shared Protobuf/IPC contracts remain language-neutral and testable across release trains.
- SQLite provides transactional local project storage behind one command authority.
- MSIX establishes stable Windows identity and lifecycle evidence early.

Costs and risks:

- The product maintains both .NET and Rust toolchains plus generated bindings.
- Windows App SDK servicing is shorter than .NET LTS and requires planned patch/major-line qualification.
- WinUI/native rendering and Rust/native media integration require explicit interop, device-loss, packaging, and diagnostics work.
- Self-contained packaging increases footprint and shifts runtime patch delivery to application releases.
- Exact media libraries and builds remain unresolved until rights, security, profile, and corpus decisions pass.

## Acceptance Record

| Role | Agent | Status | Required evidence |
| --- | --- | --- | --- |
| Product Lead: Video | `primary` | Approved | Executed owner-authorized consolidated review of exact candidate hash |
| Domain Architect: Video | `terra` | Approved | Executed owner-authorized consolidated review of exact candidate hash |
| Suite Architect | `luna` | Approved | Executed owner-authorized consolidated review of exact candidate hash |
| Security Lead | `sonnet` | Approved | Executed owner-authorized consolidated review of exact candidate hash |
| QA and Compatibility Lead | `deepseek` | Approved | Executed owner-authorized consolidated review of exact candidate hash; not independent review |
| Release Engineering Lead | `haiku` | Approved | Executed owner-authorized consolidated review of exact candidate hash |

## References

- [Video Core Architecture](../Plan/sections/04_CORE_ARCHITECTURE.md)
- [Phase 00](../Plan/roadmap/PHASE_00_FOUNDATIONS.md)
- [Video Release Plan](../../_shared/Plan/release-planning/06_VIDEO_LOCALIZATION_RELEASE_PLAN.md)
- [Shared Toolchain Decision](../../_shared/decisions/SUI-DEC-003-shared-toolchain.md)
- [Owner Approval Bundle V0](../../.agents/authority/OWNER-AUTH-V0.md)
- [.NET support policy](https://dotnet.microsoft.com/en-us/platform/support/policy)
- [Windows application development](https://learn.microsoft.com/windows/apps/)
- [Windows App SDK release channels](https://learn.microsoft.com/windows/apps/windows-app-sdk/stable-channel)
- [Windows packaging and deployment](https://learn.microsoft.com/windows/apps/package-and-deploy/)
- [Rust platform support](https://doc.rust-lang.org/rustc/platform-support.html)
- [Protocol Buffers editions](https://protobuf.dev/programming-guides/editions/)
- [SQLite WAL](https://www.sqlite.org/wal.html)
