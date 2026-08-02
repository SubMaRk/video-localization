# VID-DEC-002: Video Project and Application Identity

- **Status:** Accepted
- **Revision:** `1.0`
- **Decision owner:** Product Lead: Video
- **Allocation:** `P0:V0`
- **Depends on:** [SUI-DEC-008](../../_shared/decisions/SUI-DEC-008-planning-baseline-acceptance.md)
- **Related decision:** [VID-DEC-001](VID-DEC-001-implementation-stack.md)

## Decision

Video Localization Studio adopts the following durable identities.

| Surface | Production identity | Rule |
|---|---|---|
| Product display name | `Video Localization Studio` | Localizable display text; not a durable identifier. |
| Project file extension | `.vlsp` | Case-insensitive at the Windows shell boundary; emitted lowercase. |
| Project media type | `application/vnd.submark.video-localization-project` | Used where a MIME-style type is required. |
| Product namespace | `submark.video-localization` | Byte-exact canonical Shared namespace; never reassigned. |
| MSIX package `Name` candidate | `SubMaRk.VideoLocalizationStudio` | Production-only candidate; Store availability remains an external verification gate. |
| MSIX application `Id` | `VideoLocalizationStudio` | Must match packaged and external-location manifests where applicable. |
| Protocol scheme | `submark-vls` | Reserved for local activation; no network retrieval is implied. |
| File association name | `submark-vlsp-project` | Handles `.vlsp` only. |

The Package Family Name is not frozen as a literal. Windows derives it from package `Name` and signing publisher identity. Release evidence must record the derived production value after the production publisher certificate or Store identity is available.

## Project Identity Semantics

A project has an immutable UUID independent of its path, extension, package identity, display name, and current machine. Renaming or moving a `.vlsp` file does not change that UUID.

The `.vlsp` format is a versioned project container or manifest that stores localization state and references source media. It must not silently embed source media, credentials, licensed model payloads, or machine-specific absolute paths. Any optional embedded asset class requires an explicit format revision and policy review.

Readers identify the project from format markers and schema version, not from the extension alone. Unknown major versions fail closed without modifying the file. A failed open or migration leaves the source byte-for-byte unchanged.

## Namespace Binding

`submark.video-localization` replaces the temporary `fixture.product.video` namespace only after the corresponding Shared adapter binding is reviewed and versioned. Fixture identities remain test-only and must never appear in production projects, telemetry, package manifests, protocol registrations, or released conformance evidence.

The namespace is compared byte-for-byte. It must satisfy the canonical Shared namespace pattern and may not be reassigned. Child identifiers use stable dotted suffixes under this root and do not create ownership outside the Video product boundary.

## Channel and Modified-Build Isolation

Production, development, test, and modified builds must not share package identity, protocol registration, file-association ownership, update channel, signing identity, or writable data root.

| Channel | Package `Name` pattern | Namespace rule | Protocol rule |
|---|---|---|---|
| Production | `SubMaRk.VideoLocalizationStudio` | `submark.video-localization` | `submark-vls` |
| Development | `SubMaRk.VideoLocalizationStudio.Dev` | `dev.<operator>.video-localization` | `submark-vls-dev` |
| Test | `SubMaRk.VideoLocalizationStudio.Test` | `test.<run-id>.video-localization` | no persistent registration by default |
| Modified/fork | `<publisher>.<product>` | owner-controlled namespace distinct from `submark.*` | owner-controlled scheme distinct from `submark-vls*` |

Modified builds may retain attribution required by license but must not present themselves as the official product, claim the official namespace, overwrite official associations, consume the official update feed, or use the official signing identity. Importing an official `.vlsp` project into a modified build must not mutate it without explicit user action and a recoverable copy.

## Activation and Association Rules

Packaged builds declare file and protocol activation in the package manifest. Unpackaged development builds may register per-user activation only when explicitly enabled and must unregister cleanly. Installation must not seize unrelated defaults or register broad media extensions.

Protocol activation accepts only local, versioned commands with a strict allowlist and bounded payloads. It rejects credentials, arbitrary file execution, remote URLs, traversal, and unsupported commands. Opening through either activation path follows the same validation, recovery, and consent rules as opening from the UI.

## Packaging and Signing Gates

The production package candidate is not release-authoritative until all applicable evidence exists:

1. Package name availability or reservation is confirmed for the intended distribution channel.
2. The production publisher subject and signing custody are approved without committing private key material.
3. Package `Name`, publisher, and application `Id` match across relevant manifests.
4. The derived Package Family Name and application user model identity are captured from a signed package.
5. Side-by-side production, development, and test installation proves identity and data isolation.
6. File and protocol activation positive and negative cases pass on supported Windows versions.
7. Install, upgrade, rollback, and uninstall preserve user projects and remove owned registrations.

Self-signed development certificates are permitted only for local development evidence. They do not establish production publisher identity.

## Migration and Collision Handling

The first released project schema owns `.vlsp`. A future extension or namespace change requires a new decision, explicit import/export behavior, and compatibility evidence. Silent reassignment is prohibited.

If `.vlsp`, `submark-vls`, or the package candidate conflicts with an existing registered identity, implementation stops at the affected boundary. The product must not overwrite the conflicting owner. Resolution requires a revised exact candidate and approval before release artifacts are produced.

## Alternatives Rejected

- `.vls` is shorter but has a higher collision and ambiguity risk.
- `.vloc` describes localization but not a durable project container.
- `fixture.product.video` is reserved for conformance fixtures and is not a public identity.
- Freezing a fabricated Package Family Name is invalid because its publisher-derived component is not yet authoritative.
- Sharing production identity with development, test, or forks creates update, data, activation, and support ambiguity.

## Review and Acceptance Criteria

Acceptance required an exact-candidate review covering Video architecture, Shared namespace compatibility, security/privacy and licensing, Windows/package compatibility, release operations, and negative paths. The Owner-authorized consolidated review passed on 2026-08-01 for candidate SHA-256 `AA5455A1FA2260551D9D63CDE9BA82DB621F1DA9FA4BE887833F4DC0656F14F3`. It is not represented as an independent review.

Implementation may consume these identities only after this decision is Accepted and the package/signing gates applicable to the implementation stage are represented in executable or inspectable evidence.

## References

- [Microsoft: Package identity overview](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview)
- [Microsoft: Windows App SDK desktop activation](https://learn.microsoft.com/en-us/windows/apps/develop/launch/activate-an-app)
- [Microsoft: Handle file activation](https://learn.microsoft.com/en-us/windows/apps/develop/launch/handle-file-activation)
- [Microsoft: Package with external location](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/grant-identity-to-nonpackaged-apps)
