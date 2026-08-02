# VID-SPEC-014 - Video Design and Localization Contract

| Field | Value |
| --- | --- |
| Specification ID | `VID-SPEC-014` |
| Status | Frozen |
| Revision | `1.0` |
| Owner role | UX and Design Lead |
| Required reviewers | Product Lead: Video; Accessibility Lead; Internationalization Lead; Thai Language Quality Reviewer; QA and Compatibility Lead |
| Blocker | `P0:V0`; `Phase:V1` |
| Depends on | `SUI-SPEC-007@1.0` Frozen; `VID-DEC-004@1.0` Accepted |
| Authority | `OWNER-AUTH-V2` |

Only a Frozen revision satisfies `P0:V0`. This contract freezes design tokens, component/workspace obligations, localization catalogs, terminology, and test matrices. It does not claim that native controls, screenshots, fonts, IME behavior, assistive technology, performance, or production UI have passed implementation conformance.

## Normative Language

`MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, and `MAY` express requirement strength. The referenced machine-readable token, component, catalog, glossary, and fixture assets are normative parts of this revision.

## Frozen Assets

| Asset | Authority |
| --- | --- |
| [Video tokens](../design/tokens/vid.tokens.json) | Primitive, semantic, and component token values and mappings |
| [Component matrix](../design/component-matrix.json) | Workspace allocation, component states, density, keyboard, accessibility, and performance obligations |
| [English source catalog](../localization/en.json) | Stable source messages and translator context |
| [Thai catalog](../localization/th.json) | First official translated catalog and English fallback binding |
| [English/Thai terminology](../localization/terminology.en-th.json) | Approved recurring application terms and context |
| [Design/localization cases](../testing/conformance/vid-spec-014/fixtures/design-localization-cases.json) | Synthetic matrix for theme, locale, placeholders, Thai, bidi, accessibility, IME, and layout |

## Visual Direction

The product implements **Cinematic Ledger** from `VID-DEC-004@1.0`: warm-neutral graphite structure, paper-white Light surfaces, deep charcoal Dark surfaces, restrained amber action cues, technical rulers, compact editorial geometry, and a neutral-dark media stage in every theme.

- Amber identifies primary action, playhead, active edit focus, and selected timing affordances. It MUST NOT carry warning or error meaning.
- Success, warning, error, information, collaboration presence, review state, and track identity use separate semantic roles and a non-color signal.
- Heavy glass, continuous blur, neon glow, decorative gradients, ornamental animation, and card-grid substitution for technical tables/timelines are prohibited.
- System is the default theme and follows live OS changes. Light and Dark are equal first-class themes.
- Theme, locale, density, and UI scale are independent preferences.

## Token Architecture

- Resolve tokens in the order Primitive -> Semantic -> Component. Production components MUST NOT read primitive colors directly.
- Product semantic tokens MAY specialize timeline, waveform, subtitle, review, collaboration, media-stage, and delivery roles without redefining Shared accessibility semantics.
- A component token references semantic roles; hard-coded production colors, spacing, radius, typography, or motion durations are prohibited.
- The media-stage token resolves to the same neutral-dark value in Light and Dark. Subtitle preview colors come from subtitle style/project data, not application chrome.
- Compact and Comfortable density share component implementations and differ only through tokens.
- Normal chrome motion is 120-180 ms. Playback, scrubbing, dragging, trimming, cue stepping, waveform navigation, and keyboard timing MUST NOT wait for animation.
- Reduced motion removes nonessential transitions while preserving state communication.

### Required Contrast

- Normal text and essential icon glyphs require at least 4.5:1 against their background.
- Large text, focus indicators, control boundaries, selected outlines, timeline marks, and meaningful non-text graphics require at least 3:1 against adjacent colors.
- Disabled state MAY use lower contrast only when it remains distinguishable and is not the only way to discover required information.
- High Contrast mode uses system colors and preserves semantic labels; it does not attempt to retain brand colors.

## Typography

- `font.ui` is a logical broad-script humanist sans role; `font.timecode` is a logical tabular monospaced numeric role. A concrete payload requires separate source, license, shaping, and redistribution evidence.
- UI line metrics MUST preserve Thai combining marks above and below the nominal Latin box.
- Timecode, frames, samples, durations, and measurements use tabular numerals without forcing prose into monospaced typography.
- Subtitle preview uses the actual selected subtitle font, fallback, shaping, style, and layout semantics rather than UI typography.
- Thai, CJK, RTL, mixed script, emoji, combining sequences, and technical identifiers remain readable and do not corrupt stable IDs.

## Workspace Contract

| Workspace | Primary purpose | Required Phase 00 surface | Phase 01 completion |
| --- | --- | --- | --- |
| Edit | Source/target cue authoring | Video stage, cue list, inspector, transport | Full editing and source/target track tools |
| Timing | Frame-accurate cue timing | Timeline, playhead, waveform placeholder/tiles, transport | Scrub, frame step, loop, trim, snap, timing findings |
| Translation | Source/target language work | Stable shell route and empty/loading/error states | Translation grid, terminology, context, revision states |
| Review | Findings and approval | Stable shell route and finding states | QC/review queues, comparison, comments, approvals |
| Delivery | Validation and export | SRT export route, progress, warning, error, success | Profiles, conversion loss, reports, promoted outputs |

The Phase 00 shell MUST expose all five routes so navigation and state contracts do not fork later, but only the allocated surfaces may claim workflow completion.

## Component Contract

- Every applicable component implements default, hover, pressed, selected, focused, disabled, busy, empty, warning, error, success, and conflict states from the component matrix.
- Keyboard order follows task order and remains stable across translated string length. All primary commands are reachable without a pointer.
- Focus remains visible against both chrome and media stage. Modal focus is contained and returns to the invoking control.
- Cue list, timeline, waveform, findings, jobs, and long selectors use bounded/virtualized rendering; one native element per project object is prohibited at scale.
- Icon-only controls require localized accessible names and visible tooltips where hover exists. Icons cannot be the only indicator of destructive or safety-critical action.
- Destructive, irreversible, conversion-loss, rights-review, and security-warning actions use explicit text and confirmation proportional to consequence.

## Application Localization

### Language Identity and Fallback

- Initial application choices are System Default, English (`en`), and Thai (`th`). System Default resolves through OS language preferences to a supported catalog, then English.
- Durable language identity uses canonical BCP 47 tags independent from localized display names.
- Thai fallback is `th-TH -> th -> en`; unsupported or damaged application catalogs fall back to English.
- Application language is independent from project source language, project target languages, provider language support, subtitle language, and locale-neutral protocol data.
- Switching language MUST preserve open project, workspace, layout, focus target where valid, selection, playhead, undo history, unsaved state, and job identities.

### Catalog Rules

- All production user-facing text, accessibility labels, announcements, errors, recovery instructions, warnings, and notifications use stable message keys.
- English is the canonical source and fallback catalog. Thai is the first official translated catalog.
- Catalog entries carry translator context. Source meaning changes invalidate the corresponding translation review.
- Named placeholders MUST match exactly across catalogs. Sentence fragments, positional placeholder assumptions, and concatenated translated clauses are prohibited.
- Technical IDs, paths, model IDs, codec names, shortcut chords, and timecodes remain typed values outside localized prose.
- Missing, extra, obsolete, duplicate, empty, malformed, or placeholder-incompatible entries fail catalog validation.
- Plugins and providers use namespaced catalogs and fall back to English; untrusted localized markup cannot enter privileged UI.

### Locale Behavior

- Durable timestamps, decimals, hashes, manifests, logs, protocols, and timebase values remain locale-independent.
- Dates, ordinary numbers, percentages, and file sizes use selected locale formatting where technical clarity is preserved.
- Media timecode and frame display follow project media rules, not calendar locale. Users MAY select Latin digits for technical fields in Thai UI.
- User-facing sorting MAY use locale collation; stable IDs and technical fields use deterministic ordinal rules.

## Thai and Complex-Script Contract

- Thai layout uses script-aware line breaking and MUST NOT assume whitespace word boundaries.
- UI controls preserve combining marks, line height, cursor movement, selection, deletion, search, and accessible names.
- Keystrokes consumed by an active IME composition MUST NOT invoke editor commands.
- Truncation is allowed only when the complete localized value is available through tooltip, inspector, or accessibility text.
- Subtitle-language counting, segmentation, shaped width, wrapping, and preview/delivery parity remain product Language Core behavior; UI translation rules cannot replace them.
- Thai subtitle character metrics exclude non-spacing marks and non-visible markup only according to the approved Language Core policy; no hard-coded Thai character list is authoritative.

## Accessibility

- Meet Shared keyboard, focus, contrast, target-size, semantics, high-contrast, reduced-motion, zoom/scale, and assistive-technology contracts.
- Pair every color state with text, icon, shape, pattern, position, or accessible state.
- Localize accessible names, descriptions, errors, progress, loading, and live announcements while preserving stable automation identity.
- At 200% UI scale, critical Phase 00 flows remain operable without two-dimensional page scrolling; technical timeline horizontal scrolling remains task content, not page failure.
- Text reflow and translation expansion cannot hide safety actions, validation details, or current state.

## Test and Evidence Matrix

Planning conformance validates assets and synthetic cases. Implementation evidence additionally covers:

1. System/Light/Dark x English/Thai x Compact/Comfortable at supported scale factors.
2. High Contrast and reduced motion.
3. Keyboard-only traversal, focus visibility/return, screen-reader names/roles/states, and live announcements.
4. Thai combining marks, native IME composition, cursor/selection/deletion, mixed Thai/Latin/digits/CJK, RTL isolation, and pseudo-localization.
5. Theme and language switching with project, workspace, focus, selection, playhead, undo, unsaved state, and jobs preserved.
6. Component states and five workspace routes, including empty/loading/error/offline/limited/safe-mode conditions.
7. Baseline UI frame pacing and virtualization under the exact `VID-SPEC-015` environment.
8. Preview-versus-delivery shaping only when the relevant font/render/delivery profile is promoted.

## Conformance Gates

| Gate ID | Freeze requirement |
| --- | --- |
| VID-DESIGN-001 | Three-layer tokens and exact theme mappings are machine-readable |
| VID-DESIGN-002 | Required contrast pairs pass 4.5:1 or 3:1 thresholds |
| VID-DESIGN-003 | Cinematic Ledger semantic separation and prohibited effects are explicit |
| VID-DESIGN-004 | Five workspaces and Phase 00/Phase 01 allocation are exact |
| VID-DESIGN-005 | Component states, keyboard, focus, density, and virtualization obligations are exact |
| VID-DESIGN-006 | English and Thai catalogs have identical keys and placeholders |
| VID-DESIGN-007 | Terminology includes stable English, approved Thai, and context |
| VID-DESIGN-008 | BCP 47, System Default, fallback, and language-separation rules are exact |
| VID-DESIGN-009 | Thai, CJK, RTL, mixed-script, and IME obligations are exact |
| VID-DESIGN-010 | Accessibility, High Contrast, reduced motion, and 200% scale obligations are exact |
| VID-DESIGN-011 | Theme/locale/density/scale state-preservation matrix is exact |
| VID-DESIGN-012 | Freeze evidence is separated from rendered/native/performance evidence |

## Acceptance Record

| Role | Status | Required evidence |
| --- | --- | --- |
| UX and Design Lead | Accepted | Direction, token, workspace, and component completeness |
| Product Lead: Video | Accepted | Phase allocation and workflow state preservation |
| Accessibility Lead | Accepted by owner-authorized consolidated review | Contrast, keyboard, focus, semantics, scale, motion, and High Contrast |
| Internationalization Lead | Accepted by owner-authorized consolidated review | Catalog, BCP 47, fallback, placeholders, locale separation, and complex scripts |
| Thai Language Quality Reviewer | Accepted by owner-authorized consolidated review | Thai terminology, combining marks, layout, fallback, and IME obligations |
| QA and Compatibility Lead | Accepted by owner-authorized consolidated review | Machine-readable assets, matrix coverage, and evidence boundaries |

This acceptance is an owner-authorized consolidated self-review under `OWNER-AUTH-V2`; it is not independent review. Rendered/native/performance evidence remains downstream.

## References

- [Shared UI/i18n/accessibility contract](../../_shared/specifications/SUI-SPEC-007-ui-i18n-accessibility.md)
- [Video visual direction](../decisions/VID-DEC-004-visual-direction.md)
- [UI/UX and Design System](../Plan/sections/03A_UI_UX_AND_DESIGN_SYSTEM.md)
- [Internationalization and Localization](../Plan/sections/03B_INTERNATIONALIZATION_AND_LOCALIZATION.md)
- [Foundation PRD](VID-SPEC-001-foundation-prd.md)
- [Phase 00 Foundations](../Plan/roadmap/PHASE_00_FOUNDATIONS.md)
