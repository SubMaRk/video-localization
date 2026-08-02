# VID-SPEC-001 - Foundation and Manual Editor Product Requirements

| Field | Value |
| --- | --- |
| Specification ID | `VID-SPEC-001` |
| Status | Frozen |
| Revision | `1.0` |
| Owner | Product Lead: Video |
| Required reviewer roles | Domain Architect: Video; Shared Platform Lead; Security Lead; UX and Design Lead; Internationalization Lead; Accessibility Lead; QA and Compatibility Lead; Release Engineering Lead |
| Blocker | `P0:V0` |
| Depends on | `SUI-DEC-008@1.0` Accepted |
| GitHub work item | `SubMaRk/video-localization#19` |
| Normative scope | Product requirements and release boundaries for Video Phase 00 and Phase 01 |

## Status and Approval Dependency

`SUI-DEC-008@1.0` accepts Product Planning Baseline v1.0 and authorizes Specification Freeze work. It does not freeze this PRD or authorize implementation.

This revision is an editable first draft. It becomes a freeze candidate only after every requirement has downstream ownership, all open questions are resolved or explicitly deferred, and the required reviewers approve the same immutable revision. Only the suite Specification and Decision Register may change its status to `Frozen`.

## Normative Language

The terms **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

This PRD fixes product behavior, boundaries, and acceptance intent. It does not select an implementation language, desktop framework, database, media stack, project extension, package identity, or exact promoted format profile. Those choices remain controlled by registered decisions and downstream specifications.

## Product Promise

Video Localization Studio MUST become a Windows-first, subtitle-first, local-first workstation that remains useful when AI, network access, Teams, plugins, optional models, and accelerators are unavailable. The first stable workflow MUST let a professional user import supported media, create and time subtitles accurately, run deterministic QC, and produce validated output on CPU-only hardware.

The product MUST preserve human authority over text, timing, style, position, review, approval, and delivery. Automated or provider-derived results MUST remain inspectable observations or proposals and MUST NOT silently overwrite approved human work.

## Target Users and Outcomes

| User | Required outcome |
| --- | --- |
| Subtitle editor | Import media, navigate accurately, author and time cues, use keyboard-first editing, run QC, and export without AI |
| Translator/localizer | Work with Unicode and IME text, including English and Thai, without losing approved content or timing identity |
| Reviewer | Inspect exact cue revisions, findings, changes, recovery state, and delivery warnings before approval |
| Solo operator | Complete the core workflow offline on CPU-only Windows hardware and recover safely from interruption |
| Release operator | Produce output tied to an exact project revision, profile, tool identity, settings, and validation record |

## Scope and Release Boundaries

| Boundary | Required outcome | Excluded from the boundary |
| --- | --- | --- |
| Phase 00 | Prove durable project, timing, revision, worker, recovery, security, UI, and localization foundations through a thin offline subtitle slice | Professional editor completeness, AI, Teams, visual text, broad format support |
| `v0.1.0-alpha` | Create/save project, ingest one frozen media profile, play, build waveform, create a manual cue, run deterministic QC, export SRT, recover from a crash, and use the English/Thai System/Light/Dark shell on CPU | Production polish and broad compatibility claims |
| Phase 01 | Deliver a professional manual subtitle workflow that remains complete without AI | AI assistance, Teams, OCR, context intelligence, automation, unrestricted codec breadth |
| `v0.2.0-alpha` | Prove reliable playback and seeking, manual editing operations, correct CFR/VFR behavior, essential QC, recovery, relinking, and selected sidecar interchange | Proxy and burn-in are not required by this alpha allocation unless separately promoted |
| `v1.0.0` | Production-prove the offline CPU-capable Windows subtitle workstation and tested delivery profiles | AI, Teams, visual text, context, automation, and later capability packs |

No later release capability MAY enter Phase 00 or Phase 01 through an incidental dependency.

## Phase 00 Product Requirements

