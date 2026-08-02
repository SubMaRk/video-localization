# VID-SPEC-015 - Foundation Verification Contract

| Field | Value |
| --- | --- |
| Specification ID | `VID-SPEC-015` |
| Status | Frozen |
| Revision | `1.0` |
| Owner role | QA and Compatibility Lead |
| Required reviewers | Product Lead: Video; Domain Architect: Video; Security Lead; Accessibility Lead; Internationalization Lead; Release Engineering Lead |
| Blocker | `P0:V0`; `Exit:V0` |
| Depends on | `VID-SPEC-001@1.0`; `VID-SPEC-002@1.0`; `VID-SPEC-003@1.0`; `VID-SPEC-004@1.0`; `VID-SPEC-014@1.0` Frozen |
| Authority | `OWNER-AUTH-V2` |

Only a Frozen revision satisfies the Phase 00 entry requirement. Freezing this contract fixes profiles, thresholds, corpus governance, procedures, and required evidence; it does not satisfy `Exit:V0`. Exit requires runtime evidence from the implementation under the exact frozen profiles.

## Normative Language

`MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, and `MAY` express requirement strength. The referenced machine-readable environment, budget, corpus, and fault assets are normative.

## Evidence Levels

| Level | Meaning | Phase effect |
| --- | --- | --- |
| `Entry` | Contract, fixture identity/generator, expected outcome, environment, metric, threshold, and evidence path are frozen | Opens implementation only after the complete P0 audit passes |
| `Implemented` | A bounded implementation exists and produces raw evidence | Does not satisfy Phase 00 exit alone |
| `Verified` | Reproducible run passes the exact profile and binds code/build/config/input/output hashes | May satisfy an individual exit gate |
| `Exit` | Every required gate passes, no invalidating blocker remains, and the exit manifest is approved | Permits Phase 00 completion |

Synthetic planning checks MUST NOT be labeled runtime verification. A missing run, unsupported environment, skipped case, flaky result, indirect proxy, or stale build is not a pass.

## Frozen Assets

| Asset | Purpose |
| --- | --- |
| [Environment matrix](../testing/conformance/vid-spec-015/profiles/environment-matrix.json) | OS, architecture, hardware, storage, locale, theme, density, and support tiers |
| [Foundation budgets](../testing/conformance/vid-spec-015/profiles/foundation-budgets.json) | Metrics, percentile/statistic, workload, threshold, and priority |
| [Corpus manifest](../testing/conformance/vid-spec-015/fixtures/foundation-corpus.json) | Rights/provenance fields, generators, fixture classes, and expected outcomes |
| [Fault matrix](../testing/conformance/vid-spec-015/fixtures/fault-matrix.json) | Failure point, injection boundary, invariant, expected recovery, and evidence |

## Scope and Release Allocation

The entry profile supports only the internal/private Phase 00 and `v0.1.0-alpha` technical subtitle slice: Windows 11 x64, per-user, offline CPU path, project create/save/reopen, stable identities, one later-promoted baseline media profile, playback foundation, waveform foundation, one manual cue, deterministic QC foundation, SRT export foundation, crash recovery, and the English/Thai System/Light/Dark shell.

Professional editor completeness, broad formats, proxy/burn-in, AI, Teams, plugins, visual text, public distribution, other operating systems, ARM64, and production support claims remain outside this profile. Capability names in planning are not support claims.

## Environment and Support Matrix

- `WIN11-X64-BASE` is the required qualification profile: Windows 11 x64 version 24H2 or 25H2 on a currently serviced build, 4 physical/8 logical CPU cores, 16 GiB RAM, integrated or software-capable graphics, 1920x1080 display, NTFS local project/staging storage, and 100 GiB free test space.
- `WIN11-X64-REF` is the reference profile: currently serviced Windows 11 25H2 x64, 8 physical cores, 32 GiB RAM, Direct3D 12-capable GPU, 2560x1440 display, and local NVMe storage.
- `WIN11-X64-LOW` is safe-failure only: 4 logical cores, 8 GiB RAM, 1366x768 display, and 20 GiB free local storage. It is not a performance-support claim; it proves actionable refusal, bounded memory, CPU fallback where allocated, and project readability.
- Every run records edition, version, build, patch date, architecture, CPU, RAM, GPU/driver, display/scale, filesystem, storage class/free space, power mode, locale, theme, density, package mode, and exact build identity.
- OS servicing status is checked against Microsoft release lifecycle at qualification time. An out-of-service build cannot produce current release evidence.
- Network shares, removable active project databases, Windows preview/Insider builds, ARM64, macOS, Linux, virtualized GPU, and remote desktop are non-qualifying unless a later profile promotes them.

## Corpus Governance

- Repository fixtures MUST be synthetic, self-created with documented rights, public-domain, or under an explicit license permitting repository redistribution and testing.
- Private customer media, copyrighted commercial media, restricted subtitle/font/model payloads, credentials, and personal data MUST NOT enter the repository corpus.
- Every fixture record includes stable ID, family, generator/source, source revision, license or rights basis, redistribution status, privacy class, byte/hash identity when materialized, expected result, tolerance, owning gate, and retention policy.
- A planned generator record is valid `Entry` evidence but not a materialized fixture or runtime pass.
- Security reproductions under embargo remain outside the repository with a non-sensitive generator or regression surrogate added before promotion.
- Fixture replacement preserves the old identity/evidence history and receives a new revision/hash; expected outcomes cannot be silently weakened to match an implementation defect.

## Foundation Workloads

| Workload ID | Frozen workload |
| --- | --- |
| `VID-WL-PROJECT-SMALL` | One project, one media asset, two tracks, 100 cues, 1,000 events |
| `VID-WL-PROJECT-FOUNDATION` | One project, four assets, four tracks, 1,000 cues, 10,000 events, 500 findings |
| `VID-WL-MEDIA-CFR` | Generated 10-minute 1080p CFR source with stereo 48 kHz audio |
| `VID-WL-MEDIA-VFR` | Generated 30-minute source with deterministic nonuniform PTS and late-timeline anchors |
| `VID-WL-WAVEFORM` | Generated 60-minute stereo 48 kHz PCM-equivalent source |
| `VID-WL-LOCALE` | Five workspace routes, 18 components, English/Thai catalogs, Compact/Comfortable, System/Light/Dark |
| `VID-WL-RECOVERY` | 10,000 committed events, active WAL, one in-flight rebuildable artifact, one queued worker job |

Phase 01 owns the 10,000-cue professional editor workload; Phase 00 MUST NOT use a smaller workload to claim that Phase 01 target.

## Foundation Budgets

The machine-readable budget asset is authoritative. Unless stated otherwise, latency budgets are warm p95 over at least 30 measured iterations after 5 warmups on `WIN11-X64-BASE`; startup is cold p95 over 10 launches. Background measurements run with no unrelated user workload and record power mode.

- Interactive input and committed command acknowledgement are separate. A command is acknowledged only after its durable transaction boundary succeeds.
- UI frame pacing is measured from presentation timestamps, not average FPS alone.
- Playback and interactive editing have resource priority over background jobs.
- A result over budget fails the gate or requires a new reviewed profile; the threshold is not edited after observing the run.

## Migration and Durable Recovery

- The project format has an independent schema identity and migration chain. Every migration is deterministic, transactional, resumable or rollback-safe, and records source/target versions and build identity.
- Phase 00 verifies create/open at current schema, migration from the immediately previous fixture schema, failed migration with the original remaining readable, and rejection of unsupported future schema without mutation.
- Before migration, create and verify a recoverable backup or snapshot. Never delete the last readable project copy automatically.
- Acknowledged user commands survive force close according to the frozen RPO. Rebuildable cache/proxy/waveform loss cannot make project truth unreadable.
- Startup reconciles WAL, in-flight jobs, staging, partial artifacts, cache indexes, worker epochs, and missing optional resources into complete, safely retryable, quarantined, or actionable review states.
- Disk-full, permission loss, antivirus interference, corrupt cache, corrupt project copy, missing media, moved media, and stale fingerprints have distinct structured outcomes.

## Interoperability

- Phase 00 interoperability is bounded to the canonical project contract, generated Shared bindings, one promoted source-media profile when selected, and UTF-8 SRT output from the vertical slice.
- SRT evidence binds exact cue text, line breaks, integer/rational source time, destination rounding, encoding, newline, output hash, and conversion-loss findings.
- No player/editor interoperability claim exists until the exact third-party application/version and fixture profile passes and is added through change control.
- Import success, probe success, playback, edit, export, round-trip, mux, and burn-in remain separate claims.

## Fault Injection and Safe Outcomes

- Inject faults at named boundaries, never by corrupting unrelated user assets.
- Each case records trigger, pre-state, acknowledged revision, injected point, expected invariant, expected state/error, cleanup, restart procedure, and post-recovery integrity evidence.
- Required Phase 00 classes include worker crash, hang, cancellation, timeout, malformed result, application force close, power-loss surrogate, disk full, permission loss, partial write, interrupted atomic publication, damaged cache, migration failure, missing media, relink mismatch, invalid IPC, resource exhaustion, and unsafe protocol input.
- A safe failure preserves project truth, never promotes a partial artifact, reports a stable actionable code, and permits retry, repair, relink, Safe Mode, read-only recovery, or explicit abandonment as applicable.

## Required Exit Evidence

1. Build manifest with source revision, compiler/toolchain, dependency lock, configuration, package identity, and hashes.
2. Environment manifests for every required profile and matrix axis.
3. Materialized corpus manifest with rights and byte/hash provenance.
4. Raw measurement samples plus summarized statistics for every budget.
5. Migration, backup, restore, integrity, force-close, and unsupported-future-schema results.
6. Fault-injection results proving each invariant and safe outcome.
7. CFR/VFR source-to-export timing evidence with no cumulative drift.
8. English/Thai x System/Light/Dark evidence, keyboard/focus/IME smoke evidence, and state-preserving switches.
9. Hostile-input and worker-boundary evidence under `VID-SPEC-004` limits.
10. Shared adapter/binding compatibility and no-Video-import-in-Shared evidence.
11. Interoperability evidence for only the exact promoted Phase 00 profile.
12. Exit manifest mapping every roadmap criterion and PRD requirement to exact immutable evidence.

## Entry and Exit Gates

| Gate ID | Entry freeze requirement | Exit requirement |
| --- | --- | --- |
| VID-VERIFY-001 | Scope and release allocation exact | Build exposes only allocated claims |
| VID-VERIFY-002 | Environment/support profiles exact | Required profiles pass on serviced builds |
| VID-VERIFY-003 | Rights/provenance schema and safe defaults exact | Materialized corpus has complete rights records |
| VID-VERIFY-004 | Foundation workloads exact | Every required workload executes reproducibly |
| VID-VERIFY-005 | Metrics/statistics/thresholds exact | Every P0 budget passes |
| VID-VERIFY-006 | Migration/backup/rollback procedures exact | Current, previous, failed, and future-schema cases pass |
| VID-VERIFY-007 | Acknowledgement/RPO/reconciliation semantics exact | Force-close loses no more than one acknowledged command and preserves integrity |
| VID-VERIFY-008 | Fault matrix and safe outcomes exact | Every required fault class passes |
| VID-VERIFY-009 | CFR/VFR and source-authority tolerances exact | No cumulative cue drift and late anchors pass |
| VID-VERIFY-010 | Hostile-ingest limits and quarantine linkage exact | Runtime worker/security evidence passes |
| VID-VERIFY-011 | EN/TH, theme, density, accessibility, IME matrix exact | Allocated native evidence passes |
| VID-VERIFY-012 | Shared adapter and independent-product boundaries exact | Binding/conformance and isolation evidence passes |
| VID-VERIFY-013 | SRT and promoted-profile interoperability scope exact | Exact export/rounding/loss evidence passes |
| VID-VERIFY-014 | Exit evidence schema and traceability exact | Exit manifest has no missing, stale, skipped, or invalidating evidence |

## Entry Acceptance Record

| Role | Status | Required evidence |
| --- | --- | --- |
| QA and Compatibility Lead | Accepted | Profiles, corpus, budgets, faults, and evidence schema |
| Product Lead: Video | Accepted | Release allocation and exit-claim boundary |
| Domain Architect: Video | Accepted by owner-authorized consolidated review | Project/time/revision/migration/recovery invariants |
| Security Lead | Accepted by owner-authorized consolidated review | Rights-safe corpus, hostile-input and fault outcomes |
| Accessibility Lead | Accepted by owner-authorized consolidated review | Matrix and evidence ownership |
| Internationalization Lead | Accepted by owner-authorized consolidated review | Locale, EN/TH, IME, and complex-script ownership |
| Release Engineering Lead | Accepted by owner-authorized consolidated review | Build/environment identity and immutable evidence |

Exit approval remains pending until implementation evidence exists. Entry acceptance MUST NOT be represented as Phase 00 completion.

Entry acceptance is an owner-authorized consolidated self-review under `OWNER-AUTH-V2`; it is not independent review.

## References

- [Foundation PRD](VID-SPEC-001-foundation-prd.md)
- [Domain Schema](VID-SPEC-002-domain-schema.md)
- [Timebase and Coordinates](VID-SPEC-003-timebase-and-coordinates.md)
- [Hostile Media-Ingest Threat Model](VID-SPEC-004-media-ingest-threat-model.md)
- [Design and Localization](VID-SPEC-014-design-and-localization.md)
- [Phase 00 Foundations](../Plan/roadmap/PHASE_00_FOUNDATIONS.md)
- [Video release allocation](../../_shared/Plan/release-planning/06_VIDEO_LOCALIZATION_RELEASE_PLAN.md)
- [Video capability/version matrix](../../_shared/Plan/release-planning/17_VIDEO_CAPABILITY_VERSION_MATRIX.md)
- [Windows release health](https://learn.microsoft.com/windows/release-health/windows11-release-information)
