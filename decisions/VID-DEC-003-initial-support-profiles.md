# VID-DEC-003 - Initial Support Profiles

| Field | Value |
| --- | --- |
| Decision ID | `VID-DEC-003` |
| Status | Accepted |
| Revision | `1.0` |
| Decision owner | Product Lead: Video (`primary`) |
| Required reviewers | Domain Architect: Video; Suite Architect; Security Lead; QA and Compatibility Lead; Release Engineering Lead |
| Advisory reviewers | Media and UX Researcher; Research Triage |
| Authority | `OWNER-AUTH-V2` |
| Blocker | `Phase:V1`; `Phase:V2`; `Phase:V7A` |
| Depends on | `VID-DEC-001@1.0` Accepted |
| Immediate consumer | `VID-IMPL-P00-002B` |

## Decision Question

Which exact initial media, timed-text, encoder, AI-provider, and hardware profiles may Video advertise or execute without turning detection into support, importing later scope, redistributing unapproved payloads, or weakening the hostile-ingest boundary?

## Decision

Revision `1.0` selects one internal/private Phase 00 **probe qualification profile only**. It promotes no playback, import-acceptance, timed-text interchange, encoder, AI-provider, accelerator, hardware, mux, burn-in, or delivery profile.

| Profile family | Decision in this revision |
| --- | --- |
| Media probe qualification | Select `VID-PROBE-P00-001` below |
| Playback/decoder/renderer | None promoted |
| Timed-text import/export | None promoted; UTF-8 SRT proof remains separately gated |
| Encoder/mux/burn-in | None promoted |
| AI/provider/model | None promoted |
| GPU/NPU/hardware backend | None promoted; CPU-only functional baseline remains |

No capability may inherit support from the selected probe component. Probe, play, import, edit, waveform, proxy, subtitle extraction, export, mux, encode, and validated delivery remain independent claims.

## Selected Profile: VID-PROBE-P00-001

### Component and Acquisition

- Component role: standalone `ffprobe.exe` process used only to produce a bounded immutable observation.
- Upstream source release: FFmpeg `8.1.2` "Hoare", released 2026-06-17; FFmpeg publishes signed source, not official Windows binaries.
- Qualification build: third-party Gyan full build already installed by the owner.
- Exact executable path: `C:\ProgramData\chocolatey\lib\ffmpeg-full\tools\ffmpeg\bin\ffprobe.exe`.
- Exact executable SHA-256: `9DF3B0B5275E830961DF6D94E1F7A71121A7ABD5FF708E9FEC8A0B6084A55015`.
- Observed version: `8.1.2-full_build-www.gyan.dev`.
- Configuration classification: includes `--enable-gpl --enable-version3`; treat as a GPLv3 full build with a broad protocol/codec surface.
- Acquisition boundary: owner-installed, internal qualification only. Its dependency classification is `External only` under the Video third-party dependency policy. The executable MUST NOT be copied into this repository, bundled, uploaded, published, redistributed, or represented as an LGPL build.
- The Chocolatey shim at `C:\ProgramData\chocolatey\bin\ffprobe.exe` is not the registered executable and MUST NOT be invoked or hashed as the component identity.

Any path, hash, version, or configuration mismatch fails closed as `VID-INGEST-IDENTITY-MISMATCH`. Automatic substitution, PATH lookup, silent update acceptance, and "latest" resolution are prohibited.

### Operation Boundary

- Operation ID: `submark.video-localization.media.probe`.
- Operation version: `1.0.0-p00`.
- Implementation profile ID: `VID-PROBE-P00-001`.
- Authority class: immutable candidate observation only.
- Input: one caller-leased, preflighted, immutable local regular file with full SHA-256 and byte length from the accepted `VID-IMPL-P00-002A` boundary. The adapter MUST use an attempt-scoped immutable staged copy held under a sharing policy that denies write, delete, rename, replacement, and relink for the full child lifetime. It MUST verify full SHA-256, byte length, and file identity immediately before launch and after child exit. Any mismatch fails as `VID-INGEST-IDENTITY-MISMATCH`; no observation may be published.
- Output: one schema-validated normalized observation or one structured terminal disposition.
- The worker and child process receive no project-store, Shared-store, credential, user-profile, unrelated-file, or publication authority.
- A product-owned command may later bind a validated observation to a stable Video asset identity; execution success never performs that binding.