| ID | Requirement | Downstream authority or evidence |
| --- | --- | --- |
| `VID-PRD-F0-001` | The user MUST complete `import -> probe -> waveform -> manual cue -> deterministic QC -> SRT and ASS export` offline on the approved CPU-only Windows 11 x64 baseline. | `VID-SPEC-003`, `VID-SPEC-006`, `VID-SPEC-015` |
| `VID-PRD-F0-002` | Project create, open, save, autosave, migration, relink, recovery, and cache rebuild MUST preserve stable project, asset, track, cue, word, speaker, annotation, observation, revision, job, and artifact identities. | `VID-SPEC-002`, `VID-SPEC-015` |
| `VID-PRD-F0-003` | Durable project truth MUST be versioned and separated from rebuildable caches, temporary files, optional resources, credentials, private settings, and provider state. | `VID-SPEC-002`, `VID-DEC-002`, `SUI-SPEC-002` |
| `VID-PRD-F0-004` | Time MUST use integer values and explicit rational source time bases. VFR navigation and snapping MUST use source presentation timestamps and an explicit source-to-proxy map, never nominal-FPS arithmetic. | `VID-SPEC-003` |
| `VID-PRD-F0-005` | Save, reopen, relink, proxy mapping, migration, and cache rebuilding MUST preserve time anchors without cumulative cue drift. | `VID-SPEC-003`, `VID-SPEC-015` |
| `VID-PRD-F0-006` | Media, waveform, export, and other complex work MUST run outside the UI process through versioned operations with bounded resources, progress, cancellation, timeout, retry, checkpoint, crash handling, and structured errors. | `SUI-SPEC-003`, `VID-DEC-001`, `VID-SPEC-015` |
| `VID-PRD-F0-007` | Output-affecting operations MUST have stable identities and versions, canonical inputs, deterministic cache keys, atomic artifact publication, provenance, startup reconciliation, and force-recompute behavior. | `SUI-SPEC-003`, `VID-SPEC-002`, `VID-SPEC-015` |
| `VID-PRD-F0-008` | A worker crash, timeout, cancellation, malformed result, forced application closure, disk exhaustion, or damaged cache MUST NOT corrupt project truth or publish a partial artifact as complete. | `VID-SPEC-004`, `VID-SPEC-015` |
| `VID-PRD-F0-009` | Media, subtitle, attachment, font, archive, manifest, decoder, demuxer, converter, filesystem, and protocol inputs MUST be treated as untrusted and processed under explicit isolation, network, resource, quarantine, fuzzing, and recovery rules. | `VID-SPEC-004` |
| `VID-PRD-F0-010` | Automatic external URL resolution, unsafe media protocols, script or shell execution, traversal, device paths, symlink escape, and unbounded decompression MUST be disabled or rejected by the approved ingest profile. | `VID-SPEC-004` |
| `VID-PRD-F0-011` | Video MUST consume exact compatible Shared contracts through Video-owned adapters while retaining ownership of timebase, playback, subtitle, audio, encoding, QC, and delivery semantics. | `SUI-SPEC-001`, `VID-SPEC-002` |
| `VID-PRD-F0-012` | Video MUST remain independently installable and operable. Manga or Document installation, update, rollback, or removal MUST NOT alter Video's active dependency or resource closure. | `SUI-SPEC-001`, `VID-SPEC-015` |
| `VID-PRD-F0-013` | Fundamental project data MUST remain readable when optional models, plugins, providers, runtimes, caches, and Shared resources not required by the pinned project profile are absent. | `VID-SPEC-002`, `VID-SPEC-015` |
| `VID-PRD-F0-014` | The foundational observation-to-revision model MUST distinguish generated observations from human revisions and MUST protect approved human work even before AI providers are introduced. | `VID-SPEC-002` |
| `VID-PRD-F0-015` | The prototype shell MUST include the video stage, cue list, timeline, and inspector using primitive, semantic, and component tokens in System, Light, and Dark modes. | `VID-SPEC-014` |
| `VID-PRD-F0-016` | Theme switching MUST preserve project, workspace, focus, selection, and playhead state; Light and Dark MUST receive equal contrast, focus, accessibility, and screenshot-regression coverage. | `VID-SPEC-014`, `VID-SPEC-015` |
| `VID-PRD-F0-017` | The application localization service MUST use stable message keys, English source and fallback catalogs, Thai catalogs, BCP 47 identifiers, placeholder validation, pseudo-localization, and missing-message checks. | `SUI-SPEC-007`, `VID-SPEC-014` |
| `VID-PRD-F0-018` | English and Thai application language MUST switch without closing the project or losing shell state. Application language MUST remain independent from project source and target languages. | `VID-SPEC-014`, `VID-SPEC-015` |
| `VID-PRD-F0-019` | Thai glyphs, combining marks, line height, IME composition, search, legal segmentation, shaped-width measurement, and preview-versus-delivery behavior MUST have maintained smoke or golden fixtures. | `VID-SPEC-014`, `VID-SPEC-015` |
| `VID-PRD-F0-020` | The Phase 00 corpus MUST cover CFR, VFR, rational time bases, long media, English, Thai, CJK, RTL, mixed script, hostile inputs, worker failure, forced closure, migration, relink, and cache damage with rights-cleared provenance. | `VID-SPEC-015` |

