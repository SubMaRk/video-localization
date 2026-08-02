# VID-SPEC-002: Canonical Video Domain Schema

| Field | Value |
| --- | --- |
| Specification ID | `VID-SPEC-002` |
| Status | Frozen |
| Revision | `1.0` |
| Owner | Domain Architect: Video |
| Required reviewers | Product Lead: Video; Shared Platform Lead; QA and Compatibility Lead; Security Lead |
| Blocker | `P0:V0` |
| Depends on | `SUI-SPEC-001@1.0`; `VID-SPEC-001@1.0` |
| Product namespace | `submark.video-localization` |

## Purpose

This specification freezes the implementation-neutral Video-owned schema for projects, collections, media assets, streams, tracks, cues, words, speakers, annotations, observations, revisions, and lineage. It defines identity, authority, references, invariants, versioning, safe failure, and conformance behavior without choosing a database layout, programming-language type, wire encoding, media library, or UI representation.

The contract consumes the Frozen Shared identity and project primitives in [SUI-SPEC-001](../../_shared/specifications/SUI-SPEC-001-shared-contract-catalog.md) and [SUI-SPEC-002](../../_shared/specifications/SUI-SPEC-002-project-and-revision-primitives.md). Product requirements are governed by [VID-SPEC-001](VID-SPEC-001-foundation-prd.md), while the accepted public identity is governed by [VID-DEC-002](../decisions/VID-DEC-002-project-package-name.md).

## Normative Language

`MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, and `MAY` express requirement strength. A conforming profile MUST map every applicable normative requirement to executable or inspectable evidence.

## Authority and Domain Boundary

Video owns all interpretation and mutation of media, stream, timebase, track, cue, word, speaker, subtitle, spatial annotation, observation, and Video delivery records.

- Shared Core MAY store, version, reference, migrate, and transport namespaced Video records as typed opaque product data.
- Shared Core MUST NOT infer cue timing, text segmentation, speaker identity, language meaning, media equivalence, stream selection, split/merge lineage, approval, or delivery readiness.
- Every authoritative mutation MUST enter through a versioned Video command handler and commit through the Frozen Shared project/revision boundary.
- A worker, provider, plugin, import adapter, probe, decoder, agent, or cache MUST NOT directly mutate authoritative Video state.
- Derived output begins as an observation, proposal, or artifact. Successful execution MUST NOT imply human approval.
- Video MUST NOT import Manga or Document domain schemas, and no Video project may require either application to be installed.

## Identity Grammar

All durable identities are opaque, stable, non-recycled, and compared independently of display text or paths. An identity MUST NOT encode a mutable name, list position, time value, frame number, filesystem path, database offset, or user-facing label.

| Identity | Scope |
| --- | --- |
| `VideoProjectId` | One Video project instance under the Shared Project ID |
| `CollectionId` | Product-owned series, season, episode-group, or user collection |
| `MediaAssetId` | One logical source, proxy, audio, attachment, or generated media asset |
| `StreamId` | One stream observation within an exact media-source revision |
| `TrackId` | One logical timed-text, transcript, caption, sign, metadata, or review track |
| `CueId` | One stable timed-text unit |
| `WordId` | One stable editorial token or grapheme-aware word unit within lineage |
| `SpeakerId` | One project-scoped speaker label entity; not biometric identity |
| `AnnotationId` | One typed annotation attached to exact targets and revisions |
| `ObservationId` | One immutable derived or imported claim with provenance |
| `VideoRevisionId` | One product revision committed through a Shared Revision |
| `LineageEdgeId` | One explicit source, split, merge, replacement, derivation, or rebinding edge |

Identity rules:

1. Moving, renaming, relinking, reopening, proxying, or rebuilding caches MUST preserve logical identities.
2. Forking a project MUST create a new project identity and record the source project and exact revision.
3. Deletion MUST retain a tombstone when a durable reference can outlive the object.
4. A deleted identity MUST NOT be assigned to another logical object.
5. Imports MUST allocate product identities; external ordinals and format IDs remain provenance, not authoritative identity.
6. Duplicate content hashes MAY identify equal bytes but MUST NOT collapse independently governed logical assets without an explicit command.

## Canonical Reference

A durable `VideoObjectRef` MUST include:

- Product namespace `submark.video-localization`.
- Video Project ID and Shared Project ID.
- Object kind and stable Object ID.
- Exact Video Revision ID and compatible schema identity.
- Required lineage or source-authority qualifier when interpretation depends on it.

A moving query MAY omit an exact revision only while it remains non-durable. Before an annotation, job, artifact, approval, finding, event, export, or migration is committed, every target MUST resolve to exact revision-bound references.

Missing, stale, deleted, wrong-project, wrong-product, ambiguous, or incompatible references MUST produce structured states. They MUST NOT silently bind to a same-named, nearby, overlapping, or similar object.

## Video Project Record

A `VideoProject` MUST contain:

- Shared Project Identity and Video Project ID.
- Product namespace and exact Video schema revision.
- Current Shared Revision and Video Revision.
- Collection memberships without making collections identity authorities.
- Source-authority inventory.
- Media asset, stream-observation, track, speaker, annotation, and lineage inventories.
- Required contract ranges and extension-preservation policy.
- Project language declarations kept separate from application UI locale.
- Current durable health, migration, and read/write capability state.

Project truth MUST remain separate from proxies, waveform tiles, thumbnails, indexes, embeddings, temporary exports, and other rebuildable artifacts. A project MUST remain readable when optional workers, models, providers, plugins, or caches are absent.

## Collection Record

A `Collection` groups projects or assets as series, season, episode group, production batch, or user-defined set.

- Membership MUST use stable exact references.
- Collection order and labels MUST NOT change member identity.
- Removing membership MUST NOT delete the member.
- A collection MUST NOT imply editorial approval, chronology, or source equivalence unless a versioned Video command records that meaning.
- Cross-project references MUST declare visibility, authority, and revision scope.

## Media Asset and Source Authority

A `MediaAsset` MUST record:

- Media Asset ID, kind, role, and exact asset-schema version.
- Source-authority mode: embedded, managed, external, generated, or proxy.
- Content fingerprint or immutable source revision when available.
- Structured locator hints separate from authority.
- Size and integrity metadata where bytes are managed.
- Probe observation references and selected authoritative source profile.
- Relationship to source, proxy, extracted audio, attachment, or generated derivative.
- Relink and replacement policy.
- Rights/provenance classification without embedding licensed payloads in evidence.

Relinking MUST verify the approved source-identity procedure. A same filename, duration, or partial fingerprint is not sufficient authority. Source replacement MUST be an explicit command and MUST mark dependent observations, mappings, QC, previews, and delivery results stale according to registered impact rules.

A `StreamObservation` describes one stream from one exact probe operation and source revision. It records stream kind, external stream ordinal, codec/profile observations, language declarations, timing metadata, disposition, attachment relations, and probe provenance. Stream observations MUST NOT claim support, safety, or delivery readiness by detection alone.

## Timing and Coordinate References

Durable time values MUST use integers and explicit rational time bases. A `TimePointRef` records an integer value, rational time-base identity, source timeline identity, and mapping revision. A `TimeRangeRef` records ordered start and end points using compatible timeline semantics.

Exact CFR, VFR presentation-timestamp, frame, sample, proxy/source-map, drop-frame-display, and spatial-coordinate behavior belongs to `VID-SPEC-003`. Until that specification is Frozen, this schema MUST NOT define floating-point or nominal-FPS shortcuts.

- A cue range MUST satisfy `start < end` under its timeline contract.
- Durable VFR positions MUST NOT be derived from `frame_index / nominal_fps`.
- A proxy reference MUST identify the exact source-to-proxy mapping revision.
- Spatial references MUST identify normalized coordinate space and exact transform chain.

## Track Record

A `Track` MUST contain:

- Track ID, track kind, schema version, and project reference.
- Language tag using BCP 47 where language applies.
- Source, target, bilingual, signs, captions, transcript, review, metadata, or other namespaced role.
- Timeline and default spatial/style context references.
- Ordered cue membership represented independently of Cue ID.
- Import provenance, format identity, and conversion-loss state where applicable.
- Lock, visibility, approval, and delivery eligibility states with distinct semantics.

Track order, display color, name, visibility, or workspace placement MUST NOT be identity. Moving a cue between compatible tracks MUST preserve Cue ID only when the product command records semantic continuity; otherwise replacement lineage is required.

## Cue Record

A `Cue` MUST contain:

- Cue ID, Track ID, exact Video Revision, and cue-schema version.
- Time range and optional spatial-region reference.
- Versioned text payload made of typed runs rather than trusted markup.
- Source/target relation and explicit lineage edges.
- Speaker, style, region, note, status, QC, lock, and annotation references.
- Creation/import/observation provenance and last authoritative command.
- Approval class distinct from completion or export selection.

Cue rules:

1. Text and timing authority MAY have separate source observations and MUST be reconciled explicitly.
2. Splitting, merging, replacing, retiming, moving, and rebinding MUST create explicit lineage and affected-reference outcomes.
3. Locked fields MUST NOT be changed by automation or bulk repair without an authorized override command.
4. Overlap MUST NOT imply conflict without compatible layer and policy context.
5. Conversion loss, unsupported styling, unknown pass-through data, and format precision MUST remain inspectable.
6. Import/export round trips MUST NOT silently erase unknown data that the selected profile promises to preserve.

## Word Record

A `Word` is an editorial unit, not an assumption that every language uses whitespace segmentation.

It MUST record Word ID, owning Cue ID, exact cue revision, text or text-span identity, source language, grapheme-aware boundaries, optional time range, confidence provenance when observed, and lineage.

- Thai, CJK, RTL, mixed-script, markup, and combining-mark behavior MUST use the selected language/profile rules.
- A provider token MUST NOT automatically become an authoritative Word.
- Cue edits MAY retain, split, merge, replace, or retire Word IDs only through deterministic adapter rules with explicit lineage.
- Character counts, reading-speed counts, segmentation, and shaped widths MUST name their exact rule/profile identity.

## Speaker Record

A `Speaker` MUST contain Speaker ID, project scope, display labels and aliases, optional external/source references, provenance, merge/split lineage, and review state.

- Speaker identity is a production label and MUST NOT claim biometric, legal, demographic, emotional, or mental-state identity.
- Diarization output is an observation until accepted by a Video command.
- Merging or splitting speakers MUST preserve prior references through explicit lineage.
- Display-name changes MUST NOT change Speaker ID.
- Sensitive notes MUST use declared access and retention scopes.

## Annotation Record

An `Annotation` MUST contain Annotation ID, namespaced annotation kind/version, exact target references, author or producing operation, authority class, payload schema, lifecycle state, provenance, and access/retention classification.

Targets MAY be project, asset, stream, track, cue, word, speaker, time range, spatial region, revision, observation, artifact, QC finding, or delivery result. Multi-target annotations MUST declare relationship semantics and MUST NOT imply equivalence merely by sharing one record.

Unknown required annotation payloads MUST block unsafe writes. Unknown optional durable annotations MUST be preserved according to the Frozen Shared extension policy. Annotation payloads MUST remain inert and MUST NOT trigger network access, executable activation, or project mutation.

## Observation and Proposal Record

An `Observation` is an immutable claim produced by import, probe, parser, worker, provider, plugin, rule, model, or user analysis. It MUST contain:

- Observation ID, kind/version, producer and implementation identity.
- Exact input artifacts, source revisions, target references, parameters, and policy context.
- Output payload identity, confidence or deterministic basis, limitations, and integrity.
- Created instant, expiry/staleness rules, and provenance.
- Authority class fixed as non-authoritative until an explicit acceptance command creates a revision.

Accepting an observation MUST record the selected fields, reviewer/actor authority, prior revision, command, resulting revision, and observation reference. Rejecting or superseding an observation MUST NOT erase its retained provenance when audit policy requires it.

## Video Revision and Lineage

A `VideoRevision` refines one Frozen Shared Revision and MUST include:

- Video Revision ID and exact Shared Revision ID.
- Parent Video Revision IDs.
- Product command and transaction identities.
- Actor, authority epoch, commit ordering, and schema revisions.
- Changed Video references and adapter-supplied impact summary.
- Lineage edges, audit, provenance, and required extensions.

Committed revisions are immutable. Undo, redo, restore, acceptance, and repair create new revisions. History MUST NOT be silently rewritten.

A `LineageEdge` MUST declare edge identity, exact source and target references, relation kind, producing command/revision, ordering when needed, and rationale/provenance. Required relation kinds are:

- `derived-from`
- `source-to-target`
- `split-into`
- `merged-from`
- `replaced-by`
- `rebound-to-source`
- `accepted-from-observation`
- `restored-from-revision`

Lineage MUST NOT be inferred from text similarity, time overlap, list adjacency, names, or content hashes alone. Many-to-many relations require explicit edges and compatibility validation.

## Artifact and Delivery References

Derived waveforms, proxies, thumbnails, indexes, subtitle renders, reports, and exports remain Shared artifact records with Video-owned typed payloads and references.

Every artifact reference MUST bind exact input hashes, relevant Video revisions, operation/version, implementation/provider identity, parameters, runtime/backend, output hash, and policy context. Rebuildable artifact deletion MUST NOT remove project truth. A delivery result MUST bind its exact project revision, selected tracks/cues, delivery profile, QC gate, conversion-loss report, and produced artifact.

## Commands and Mutation Invariants

Every authoritative command MUST carry command/version identity, actor and authorization context, expected base revision, affected object preconditions, idempotency identity/digest, typed payload, and correlation/causation identities.

- Validation, authorization, fencing, schema, integrity, lock, and object preconditions MUST pass before mutation.
- A command MUST commit product records, Shared revision/event records, lineage, audit, and required inverse metadata atomically.
- Duplicate commands with matching digest MUST return the original result; conflicting reuse MUST fail closed.
- Long-running media/provider work MUST occur outside the authoritative transaction.
- A partial command MUST NOT become visible.
- Failure after commit but before response MUST be recoverable through idempotent status lookup.

## Versioning, Migration, and Preservation

Every record family has an independent schema identity and compatibility range under `SUI-SPEC-001`.

- Unknown required fields or extensions MUST produce incompatibility before write.
- Unknown optional durable data MUST be preserved byte-for-byte when safe re-encoding is unavailable.
- Migration MUST declare source/target ranges, identity mappings, ordered restartable steps, validation, rollback/read-only behavior, and data-loss impact.
- A migration that changes timing, spatial, identity, lineage, or approval meaning MUST supply explicit fixtures and mapping evidence.
- Failed migration MUST leave either the previous project or the fully validated replacement readable.
- No schema migration may silently promote observations, clear locks, erase lineage, or change approval.

## Security and Privacy

All project, media metadata, imported text, annotation, extension, and observation payloads are untrusted until validated.

- Records MUST enforce size, count, depth, text, nesting, and expansion limits before allocation.
- Project records MUST NOT contain credentials, private keys, ambient tokens, or unrestricted provider requests.
- Structured locators MUST prevent traversal, device-path access, link escape, and automatic external resolution.
- Content MUST NOT trigger network requests, plugins, scripts, decoders, fonts, or executables merely by being opened or preserved.
- Logs and provenance MUST be bounded, redacted, and access-controlled.
- Speaker and language records MUST NOT infer protected, biometric, emotional, or personnel attributes beyond authorized production labels.
- Hostile media-specific parser/decoder limits and quarantine behavior are owned by `VID-SPEC-004`.

## Safe Failure and Degraded Modes

- Wrong-product, unsupported schema, missing required extension, corrupt identity, ambiguous lineage, or failed integrity MUST open read-only with structured diagnostics or fail safely.
- Missing optional media, proxies, caches, models, plugins, or providers MUST NOT make fundamental project truth unreadable.
- A missing external source MUST retain approved text and timing while marking dependent operations unavailable or stale.
- Corrupt rebuildable artifacts MAY be quarantined and regenerated; corrupt authoritative data MUST NOT be silently replaced.
- A failed relink, import, migration, or observation acceptance MUST leave the prior revision authoritative.

## Conformance Gates

| Gate | Requirement |
| --- | --- |
| `VID-DOM-001` | Every durable record has stable, non-path, non-recycled identity. |
| `VID-DOM-002` | Durable references bind exact project, object, schema, and revision identity. |
| `VID-DOM-003` | Shared storage and transport do not acquire Video semantic or mutation authority. |
| `VID-DOM-004` | Project truth remains independent of rebuildable artifacts and optional capabilities. |
| `VID-DOM-005` | Media relink/replacement verifies source authority and invalidates dependents explicitly. |
| `VID-DOM-006` | Cue/word/speaker/annotation split, merge, replacement, and source-target lineage are explicit. |
| `VID-DOM-007` | Observation success cannot become an approved revision without an authorized command. |
| `VID-DOM-008` | Commands, revisions, events, lineage, audit, and inverse metadata commit atomically. |
| `VID-DOM-009` | Time and spatial references defer exact semantics to `VID-SPEC-003` without nominal-FPS shortcuts. |
| `VID-DOM-010` | Unknown, incompatible, hostile, or missing data fails safely and preserves project truth. |
| `VID-DOM-011` | Migration preserves identity, extensions, lineage, locks, approval, and prior readable state. |
| `VID-DOM-012` | Video remains independent of Manga/Document and all later AI/Teams semantics remain outside Phase 00 authority. |

## Fixture Families

Specification-freeze evidence MUST include synthetic, redistributable cases for:

1. Identity, move, relink, fork, tombstone, duplicate bytes, and non-reuse.
2. Project, collection, asset, stream, track, cue, word, speaker, annotation, and exact-reference round trips.
3. Split, merge, replacement, source-target, observation-acceptance, undo, restore, and many-to-many lineage.
4. CFR/VFR/rational time references, proxy mappings, mixed coordinate spaces, and invalid ranges without defining `VID-SPEC-003` math.
5. Thai, CJK, RTL, mixed script, combining marks, markup, and language/profile-bound counting references.
6. Wrong product, stale revision, deleted object, unsupported schema, unknown extension, corrupt integrity, and ambiguous relink.
7. Worker/provider/plugin attempts to mutate project truth or promote observations directly.
8. Interrupted commands, duplicate requests, failed migration, missing source, damaged cache, and read-only recovery.

Native database, media, timing, migration, crash, and product-adapter conformance remains downstream `VID-SPEC-015` and Shared conformance evidence.

## Freeze Criteria

This specification may become Frozen when:

1. `SUI-SPEC-001@1.0` and `VID-SPEC-001@1.0` remain Frozen.
2. Every canonical record, identity, authority boundary, and conformance gate has a stable definition.
3. Synthetic fixture families cover positive, negative, stale, incompatible, interruption, migration, and recovery outcomes.
4. Video, Shared, QA/compatibility, and Security reviews approve the exact candidate hash.
5. No unresolved point can change durable identity, reference, lineage, approval, or source-authority meaning.
6. Evidence explicitly defers time/coordinate math, hostile ingest, native storage, and performance claims to their registered owners.

## Downstream Ownership

- `VID-SPEC-003` freezes timebase, VFR PTS, frame/sample, proxy/source-map, and spatial-coordinate semantics.
- `VID-SPEC-004` freezes hostile media-ingest and parser/decoder isolation policy.
- `VID-SPEC-005` freezes manual editing, deterministic timing, subtitle language, QC, and interaction behavior.
- `VID-SPEC-015` owns Phase 00 native schema, migration, recovery, corpus, fault, support, and budget evidence.
- `SUI-SPEC-012-D` owns complete Video-to-Shared adapter and lifecycle conformance at the Video adoption checkpoint.

No downstream specification may silently reinterpret a Frozen identity, lineage, approval, or source-authority rule.

## Planning References

- [Product Scope](../Plan/sections/02_PRODUCT_SCOPE.md)
- [Core Architecture](../Plan/sections/04_CORE_ARCHITECTURE.md)
- [AI Data Model and Providers](../Plan/sections/05_AI_DATA_MODEL_AND_PROVIDERS.md)
- [Release Gates](../Plan/sections/10_RELEASE_GATES.md)
- [Next Planning Documents](../Plan/sections/11_NEXT_PLANNING_DOCUMENTS.md)
- [Phase 00 Foundations](../Plan/roadmap/PHASE_00_FOUNDATIONS.md)