### Fixed Invocation Policy

The reviewed adapter launches the exact absolute executable directly with an argv array. It MUST NOT use a shell, command string, user-controlled option, URL, playlist, environment expansion, or PATH lookup.

The profile requires:

- `-v error`
- `-hide_banner`
- `-protocol_whitelist file`
- `-format_whitelist matroska` for the selected synthetic qualification fixture
- `-show_format`
- `-show_streams`
- `-show_chapters`
- `-of json`
- exactly one final preflighted local input path

The adapter MUST apply an explicit output-field/schema allowlist after parsing. Unknown fields are ignored only when declared optional; invalid, deeply nested, duplicate, oversized, or schema-incompatible output fails safely. Packet/frame enumeration, payload dumps, arbitrary command options, and attachment extraction are outside this operation.

### Qualification Fixture Profile

`VID-PROBE-FIXTURE-P00-001` is a locally generated, synthetic, rights-safe Matroska fixture with:

- one FFV1 video stream;
- one 48 kHz stereo PCM signed-16 audio stream;
- one UTF-8 SubRip subtitle stream;
- deterministic duration and stream/time-base expectations;
- no network reference, external playlist, DRM, private media, licensed font payload, or customer data.

This fixture qualifies only structural probe observation. FFV1, PCM, Matroska, and SubRip detection does not promote playback, subtitle import, rendering, extraction, round-trip, or delivery support.

### Resource and Containment Profile

- Windows 11 x64 internal qualification host.
- One Rust media worker and one declared `ffprobe.exe` child.
- Wall-clock and child CPU ceiling: 60 seconds. A worker-side monotonic watchdog enforces wall time independently of Job Object CPU accounting and terminates the complete job on expiry.
- Resident-memory ceiling: 1 GiB.
- Control envelope and retained normalized JSON ceiling: 1 MiB before decode.
- Stream count ceiling: 256.
- Stdout and stderr are captured separately, bounded, sanitized, and never interpreted as command text.
- Job Object enforcement owns process count, kill-on-close, CPU, and memory limits.
- Job Objects do not deny network. A separate OS-level deny control is mandatory and must produce evidence before a successful conformance claim.
- `-protocol_whitelist file` is defense in depth, not a replacement for OS-level network denial.
- Child handles and environment variables are allowlisted; write access is limited to an attempt-scoped staging directory when output is required.

If OS-level network denial, resource enforcement, executable identity verification, input lease, or output bounding cannot be established, the operation MUST fail closed before invoking `ffprobe`.

## Structured Outcomes

The adapter maps bounded process and validation results to stable Video dispositions:

- `VID-INGEST-UNSUPPORTED`
- `VID-INGEST-MALFORMED`
- `VID-INGEST-LIMIT`
- `VID-INGEST-PROTECTED`
- `VID-INGEST-QUARANTINED`
- `VID-INGEST-CANCELLED`
- `VID-INGEST-WORKER-FAILED`
- `VID-INGEST-IDENTITY-MISMATCH`
- `VID-INGEST-REVIEW`

Raw exit codes and untrusted stderr are diagnostic evidence, not public disposition identities. Deterministic input/schema/limit failures do not retry automatically. Cancellation, timeout, crash, malformed JSON, excess output, source mutation, stale attempt, or component mismatch cannot publish a complete observation.

## Shared Admission Gate for VID-IMPL-P00-002B

This decision selects a component and containment profile; it does not by itself publish a complete `SUI-SPEC-003` operation contract or authorize Shared job admission. `VID-IMPL-P00-002B` may build and test the adapter locally, but Shared admission and any `Implemented` compatibility claim remain fail-closed until one implementation-freeze bundle records exact canonical digests for all rows below.

