# VID-SPEC-003: Timebase, Presentation Timestamp, and Coordinate Contract

| Field | Value |
| --- | --- |
| Specification ID | `VID-SPEC-003` |
| Status | Frozen |
| Revision | `1.0` |
| Owner | Domain Architect: Video |
| Required reviewers | Product Lead: Video; QA and Compatibility Lead; Performance and Hardware Lead; Security Lead |
| Blocker | `P0:V0` |
| Depends on | `VID-SPEC-002@1.0` |

## Purpose

This specification freezes exact Video-owned semantics for rational time bases, integer time points and ranges, CFR and VFR presentation timestamps, frame and audio-sample references, drop-frame display, source/proxy mappings, and normalized spatial coordinates and transforms.

It refines the canonical references in [VID-SPEC-002](VID-SPEC-002-domain-schema.md) and the Phase 00 requirements in [Core Architecture](../Plan/sections/04_CORE_ARCHITECTURE.md) and [Phase 00 Foundations](../Plan/roadmap/PHASE_00_FOUNDATIONS.md). It does not select a media library, decoder, player, waveform engine, storage encoding, or rendering API.

## Normative Language

`MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, and `MAY` express requirement strength.

## Core Invariants

1. Durable time is an integer value interpreted by an explicit normalized rational time base.
2. Durable VFR position is resolved from presentation timestamps, never `frame_index / nominal_fps`.
3. Conversion uses checked exact rational arithmetic before an explicitly named rounding boundary.
4. A frame number is derived within one declared timeline and MUST NOT serve as cross-timeline identity.
5. Source, proxy, audio, subtitle, and delivery timelines remain distinct and are connected only by versioned mappings.
6. Drop-frame notation changes display labels, not media time or frame count.
7. Durable spatial coordinates use a declared normalized coordinate space and exact transform revision.
8. Every seek, snap, map, quantize, and display operation names its policy and source revision.

## Rational Time Base

A `RationalTimeBase` contains:

- Stable time-base identity and schema version.
- Positive integer numerator `n` and denominator `d` in lowest terms.
- Timeline identity and semantic kind.
- Optional source provenance and external format representation.

One tick represents `n / d` seconds. A `TimePoint` is a signed integer tick value plus the exact time-base and timeline identities. Implementations MUST support the declared project duration without overflow and MUST reject arithmetic that cannot be represented safely.

Normalization rules:

- `n > 0`, `d > 0`, and `gcd(n,d) = 1`.
- Zero or negative components are invalid.
- Equivalent unreduced fractions MUST normalize before identity comparison but their imported form MAY remain provenance.
- A floating-point frame rate or timestamp MUST NOT become durable authority without conversion to a declared rational representation and recorded rounding evidence.

## Exact Conversion and Rounding

For source ticks `a` in time base `nA/dA`, conversion to target time base `nB/dB` first computes the exact rational target value:

~~~text
a * nA * dB
---------------
    dA * nB
~~~

Intermediate multiplication MUST use arbitrary precision or checked widening sufficient to prevent overflow. Reduction SHOULD occur before multiplication where possible.

If the exact result is not integral, the caller MUST select a versioned rounding policy. Required policies are:

- `floor`
- `ceiling`
- `nearest-ties-to-even`
- `nearest-ties-earlier`
- `range-outward` (`start=floor`, `end=ceiling`)
- `format-defined` with exact format/profile identity

No global implicit rounding policy is permitted. Conversion records used for durable timing MUST retain source value/base, target base, exact pre-round fraction or equivalent remainder, policy, result, and operation version.

Repeated conversions MUST use the original authoritative value or one exact composed mapping. They MUST NOT accumulate rounded deltas through iterative float or frame conversions.

## Time Ranges and Boundaries

Durable ranges are half-open `[start,end)` unless a versioned external format adapter explicitly records different source semantics.

- `start < end` is required for a nonempty cue or media interval.
- Touching ranges where `A.end = B.start` do not overlap.
- Open-ended observations MUST use an explicit unknown/open boundary state rather than a sentinel tick.
- Inclusive external endpoints MUST convert through a named adapter policy and preserve source precision/conversion-loss evidence.
- Range intersection, containment, gap, and ordering MUST occur in one compatible timeline or through an exact mapping.

## Timeline Identity

A timeline record contains timeline identity, source asset/stream revision, time base, origin, duration or open-duration state, discontinuity inventory, and mapping/version references.

Required timeline kinds include source presentation, source decode, audio sample, proxy presentation, subtitle working, and delivery. Matching numeric ticks across different timeline identities MUST NOT imply the same instant.

An origin shift, edit list, stream replacement, discontinuity repair, proxy regeneration, speed change, trim, or delivery conversion creates a new timeline or mapping revision when it changes interpretation.

## CFR Frame Contract

A CFR timeline declares exact frame rate `p/q` frames per second, presentation time base, first presentation instant, frame count or open count, and frame-boundary policy.

- Frame index is zero-based within the exact timeline revision.
- Frame `i` begins at the exact presentation instant derived from the timeline origin and `i * q/p` seconds.
- A frame interval ends at the next frame start; the final interval uses declared media end when known.
- Seeking by frame index MUST validate range and timeline revision.
- Snapping MUST return the exact frame presentation timestamp and record the snap policy.
- Decimal labels such as `29.97` or `23.98` MUST NOT replace exact rates such as `30000/1001` or `24000/1001`.

## VFR Presentation Timestamp Index

Each promoted VFR source stream MUST have a versioned `PresentationTimestampIndex` built from validated packet/frame presentation evidence.

An entry contains:

- Stable frame-observation identity.
- Presentation sequence index.
- Exact PTS and presentation time base.
- Duration when known or derivable under the profile.
- Keyframe/random-access classification where observed.
- Source packet/frame provenance and integrity scope.
- Discontinuity, duplicate, corruption, or uncertainty flags.

After profile-defined normalization, valid presentation entries MUST be strictly ordered by presentation instant. Decode timestamp and decode order MAY differ and MUST remain separate fields.

VFR detection MUST compare declared rates/durations with sampled or complete packet timing according to the promoted profile. A single container frame-rate field is insufficient authority.

For a target presentation instant:

- `containing-frame` selects the frame interval containing the instant.
- `at-or-before` selects the latest PTS not later than the target.
- `at-or-after` selects the earliest PTS not earlier than the target.
- `nearest` compares exact rational distances; the default tie policy is earlier unless the operation profile states otherwise.

Out-of-range, discontinuous, duplicate, corrupt, or uncertain regions MUST return structured outcomes and MUST NOT fall back silently to nominal FPS arithmetic.

## Frame Reference

A durable `FrameRef` contains source asset and stream revision, timeline revision, PTS-index revision, presentation sequence index, exact PTS, and optional frame integrity/provenance.

Frame index alone is a UI/query convenience. Any committed annotation, cue snap, thumbnail, OCR observation, QC finding, or artifact MUST bind the full FrameRef or an exact TimePoint plus mapping revision.

## Audio Sample Clock

An audio timeline declares exact sample rate, channel-layout identity, first sample origin, sample count, discontinuities, and source stream revision.

- Sample position is an integer in the declared sample clock.
- Mapping between audio samples and presentation time uses exact rational conversion.
- Playback synchronization MUST record which clock is master and the correction policy.
- Resampling creates a new timeline/mapping revision and MUST record source/target rates, implementation identity, delay/trim, and rounding.
- Waveform and analysis tiles MUST bind exact sample ranges and source revision.

## Drop-Frame and Non-Drop-Frame Display

Timecode is a presentation and interchange label over an exact timeline, not the authority for media time.

- A timecode profile MUST declare nominal count rate, exact media rate, drop/non-drop rule, start label, 24-hour wrap policy, and negative-time policy.
- Drop-frame numbering MUST skip labels according to the exact profile but MUST NOT skip media frames or alter PTS.
- Parsing MUST reject impossible skipped labels and ambiguous profile-free strings.
- Formatting followed by parsing under the same profile MUST resolve to the same frame reference within the profile's declared wrap scope.
- Semicolon or punctuation alone MUST NOT establish a drop-frame profile.

## Source-to-Proxy Mapping

A `TimelineMap` binds exact source and target timeline revisions and contains ordered mapping segments or anchor pairs.

Each segment MUST declare source interval, target interval, monotonic direction, mapping function/version, precision, discontinuity behavior, and validation result.

- Mapping MUST be monotonic for ordinary proxies and MUST NOT invent a source instant.
- CFR proxy generation from VFR source MUST preserve an inspectable source-to-proxy relation for every promoted profile.
- Cue truth remains on the approved source timeline unless an explicit product decision says otherwise.
- Proxy seeking/snapping MUST map back to source PTS before committing durable cue timing.
- Regenerating a proxy creates a new proxy timeline and map revision; old references do not silently rebind.
- Round-trip mapping error MUST be measured in the source time base and bounded by the exact verification profile in `VID-SPEC-015`.

## Spatial Coordinate Space

A `CoordinateSpace` contains identity, schema version, source asset/frame or render surface revision, origin convention, axis direction, orientation, pixel dimensions/aspect where observed, normalized scale, crop/aperture, and transform-chain reference.

Durable normalized scalars use an integer numerator and positive declared scale. The Phase 00 canonical scale is profile-declared and MUST provide sufficient precision for the promoted corpus; binary floating-point values MUST NOT be durable authority.

The canonical untransformed space uses:

- Origin at the top-left of the declared presentation aperture.
- Positive X to the right and positive Y downward.
- Normalized bounds `[0,1]` represented by the declared integer scale.
- Half-open rectangles `[x0,x1) x [y0,y1)` with `x0 < x1` and `y0 < y1`.

Out-of-bounds regions MUST declare clipping, rejection, or intentional overscan policy. They MUST NOT be clamped silently.

## Spatial Transforms

A `SpatialTransform` records source and target coordinate-space revisions, transform kind/version, exact rational or fixed-point parameters, order, inverse availability, rounding policy, and provenance.

Required transform kinds include crop/aperture, scale, rotation by supported orientation, translation, pixel-aspect correction, proxy mapping, and composition.

- Transform order is significant and MUST be recorded.
- Repeated UI edits SHOULD compose against the authoritative source transform rather than accumulate rounded screen coordinates.
- A committed spatial annotation MUST bind its source coordinate space and exact transform chain.
- Mapping to pixels MUST use an explicit edge/center convention and rounding policy.
- Non-invertible, ambiguous, stale, or unsupported transforms MUST produce structured incompatibility.
- OCR, tracking, masks, overlays, previews, and delivery rendering MUST share this coordinate foundation when they claim alignment.

## Seeking, Snapping, and Editing

Every seeking or snapping request MUST declare target timeline, source value/reference, policy, tolerance, lock/authority context, and expected mapping revision.

- UI playhead movement MAY be approximate, but a committed edit MUST resolve to an exact source TimePoint or FrameRef.
- Locked cue boundaries remain fixed unless an authorized override command is used.
- Snapping to shot, word, waveform, cue, frame, or sample boundaries MUST identify the evidence revision and deterministic tie order.
- If constraints cannot all be satisfied, the operation MUST preserve approved data and return a finding rather than choose an undocumented boundary.

## Versioning and Migration

Time bases, timelines, PTS indexes, mappings, coordinate spaces, transforms, and policies are independently versioned.

- A mapping or index revision MUST NOT silently reinterpret existing references.
- Migration MUST preserve the original authoritative values and record exact old/new mappings and conversion remainder.
- A migration from floating-point or nominal-FPS legacy data MUST retain source values, uncertainty, selected rational interpretation, and review state.
- Failed migration MUST leave the prior project readable.
- Unknown required timing or coordinate extensions MUST block writes; optional durable extensions follow the Frozen Shared preservation policy.

## Security and Resource Bounds

- Timestamp/index counts, rational components, durations, transform chains, dimensions, and nesting MUST be bounded before allocation or multiplication.
- Arithmetic overflow, division by zero, invalid normalization, NaN/infinity import, and pathological transform expansion MUST fail closed.
- PTS index and mapping builders MUST run within hostile-ingest worker boundaries defined by `VID-SPEC-004`.
- Imported timecode, subtitle timing, edit lists, metadata, dimensions, and transforms are untrusted.
- Diagnostic output MUST avoid unrestricted media paths or payloads and include trace identity.

## Conformance Gates

| Gate | Requirement |
| --- | --- |
| `VID-TIME-001` | Rational time bases normalize and reject invalid or overflowing values. |
| `VID-TIME-002` | Exact conversion and named rounding produce deterministic results without cumulative drift. |
| `VID-TIME-003` | Half-open range behavior is consistent across boundaries and adapters. |
| `VID-TIME-004` | CFR frame references use exact rational rates and timeline revisions. |
| `VID-TIME-005` | VFR seeking/snapping uses validated PTS indexes and never nominal FPS fallback. |
| `VID-TIME-006` | Audio sample mapping is exact and records resampling/correction policy. |
| `VID-TIME-007` | Drop-frame/non-drop labels round-trip under an exact profile without changing media time. |
| `VID-TIME-008` | Source/proxy maps are versioned, monotonic, inspectable, and source-authoritative. |
| `VID-SPATIAL-001` | Normalized coordinate spaces have exact identity, bounds, orientation, and aperture. |
| `VID-SPATIAL-002` | Transform chains preserve order, precision, revision, and explicit rounding. |
| `VID-SPATIAL-003` | Stale, ambiguous, non-invertible, corrupt, or out-of-bounds inputs fail safely. |
| `VID-TIME-009` | Long-media, mixed-timebase, discontinuity, overflow, migration, and recovery fixtures preserve approved timing. |

## Fixture Families

Specification-freeze evidence MUST include synthetic, redistributable fixtures for:

1. Reduced/unreduced/invalid rationals, negative times, large products, and overflow.
2. Exact and fractional conversions under every required rounding policy.
3. CFR `24/1`, `25/1`, `30/1`, `24000/1001`, `30000/1001`, and `60000/1001` timelines.
4. VFR PTS sequences with irregular duration, long tails, discontinuities, duplicates, corruption, and metadata disagreement.
5. Drop-frame and non-drop-frame valid, skipped, ambiguous, negative, and wrap labels.
6. Audio sample clocks, resampling delay, drift correction, and waveform ranges.
7. VFR-source/CFR-proxy mappings, regenerated proxies, stale maps, and round-trip error.
8. Normalized regions, crop, scale, orientation, pixel aspect, transform order, clipping, and non-invertible transforms.

Native decoder, seek, playback, proxy generation, audio, render, long-media, and fault-injection evidence remains downstream `VID-SPEC-015` scope.

## Freeze Criteria

This specification may become Frozen when:

1. `VID-SPEC-002@1.0` remains Frozen.
2. Every time, frame, sample, display, mapping, coordinate, and transform identity and policy is exact.
3. Synthetic fixtures cover deterministic positive, negative, boundary, long-range, discontinuity, overflow, stale, and migration cases.
4. Video, QA/compatibility, performance/hardware, and Security reviews approve the exact candidate.
5. Evidence does not represent reference arithmetic as native media or playback conformance.
6. No unresolved issue can change durable timing, frame, mapping, or coordinate meaning.

## Downstream Ownership

- `VID-SPEC-004` owns hostile media parser/decoder isolation and limits.
- `VID-SPEC-005` owns timing-editor behavior, deterministic timing policies, locks, and QC.
- `VID-SPEC-006` owns format-specific precision, conversion loss, and delivery profiles.
- `VID-SPEC-015` owns native VFR/long-media/proxy/playback/recovery corpus and thresholds.

## Planning References

- [Core Architecture](../Plan/sections/04_CORE_ARCHITECTURE.md)
- [Release Gates](../Plan/sections/10_RELEASE_GATES.md)
- [Phase 00 Foundations](../Plan/roadmap/PHASE_00_FOUNDATIONS.md)