## Phase 01 Product Requirements

| ID | Requirement | Downstream authority or evidence |
| --- | --- | --- |
| `VID-PRD-F1-001` | The manual editor MUST support create, edit, split, merge, move, snap, offset, ripple, and batch timing while preserving stable cue lineage and protected locks. | `VID-SPEC-002`, `VID-SPEC-005` |
| `VID-PRD-F1-002` | Playback MUST provide reliable seek, frame step, A/B loop, audio scrub, source/proxy mapping, waveform access, and playback priority while background work runs. | `VID-SPEC-003`, `VID-SPEC-005`, `VID-SPEC-015-A` |
| `VID-PRD-F1-003` | Undo/redo, autosave, snapshots, revision comparison, relinking, recovery, and transactional edit groups MUST preserve acknowledged and approved work. | `VID-SPEC-002`, `VID-SPEC-005`, `VID-SPEC-015-A` |
| `VID-PRD-F1-004` | Deterministic timing proposals MUST be ordered, idempotent, VFR-correct, lock-aware, layer-aware, diffable, selectively acceptable, and attributable to the pass that changed each boundary. | `VID-SPEC-005` |
| `VID-PRD-F1-005` | Deterministic QC MUST use stable rule IDs, profile-owned thresholds, previewable fixes, one undoable command group, scoped exceptions, rerun deltas, and export severity gates. | `VID-SPEC-005`, `VID-SPEC-006` |
| `VID-PRD-F1-006` | Timed-text import and export MUST preserve native precision and safely pass-through supported unknown data, identify source encoding and timing precision, and report conversion loss before delivery. | `VID-SPEC-006` |
| `VID-PRD-F1-007` | Only exact media and timed-text profiles accepted by `VID-DEC-003` and frozen in `VID-SPEC-006` MAY be advertised as supported; probe, play, import, edit, round-trip, export, mux, stream-copy, encode, and validated delivery are independent claims. | `VID-DEC-003`, `VID-SPEC-006` |
| `VID-PRD-F1-008` | Preview and burn-in MUST use equivalent subtitle rendering, font, shaping, line-breaking, positioning, style, and destination-rounding semantics for each promoted profile. | `VID-SPEC-005`, `VID-SPEC-006`, `VID-SPEC-015-A` |
| `VID-PRD-F1-009` | The cue list and timeline MUST remain responsive with 10,000 cues, and playback MUST retain priority during proxy, waveform, QC, and export jobs under frozen environment and measurement profiles. | `VID-SPEC-005`, `VID-SPEC-015-A` |
| `VID-PRD-F1-010` | Keyboard-first navigation and editing MUST cover primary manual workflows with visible focus, searchable/customizable commands, IME-safe handling, and reduced-motion behavior. | `VID-SPEC-005`, `VID-SPEC-014` |
| `VID-PRD-F1-011` | The five primary workspaces and their production components MUST implement approved System, Light, and Dark themes, Compact and Comfortable densities, documented states, accessibility targets, and bounded rendering behavior. | `VID-SPEC-014`, `VID-SPEC-015-A` |
| `VID-PRD-F1-012` | Every critical Phase 01 workflow, validation message, error, recovery instruction, export warning, notification, and accessibility description MUST be complete in English and Thai with safe English fallback. | `VID-SPEC-014`, `VID-SPEC-015-A` |
| `VID-PRD-F1-013` | Unicode, Thai, CJK, RTL, mixed-script, no-whitespace, combining-mark, legacy-encoding, and IME fixtures MUST import, display, edit, search, time, preview, and export without silent data loss. | `VID-SPEC-005`, `VID-SPEC-006`, `VID-SPEC-015-A` |
| `VID-PRD-F1-014` | Every promoted parser, decoder, renderer, converter, mux, and delivery path MUST pass malformed, oversized, attachment-heavy, timeout, cancellation, crash, disk-full, unsafe-protocol, and partial-output tests within declared boundaries. | `VID-SPEC-004`, `VID-SPEC-006`, `VID-SPEC-015-A` |
| `VID-PRD-F1-015` | Every final artifact MUST identify the source project revision, selected delivery profile, QC disposition, actual tools and versions, settings, hashes, and reproducibility metadata required by that profile. | `VID-SPEC-006`, `VID-SPEC-015-A` |

