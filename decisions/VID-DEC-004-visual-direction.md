# VID-DEC-004 - Video Visual Direction

| Field | Value |
| --- | --- |
| Decision ID | `VID-DEC-004` |
| Status | Accepted |
| Revision | `1.0` |
| Decision owner | Product Lead: Video (`primary`) |
| Required reviewers | UX Design Lead; Accessibility Reviewer; Thai Language Quality Reviewer; QA and Compatibility Lead |
| Authority | `OWNER-AUTH-V0`; Owner Resolution Bundle V1; owner-authorized consolidated self-review |
| Blocker | `P0:V0` |
| Depends on | `SUI-DEC-007@1.0` Accepted; `SUI-SPEC-007@1.0` Frozen |
| Informs | `VID-SPEC-014`; `VID-SPEC-015`; Phase 00 shell and workspace implementation |

## Decision Question

Which visual direction should make Video Localization feel like a premium creative workstation while preserving precision, accessibility, international text support, media performance, and equal first-class Light and Dark themes?

## Direction Studies

| Study | Character | Disposition |
| --- | --- | --- |
| Cinematic Ledger | Warm graphite, paper-white working surfaces, restrained amber action cues, technical rulers, compact editorial geometry | Selected |
| Broadcast Blueprint | Cool blue control-room surfaces and broadcast-monitor language | Rejected because it reads as generic operations software and makes collaboration/status blue harder to distinguish |
| Studio Glass | Translucent layers, luminous gradients, floating controls, and soft blur | Rejected because blur, decoration, contrast variability, and composition cost conflict with precision and media performance |

## Decision

Adopt **Cinematic Ledger** as the Video product visual direction.

### Visual Character

- Present the application as a calm editing instrument, not an enterprise dashboard, gaming interface, or decorative cinema skin.
- Use warm-neutral graphite structure, paper-white or ink-tinted working surfaces, restrained hairline boundaries, and compact editorial spacing.
- Use an amber action family for the playhead, primary action, selected timing affordances, and active edit focus. Amber MUST NOT carry warning or error meaning.
- Keep success, warning, error, collaboration presence, review state, and track identity on distinct semantic families. State MUST remain understandable without color.
- Use restrained corner radii and elevation. Continuous blur, heavy glass, neon glow, decorative gradients, and ornamental animation are prohibited.
- The media stage remains a calibrated neutral-dark surface in both Light and Dark themes so image judgment does not change with application chrome.

### Theme and Token Direction

- System is the default theme and follows live operating-system changes. Light and Dark are equal first-class modes.
- Theme changes MUST preserve focus, selection, playback, timeline position, panel layout, open dialogs, and unsaved edit state.
- Implement the Shared primitive-to-semantic-to-component token hierarchy. Product tokens MAY specialize timeline, waveform, subtitle, review, collaboration, media-stage, and delivery semantics but MUST NOT bypass Shared accessibility contracts.
- Exact production color values, contrast measurements, high-contrast mappings, component tokens, and rendered evidence belong to `VID-SPEC-014` and `SUI-SPEC-012@Exit:S1`; this decision fixes direction and semantic separation, not unmeasured token claims.

### Typography and International Text

- Use a licensed, locally packaged humanist sans family with broad script coverage for UI text. Do not freeze a font payload until source, license, shaping, and redistribution evidence is approved.
- Use tabular monospaced numerals for timecode, frame, sample, duration, and measurement readouts without forcing monospaced text on prose.
- Preview surfaces render the actual subtitle shaping and fallback chain rather than substituting UI typography.
- Thai, CJK, and RTL strings MUST survive compact controls without clipping, reordered meaning, broken combining marks, or hidden focus indicators.

### Layout and Density

- Support Compact and Comfortable density through tokens; do not maintain divergent component implementations.
- Build the five workspaces as purposeful arrangements: Edit, Timing, Translation, Review, and Delivery.
- Prefer stable pane geometry, explicit hierarchy, technical rulers, and aligned numeric columns. Avoid card grids where a timeline, inspector, table, or structured list communicates the work more directly.
- The timeline and transport are the primary precision surfaces. Review and collaboration information remains visible but must not compete with playback, cue text, or timing controls.

### Motion and Performance

- Normal chrome transitions target 120-180 ms with restrained easing.
- Playback, scrubbing, dragging, trimming, waveform navigation, keyboard timing, and cue stepping MUST never wait for decorative motion.
- Reduced-motion mode removes nonessential transitions without hiding state changes.
- Visual effects MUST degrade before timing interaction or media presentation falls below the baseline performance contract.

### Accessibility

- Meet the Shared contrast, focus, keyboard, target-size, high-contrast, reduced-motion, and assistive-technology contracts.
- Pair color with shape, iconography, text, pattern, or position for every status and timeline distinction.
- Focus treatment must remain visible against both application chrome and the neutral-dark media stage.
- Component states cover default, hover, pressed, selected, focused, disabled, busy, empty, warning, error, success, and conflict where semantically applicable.

## Consequences

Positive consequences:

- The product gains a recognizable creative-workstation identity without compromising technical density.
- Light and Dark modes share one semantic system while the media stage remains perceptually stable.
- Amber action cues distinguish Video from generic blue enterprise tooling without conflating errors or warnings.
- The direction scales from the Phase 00 shell to timing, translation, review, and delivery workspaces.

Costs and constraints:

- Amber contrast and status separation require measured token work rather than direct palette use.
- Broad-script typography requires licensed payload and shaping evidence before a concrete family is frozen.
- Compact density increases the need for Thai/CJK/RTL clipping tests and keyboard/focus evidence.
- Restrained effects place more responsibility on spacing, typography, borders, and state clarity.

## Acceptance Record

| Role | Status | Basis |
| --- | --- | --- |
| Product Lead: Video | Accepted | Selected Cinematic Ledger and bounded implementation ownership |
| UX Design Lead | Accepted by owner-authorized consolidated review | Direction, workspace hierarchy, density, and component-state review |
| Accessibility Reviewer | Accepted by owner-authorized consolidated review | Contrast ownership, non-color state, focus, motion, and high-contrast review |
| Thai Language Quality Reviewer | Accepted by owner-authorized consolidated review | Thai/CJK/RTL typography and clipping obligations |
| QA and Compatibility Lead | Accepted by owner-authorized consolidated review | Theme-state preservation and evidence boundaries |

This acceptance is a consolidated self-review authorized by the owner. It is not independent review and does not substitute for rendered conformance evidence.

## References

- [Video UI/UX and Design System](../Plan/sections/03A_UI_UX_AND_DESIGN_SYSTEM.md)
- [Shared UI Responsibility Decision](../../_shared/decisions/SUI-DEC-007-ui-responsibility-boundary.md)
- [Shared UI Specification](../../_shared/specifications/SUI-SPEC-007-ui-i18n-accessibility.md)
- [Phase 00](../Plan/roadmap/PHASE_00_FOUNDATIONS.md)