| Required binding | Frozen Phase 00 requirement |
| --- | --- |
| Operation descriptor | Bind `submark.video-localization.media.probe@1.0.0-p00` to exact input, output, parameter, error, resource, policy, and output-contract schema IDs, versions, and canonical SHA-256 digests. |
| Determinism and cache | Deterministic only for the exact input bytes, canonical parameters, executable/configuration identity, adapter build, policy, and resource profile; cache is disabled in Phase 00. |
| Capabilities | Cancellation and deadline are required; progress, checkpoint, pause/resume, degraded output, partial success, acceleration, network, and automatic fallback are unsupported. |
| Input manifest | Record source/content SHA-256, byte length, staged-copy identity, lease ID, lease epoch, fencing token, access scope, expiry, manifest digest, and original caller-owned logical source reference. A mutable path is never identity. |
| Attempt identity | Record Job ID, Attempt ID, Dispatch ID, Worker Instance ID, stable Implementation ID, operation/profile IDs, executable/configuration hashes, selected compatibility set, and resource/policy digests. Worker restart creates a new Worker Instance ID and stale lease or fencing values fail closed. |
| Worker handshake | `WorkerHello` advertises exact worker/control protocol ranges, operation and schema versions, implementation and executable identities, Windows x64 isolation profile, limits, and selected local transport. Incompatible combinations are rejected before dispatch. |
| Transport | Input media is out-of-band through the leased staged local file. Normalized observation is the required `probe_observation` output and may be in-band only after the 1 MiB pre-decode envelope check. Raw stdout/stderr are bounded transient diagnostics and are not published artifacts. |
| Artifact and publication | A complete normalized observation is staged application-private with type/schema, content hash/size, completeness, output port, producing identities, input/policy/resource digests, Publication ID, and fencing token. Atomic publication occurs only after schema, integrity, lease, source, and epoch validation. Failed, partial, cancelled, timed-out, stale, or mismatched output is quarantined or deleted under bounded retention and never satisfies success. Product acceptance remains a later Video command. |
| Error contract | Every terminal failure emits a versioned Shared `StructuredError` with complete `ErrorRef`, operation stage, applicable job/attempt/dispatch/worker/publication references, safe bounded details and causes, plus exactly one Video disposition below. Raw stderr and exit codes remain restricted evidence. |

The operation bundle MUST map all nine Video dispositions before admission. `VID-INGEST-CANCELLED` maps to Shared `Cancelled`; watchdog expiry maps to Shared `TimedOut` with `VID-INGEST-LIMIT`; every other disposition maps to Shared `Failed` unless a product-owned review workflow records `VID-INGEST-REVIEW` outside worker execution. `UNSUPPORTED`, `MALFORMED`, `LIMIT`, `PROTECTED`, `QUARANTINED`, and `IDENTITY-MISMATCH` are non-retryable for unchanged input/profile. `WORKER-FAILED` may retry only when the complete Shared retry policy classifies the cause as transient, creates a new Attempt and Dispatch, preserves the Job, and remains within cumulative limits. No other automatic retry is permitted.

## Required Evidence Before Implemented or Verified

1. Exact executable path, SHA-256, verbatim version and configuration output plus their hashes, acquisition record, `External only` classification, license classification, and no-redistribution statement.
2. Fixed argv adapter tests proving no shell, PATH lookup, URL, user option, or implicit protocol path.
3. OS-level network-denial evidence independent of Job Object and protocol allowlisting.
4. Job Object evidence for process count, kill-on-close, CPU, memory, timeout, cancellation, and child cleanup.
5. Positive synthetic fixture evidence and negative malformed, unsupported, protocol-attempt, oversized-output, stream-limit, timeout, cancellation, crash, executable-mismatch, source-mutation, and invalid-JSON evidence.
6. Output schema, field limits, rational parsing, provenance, component identity, input fingerprint, operation identity, and observation-only authority checks.
7. Confirmation that no binary, private media, licensed payload, or unapproved third-party artifact entered the repository or GitHub.
8. Exact-hash operation-descriptor bundle satisfying the Shared admission gate above, including schema, transport, identity, error, artifact, publication, lease-race, stale-worker, and restart evidence.

`Implemented` means the bounded operation exists. `Verified` additionally requires the exact profile and negative evidence above. Neither status promotes a media support claim.