## Cross-Cutting Invariants

1. Approved human work MUST NOT be silently overwritten.
2. The UI thread MUST NOT execute media, export, AI, network, or other unbounded work.
3. CPU-only completion of the core workflow MUST remain available.
4. Project truth MUST survive loss of rebuildable and optional components.
5. Stable identity MUST NOT be derived from display names, mutable paths, list positions, or nominal timestamps.
6. Large waveform, thumbnail, audio, and media payloads MUST NOT cross the general command channel as base64 JSON.
7. Unknown forward-compatible data MUST be preserved safely or rejected explicitly; it MUST NOT disappear silently.
8. Every support claim MUST name an exact profile and evidence revision.
9. Product code MUST NOT import Manga or Document domain types.
10. No release gate MAY be passed by prose alone when the register requires frozen thresholds, fixtures, environments, and evidence.

## Implementation Entry Gates

Phase 00 implementation MUST NOT begin until all applicable `P0:V0` blockers and their transitive Shared dependencies are `Frozen`, `Implemented`, `Verified`, or `Accepted` as required by the register. At minimum, the entry review MUST resolve:

- `VID-SPEC-001`, `VID-SPEC-002`, `VID-SPEC-003`, `VID-SPEC-004`, `VID-SPEC-014`, and the Phase 00 entry subset of `VID-SPEC-015`.
- `VID-DEC-001`, `VID-DEC-002`, and `VID-DEC-004`.
- The exact compatible Shared foundation subset including `SUI-SPEC-001`, `SUI-SPEC-002`, `SUI-SPEC-003`, `SUI-SPEC-007`, and `SUI-SPEC-008` where consumed by Video.
- Exact corpus, Windows, CPU, filesystem, locale, theme, language, dependency, and hostile-ingest profiles used by the first implementation slice.

Phase 01 implementation MUST additionally satisfy Phase 00 exit and freeze the applicable `VID-SPEC-005`, `VID-SPEC-006`, `VID-DEC-003`, and `VID-SPEC-015-A` entry requirements.

## PRD Acceptance Gates

