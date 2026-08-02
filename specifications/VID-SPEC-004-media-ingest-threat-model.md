# VID-SPEC-004 - Hostile Media-Ingest Threat Model

| Field | Value |
| --- | --- |
| Specification ID | `VID-SPEC-004` |
| Status | Frozen |
| Revision | `1.0` |
| Owner role | Security Lead |
| Required reviewers | Product Lead: Video; Domain Architect: Video; QA and Compatibility Lead; Legal and Licensing Steward; Release Engineering Lead |
| Blocker | `P0:V0` |
| Depends on | `SUI-SPEC-008@1.0` Frozen |
| Authority | `OWNER-AUTH-V2` |

Only a Frozen revision satisfies `P0:V0`. This specification freezes planning controls and adversarial evidence requirements. It does not claim that a parser, decoder, sandbox, codec, format, font, optical-disc path, or media worker has been implemented or promoted.

## Normative Language

`MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, and `MAY` express requirement strength. Tables named Required Baseline, Security Limits, Threat Model, Failure Contract, and Security Gates are normative.

## Scope

All source media and sidecars are untrusted, including local files, removable media, network-share copies, archives, project packages, subtitle files and streams, attachments, fonts, playlists, optical-disc folders/images, metadata, thumbnails, proxies, and outputs from external tools.

This specification owns Video-specific probe, demux, decode, subtitle/font parse, optical-disc parse, waveform/thumbnail/proxy generation, conversion, and artifact-publication threats. `SUI-SPEC-008` owns generic package, IPC, worker, credential, logging, update, rights, and Shared-store controls. `VID-SPEC-006` and `VID-DEC-003` own exact promoted components and support claims.

Phase 00 is internal/private Windows 11 x64, per-user, offline-capable, and CPU-functional. DRM, encryption, region control, copy protection, access control, and signature protection are never bypassed. Protected media is unsupported unless the user provides a lawfully accessible unprotected source.

## Security Objectives

- Hostile input cannot mutate project truth, Shared metadata, credentials, application binaries, or unrelated files.
- Parser, decoder, font, subtitle, and media-tool failure remains inside the declared worker and job boundary.
- Input identity, source timebase, stream inventory, selected operation, component identity, limits, and result lineage remain inspectable.
- Unsupported, suspicious, partial, stale, or malformed output cannot become an approved artifact.
- Cancellation, timeout, crash, disk pressure, and application restart fail safely and preserve acknowledged commands.
- No file name, metadata field, subtitle payload, filter expression, or user path becomes an ad-hoc shell command.

## Trust Boundaries

1. File or source selection enters bounded preflight before any complex parser.
2. The Video core issues a typed operation to an isolated media worker through the accepted Shared IPC profile.
3. The worker receives only leased immutable inputs and an ACL-scoped unique output directory.
4. Complex libraries parse inside the worker; they receive no project-database, Shared-store, credential, home-directory, or ambient network authority.
5. Worker output crosses schema, size, identity, provenance, and semantic validation before atomic publication.
6. Product command handlers, never the worker, decide whether a validated observation or artifact affects project revisions.

## Required Baseline

### Identification and Preflight

- Identify candidates from bounded signature and structure checks; file extensions are hints only.
- Canonicalize paths through platform path APIs. Reject device paths, alternate data streams, parent traversal, absolute archive members, links, duplicate normalized names, case collisions, null bytes, and names outside the declared tree.
- Record source size, full identity when required, fast-fingerprint provenance, selected operation, and limits before complex parsing.
- A network location is copied or imported into a managed local staging area before authoritative processing. Workers MUST NOT open unmanaged network-share paths directly.
- Unknown, ambiguous, encrypted, protected, unsupported, or over-limit input returns a structured disposition before allocation or publication.

### Worker Isolation

- Probe, demux, decode, subtitle parsing, font inspection/shaping, optical-disc parsing, conversion, mux, burn-in, and attachment extraction run outside the UI process.
- The Windows base worker uses a restricted token, Job Object, explicit process/memory/time/output limits, dedicated ACL-scoped directories, inherited-handle allowlisting, and network denied by default.
- Workers MUST NOT receive project-store or Shared-store write handles. They publish typed candidate results only to dedicated staging.
- Child processes are denied unless the exact operation profile declares an executable identity, argument adapter, inherited-handle set, and equivalent containment.
- Process separation is crash containment, not a complete sandbox claim. Promotion requires platform enforcement evidence.

### Protocol, URL, and Command Safety

- Disable automatic external URL resolution, network protocols, scripts, external entities, unsafe media-tool protocol handlers, device capture, and implicit playlist fetching.
- Permit only local file and brokered-handle inputs in the Phase 00 profile.
- Keep paths, options, filters, and arguments structured until one reviewed adapter invokes a process or API. Shell invocation and concatenated command strings are prohibited.
- Subtitle markup, ASS/SSA commands, attachment names, font names, XML, chapters, metadata, and playlists are data; they cannot invoke code, network, filesystem, or environment expansion.

### Output and Publication

- Write each operation into a unique non-authoritative staging directory and enforce declared file count, byte, type, and name bounds.
- Flush and close candidate output, validate exact expected artifacts, compute identities, and attach source/operation/component lineage before atomic publication.
- Partial, unexpected, executable, stale-epoch, mismatched-source, over-limit, or schema-invalid output is quarantined or deleted according to evidence policy and never registered as complete.
- Cache deletion, worker death, and quarantine cannot delete project truth or the original user source.

## Security Limits

These are hard preflight and containment ceilings for the Phase 00 profile, not support claims. A lower operation-specific limit MAY apply. Exceeding a ceiling returns `VID-INGEST-LIMIT` or `VID-INGEST-REVIEW`; it never authorizes an unbounded attempt.

| Resource | Phase 00 ceiling |
| --- | --- |
| Source object size | 2 TiB per selected object |
| Probe wall time / worker CPU time | 60 s / 60 s |
| Probe resident memory | 1 GiB |
| Decode/conversion worker resident memory | 4 GiB or 50% of physical RAM, whichever is lower, with a 1 GiB minimum supported host requirement |
| Worker process count | 1 worker plus 2 declared child processes per operation |
| Control envelope | 1 MiB before decode; bulk media prohibited |
| Streams / programs / chapters / playlist entries | 256 / 256 / 100,000 / 100,000 |
| Attachments | 128 files, 64 MiB each, 512 MiB expanded total |
| Metadata | 64 MiB aggregate; 1 MiB per scalar field |
| Subtitle cues / cue text / markup depth | 10,000,000 / 1 MiB UTF-8 per cue / 64 levels |
| Frame geometry | 32,768 per dimension and 268,435,456 pixels per frame |
| Audio | 768 kHz, 128 channels, validated sample-count arithmetic |
| Declared duration | 1,000 hours; longer sources require a reviewed profile |
| Archive/structured-source depth and entries | 3 nested layers and 100,000 entries |
| Expanded staging output | min(2 TiB, declared estimate plus 10%, available disk minus 10 GiB recovery reserve) |
| Cancellation grace / forced termination | 2 s cooperative / 5 s total |

Every count, byte size, timestamp, stride, sample, frame, allocation, and multiplication MUST use checked arithmetic before allocation. VFR indexes use bounded disk-backed structures and source presentation timestamps; they MUST NOT derive durable time from nominal FPS.

## Input-Class Policies

### Containers, Streams, and Timing

- Validate stream counts, packet sizes, timestamp domains, rational denominators, ordering, discontinuities, wrap, negative values, and arithmetic before indexing.
- Preserve source evidence when timestamps are malformed or contradictory; do not silently repair durable time.
- Unsupported codecs or profiles fail as unsupported, not corrupt, unless structural evidence is invalid.

### Timed Text, Attachments, and Fonts

- Decode text with a declared or reviewable encoding result and bounded replacement policy; invalid sequences cannot become command text.
- Parse ASS/SSA overrides, karaoke, drawing, style, attachment, and font references with bounded grammar depth and payload size.
- Inspect fonts in the worker. Reject malformed tables, extreme counts, embedded executable content, unsafe external references, and rights-unknown embedding or redistribution.
- Font availability and license acceptance do not imply permission to package, transfer, embed, or redistribute.

### Optical Disc and Structured Sources

- Treat DVD/BDMV playlists, navigation data, angles, chapters, seamless branches, and referenced paths as hostile structured input.
- Resolve references only inside the selected source root after canonicalization and cycle/depth checks.
- Detect encryption/protection and return `VID-INGEST-PROTECTED`; do not invoke, recommend, bundle, or automate circumvention.

## Threat Model

| Threat ID | Abuse path | Impact | Required prevention | Detection/evidence | Priority |
| --- | --- | --- | --- | --- | --- |
| VID-MEDIA-THR-001 | Crafted container exploits probe/demux/decoder | Worker escape or code execution | Isolated restricted worker, pinned component, bounded probe, no ambient authority | Crash fingerprint, sandbox denial, component/build identity, fuzz regression | Critical |
| VID-MEDIA-THR-002 | Playlist, subtitle, metadata, or tool resolves external protocol | SSRF, disclosure, remote content substitution | Network deny, protocol allowlist limited to local handles, no automatic resolution | Network canary and denied-destination audit | Critical |
| VID-MEDIA-THR-003 | Archive, attachment, font, or disc path escapes staging | File overwrite, persistence, unrelated data access | Canonical relative paths, no links/device paths, unique ACL staging | Escape canaries, rejected-member inventory | Critical |
| VID-MEDIA-THR-004 | Extreme dimensions, counts, duration, nesting, compression, or output | CPU/RAM/GPU/disk exhaustion | Preflight ceilings, checked arithmetic, Job Object, quotas, recovery reserve | Limit code, peak resource record, no partial publication | High |
| VID-MEDIA-THR-005 | Malformed timestamp, rational, packet order, or wrap poisons index | Cue drift, overflow, corrupt durable timing | Exact rational validation, checked arithmetic, source PTS authority, no nominal VFR fallback | Timing anomaly report and retained source evidence | High |
| VID-MEDIA-THR-006 | ASS/SSA, XML, font, or attachment invokes script/external behavior | Code execution, network/file access | Data-only parsers, external entity/script disablement, worker inspection | Grammar/fuzz corpus and forbidden-handler audit | Critical |
| VID-MEDIA-THR-007 | Protected optical media triggers circumvention path | Legal/security exposure | Detect and reject protection; no decryption tool or invocation | Structured protected-media disposition | High |
| VID-MEDIA-THR-008 | Worker writes project/store or inherits excessive handles | Durable corruption or credential theft | No write handles, handle allowlist, restricted token, product command authority | Handle/access canaries and mutation audit | Critical |
| VID-MEDIA-THR-009 | Crash/cancel races publish partial or stale output | Corrupt cache or false-complete artifact | Unique staging, epoch/source binding, flush/validate/hash/atomic publish | Quarantine manifest and startup reconciliation | High |
| VID-MEDIA-THR-010 | Path or metadata reaches shell/filter command construction | Command injection | Structured APIs and reviewed argument adapter; shell prohibited | Metacharacter corpus and invocation audit | Critical |
| VID-MEDIA-THR-011 | Hostile text reaches logs, UI, or diagnostics | Secret/path disclosure or log injection | Structured bounded escaping and Shared redaction policy | Control-character/canary regression | High |
| VID-MEDIA-THR-012 | Result identity or lineage is substituted across jobs | Wrong source artifact accepted | Job epoch, source hash, operation version, component identity, typed result validation | Mismatch rejection and provenance audit | High |

## Failure Contract

| Code | Disposition | Durable effect | User recovery |
| --- | --- | --- | --- |
| `VID-INGEST-UNSUPPORTED` | Reject without quarantine | None | Select a promoted profile or convert externally |
| `VID-INGEST-MALFORMED` | Reject and retain bounded evidence | None | Inspect details or use another source |
| `VID-INGEST-LIMIT` | Terminate safely | None | Use a reviewed lower-cost operation/profile |
| `VID-INGEST-PROTECTED` | Reject | None | Provide a lawful unprotected source |
| `VID-INGEST-QUARANTINED` | Isolate candidate output | None | Review evidence; never open automatically |
| `VID-INGEST-CANCELLED` | Remove or quarantine partial output | None | Retry explicitly |
| `VID-INGEST-WORKER-FAILED` | Reconcile job and staging | None | Retry, Safe Mode, or alternate promoted component |
| `VID-INGEST-IDENTITY-MISMATCH` | Reject stale/substituted output | None | Re-probe the current source |
| `VID-INGEST-REVIEW` | No complex operation starts | None | Owner/operator approves a registered profile |

Errors include stable code, operation ID/version, job/trace ID, bounded source identity, failed gate, component/build identity when invoked, retry class, and safe recovery actions. Raw untrusted strings are escaped and bounded.

## Fuzzing and Adversarial Intake

- Maintain redistributable synthetic seeds for containers, streams, timestamps, subtitles, fonts, attachments, playlists, disc structures, paths, protocols, worker faults, and publication races.
- Keep private customer media and licensed payloads out of the repository corpus.
- Every promoted parser/decoder adapter supplies coverage-guided fuzz targets for its externally reachable grammar and structured output validator.
- Run sanitizers or equivalent memory/undefined-behavior checks where the component/toolchain supports them.
- Deduplicate crashes by component, build, stack/signature, input hash, operation, and sandbox profile; minimize only in an isolated workspace.
- A security fix adds a non-sensitive regression fixture or a reproducible generator before promotion resumes.

## Incident Response

1. Quarantine the input, candidate output, component profile, and affected cache entries without deleting project truth.
2. Record hashes, operation, component/build, worker profile, limits, crash and sandbox evidence, and affected versions.
3. Disable the exact operation/profile through signed or local emergency policy while preserving manual editing and read-only recovery.
4. Determine whether project/store mutation, credential access, network access, or filesystem escape occurred.
5. Patch, replace, or revoke the component; add corpus coverage and rerun the full promoted profile.
6. Restore promotion only through reviewed evidence and notify affected release/product owners.

## Security Gates

| Gate ID | Freeze requirement | Implementation evidence owner |
| --- | --- | --- |
| VID-MEDIA-SEC-001 | Trust boundaries and product/Shared ownership are exact | Security Lead |
| VID-MEDIA-SEC-002 | Signature preflight and structured path policy are exact | Domain Architect: Video |
| VID-MEDIA-SEC-003 | Windows worker authority and denied capabilities are exact | Security Lead |
| VID-MEDIA-SEC-004 | Protocol, URL, script, entity, and shell defaults deny unsafe behavior | Security Lead |
| VID-MEDIA-SEC-005 | Phase 00 security ceilings and checked arithmetic are exact | QA and Compatibility Lead |
| VID-MEDIA-SEC-006 | VFR/timebase hostile-input behavior preserves `VID-SPEC-003` | Domain Architect: Video |
| VID-MEDIA-SEC-007 | Subtitle, attachment, font, and optical-disc policies are exact | Security Lead |
| VID-MEDIA-SEC-008 | Staging, quarantine, atomic publication, and reconciliation are exact | QA and Compatibility Lead |
| VID-MEDIA-SEC-009 | Stable failure codes and safe recovery actions are exact | Product Lead: Video |
| VID-MEDIA-SEC-010 | Fuzz/adversarial intake and private-asset exclusion are exact | QA and Compatibility Lead |
| VID-MEDIA-SEC-011 | Incident response, disablement, revocation, and regression rules are exact | Security Lead |
| VID-MEDIA-SEC-012 | Rights/protection rules make no circumvention or redistribution claim | Legal and Licensing Steward |

Freeze evidence proves the planning contract and synthetic fixture coverage only. Runtime promotion requires exact component/build, OS, architecture, sandbox, operation, corpus, resource, crash, malformed-input, cancellation, disk-full, quarantine, and recovery evidence. Official distribution additionally requires approved dependency, license, patent, notice, SBOM, signing, update, and source-publication records.

## Acceptance Record

| Role | Status | Required evidence |
| --- | --- | --- |
| Security Lead | Accepted | Threat/control/gate completeness |
| Product Lead: Video | Accepted | Scope and recovery behavior |
| Domain Architect: Video | Accepted by owner-authorized consolidated review | Timebase, command authority, and artifact boundaries |
| QA and Compatibility Lead | Accepted by owner-authorized consolidated review | Limits, corpus, negative outcomes, and promotion evidence |
| Legal and Licensing Steward | Accepted by owner-authorized consolidated review | Confirms no circumvention or redistribution approval; source publication remains gated |
| Release Engineering Lead | Accepted by owner-authorized consolidated review | Component promotion remains release-gated |

This acceptance is an owner-authorized consolidated self-review under `OWNER-AUTH-V2`; it is not independent review and does not substitute for runtime security evidence.

## References

- [Shared Security and Privacy Threat Model](../../_shared/specifications/SUI-SPEC-008-shared-threat-model.md)
- [Video Core Architecture](../Plan/sections/04_CORE_ARCHITECTURE.md)
- [Format and Feature Expansion](../Plan/sections/09A_FORMAT_AUDIO_HARDWARE_AND_FEATURE_EXPANSION.md)
- [Source and Governance](../Plan/sections/01_SOURCE_AND_GOVERNANCE.md)
- [Phase 00 Foundations](../Plan/roadmap/PHASE_00_FOUNDATIONS.md)
- [Timebase and Coordinates](VID-SPEC-003-timebase-and-coordinates.md)