## Alternatives Rejected for This Revision

| Alternative | Disposition | Reason |
| --- | --- | --- |
| Project-bundled current Gyan full build | Rejected | GPLv3 full build, broad attack surface, and no redistribution authority |
| Project-managed LGPL Windows build | Deferred | Requires a separately acquired/minimized build, exact configuration/hash, update and notice process, and legal/distribution review |
| Windows Media Foundation primary probe | Deferred | Removes third-party acquisition but not hostile-parser containment; behavior/profile and fixture evidence remain unresolved |
| Custom Rust container parser | Rejected | Creates new parser security and compatibility burden without reducing Phase 00 risk |
| PATH-resolved or unpinned ffprobe | Rejected | Non-reproducible component identity and binary-planting/update risk |

## Consequences

Positive:

- `VID-IMPL-P00-002B` can implement a real, exact, no-redistribution probe qualification path.
- Probe support remains separate from playback and delivery claims.
- The local executable is immutable by path/hash/configuration and can be rejected after update rather than silently changing behavior.

Costs and risks:

- The selected build is intentionally not shippable.
- The full GPL build has a larger compiled surface than a future minimized LGPL probe build.
- OS-level network denial and Windows child containment are implementation work, not properties of `ffprobe` or Job Objects.
- Updating the local component requires a new reviewed profile revision and evidence.
- Arm's-length invocation of the GPLv3 build from the source-available application remains an unresolved legal-interpretation risk while `SUI-DEC-001` is open. Any incompatible resolution retires this profile and requires a separately reviewed non-GPL/minimized build or Media Foundation profile.

## Release, CI, and Lifecycle Boundary

- This profile is intentionally machine-local and is not eligible for GitHub Actions, contributor-default setup, release packaging, release artifacts, support claims, or automated download. CI MUST skip it unless a separately approved private runner already holds the exact owner-managed binary and records the same identity evidence.
- No workflow may fetch, cache, upload, or redistribute the selected executable under this decision. A portable/team/release profile requires a new reviewed acquisition, license, notice, hash, containment, and distribution decision.
- Security advisories, component/configuration drift, owner removal, incompatible `SUI-DEC-001` resolution, or a required production/release path retire this revision immediately and trigger a new candidate review. Silent rotation is prohibited.
- Release notes may cite this decision only as internal Phase 00 qualification evidence and MUST NOT imply that a shipped artifact contains or supports the selected component.

## Review and Acceptance Rule

All required reviewers must review the same candidate SHA-256. Owner-authorized consolidated review is permitted but must be labeled non-independent. Independent review, when performed, remains separately identified. Acceptance of this decision does not resolve `SUI-DEC-001`, grant redistribution rights, or approve later support-profile families.

## References

- [Video implementation stack](VID-DEC-001-implementation-stack.md)
- [Foundation PRD](../specifications/VID-SPEC-001-foundation-prd.md)
- [Domain schema](../specifications/VID-SPEC-002-domain-schema.md)
- [Hostile media-ingest threat model](../specifications/VID-SPEC-004-media-ingest-threat-model.md)
- [Foundation verification](../specifications/VID-SPEC-015-foundation-verification.md)
- [Shared worker protocol](../../_shared/specifications/SUI-SPEC-003-job-worker-artifact-protocol.md)
- [Shared threat model](../../_shared/specifications/SUI-SPEC-008-shared-threat-model.md)
- [Accepted local preflight and fingerprint review](../../.agents/reviews/VID-IMPL-P00-002A/primary-acceptance.md)
- [Video source and governance](../Plan/sections/01_SOURCE_AND_GOVERNANCE.md)
- [Shared source-license decision](../../_shared/decisions/SUI-DEC-001-source-license.md)
- [FFmpeg download and signed releases](https://ffmpeg.org/download.html)
- [FFprobe documentation](https://ffmpeg.org/ffprobe.html)
- [FFmpeg protocols](https://ffmpeg.org/ffmpeg-protocols.html)
- [FFmpeg legal](https://ffmpeg.org/legal.html)
- [FFmpeg security](https://ffmpeg.org/security.html)