| Gate ID | Pass condition | Evidence owner |
| --- | --- | --- |
| `VID-PRD-GATE-001` | Every `VID-PRD-F0-*` and `VID-PRD-F1-*` requirement maps to a registered specification, decision, fixture family, or explicit deferral. | Product Lead: Video |
| `VID-PRD-GATE-002` | The release allocation contains no AI, Teams, visual-text, context, automation, broad-format, or later-platform dependency in Phase 00/01. | Product Lead: Video; Release Engineering Lead |
| `VID-PRD-GATE-003` | Domain ownership review confirms Video owns timebase, playback, subtitle, audio, encoding, QC, and delivery while Shared remains domain-neutral. | Domain Architect: Video; Shared Platform Lead |
| `VID-PRD-GATE-004` | Security review confirms every ingest and worker boundary has downstream threat-model and failure-evidence ownership. | Security Lead |
| `VID-PRD-GATE-005` | UX, accessibility, and localization review confirms System/Light/Dark and English/Thai requirements are complete for the allocated phases. | UX and Design Lead; Internationalization Lead; Accessibility Lead |
| `VID-PRD-GATE-006` | QA review confirms every exit claim names a downstream corpus, environment, metric, tolerance, or a registered specification that must freeze those values. | QA and Compatibility Lead |
| `VID-PRD-GATE-007` | All required reviewer approvals bind the exact candidate revision and no tracked blocking comment remains open. | Product Lead: Video; Release Engineering Lead |

## Evidence and Traceability Rules

- Requirement evidence MUST name the requirement ID, exact specification revision, exact fixture or corpus revision, environment profile, result, and retained artifact.
- A passing draft fixture MUST NOT change a specification status to `Frozen`.
- Exact thresholds belong in `VID-SPEC-005`, `VID-SPEC-006`, `VID-SPEC-015`, or `VID-SPEC-015-A`, not in unversioned issue comments.
- GitHub issue `SubMaRk/video-localization#19` MAY mirror progress but MUST NOT override this specification or the suite register.
- Local `Plan/` files MUST NOT be uploaded to satisfy the GitHub issue.

## Explicit Non-Goals and Deferred Scope

- AI transcription, translation, diarization, model installation, and cloud providers.
- LAN or public-network Teams collaboration.
- OCR, visual-text tracking, restoration, and replacement.
- Story, context, recurring-region, and semantic retrieval capabilities.
- Recipes, node workflows, watch folders, headless automation, and agents.
- Broad professional format, codec, encoder, accelerator, sensor, STT, and hardware-backend claims.
- Windows ARM64, macOS, Linux, portable distribution, and machine-wide installation unless promoted by a later decision.
- Full nonlinear editing, arbitrary dockable UI, vendor-hosted SaaS, billing, SSO/SCIM, or automatic approval of probabilistic output.

## Open Review Work

1. Confirm that each downstream specification accepts the requirement IDs assigned to it.
2. Resolve `VID-DEC-001`, `VID-DEC-002`, and `VID-DEC-004` before a Phase 00 implementation-ready claim.
3. Bind exact Phase 00 and Phase 01 corpus, support, budget, profile, and failure-evidence identifiers in `VID-SPEC-015` and `VID-SPEC-015-A`.
4. Obtain all required reviewer approvals on one immutable candidate revision.

## References

- [Video Planning Index](../Plan/README.md)
- [Phase 00 - Foundations and Vertical Spike](../Plan/roadmap/PHASE_00_FOUNDATIONS.md)
- [Phase 01 - Manual Production Editor](../Plan/roadmap/PHASE_01_MANUAL_PRODUCTION_EDITOR.md)
- [Release Gates](../Plan/sections/10_RELEASE_GATES.md)
- [Next Planning Documents](../Plan/sections/11_NEXT_PLANNING_DOCUMENTS.md)
- [Video Release Plan](../../_shared/Plan/release-planning/06_VIDEO_LOCALIZATION_RELEASE_PLAN.md)
- [Video Capability-to-Version Matrix](../../_shared/Plan/release-planning/17_VIDEO_CAPABILITY_VERSION_MATRIX.md)
- [Specification and Decision Register](../../_shared/Plan/sections/09_SPECIFICATION_AND_DECISION_REGISTER.md)
- [`SUI-DEC-008@1.0`](../../_shared/decisions/SUI-DEC-008-planning-baseline-acceptance.md)
