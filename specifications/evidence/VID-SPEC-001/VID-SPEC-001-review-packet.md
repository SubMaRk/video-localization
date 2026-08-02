# VID-SPEC-001 Draft.1 Review Packet

## Exact Candidate

- Specification: `VID-SPEC-001@0.1-draft.1`
- Candidate SHA-256: `9DCE066CF95A0C13B1169F8BF9F92A4A93429B5F833F5F366CB0B37C55244CAC`
- Dependency: `SUI-DEC-008@1.0` Accepted, SHA-256 `A655CD115F68A286ED5A854ABCB7AF198E585BCCD0CA224AA23A1681B04D48A5`
- Machine result: `VID-EVID-001@0.1.0-draft.1` Pass, SHA-256 `E678F97244E88C60E12CF08C1D4D97845F90360E9659661B00521D2ABDC10670`
- Verifier: `verify.ps1`, SHA-256 `9819F04A7D40E08C67D96CA22D2E9E245A5BC36E105E4B671DA469EF63BA2CFD`
- GitHub work item: `SubMaRk/video-localization#19`, open in `Planning and Specification Freeze`

The machine result proves 13 structural and traceability checks, including 20 Phase 00 requirements, 15 Phase 01 requirements, seven PRD gates, registered downstream references, no later-phase specification leakage, valid local links, and the accepted dependency status. It does not approve or freeze the PRD.

## Manual Herdr Review Routing

The repository owner assigned the formal roles in [`OWNER-AUTH-V0`](../../../../.agents/authority/OWNER-AUTH-V0.md). A review counts only when the assigned agent records an explicit decision, exact candidate hash, evidence, role, authority reference, and timestamp. Reviewers MUST NOT edit the candidate while reviewing it.

| Agent | Review focus | Required output |
| --- | --- | --- |
| `terra` | Video domain completeness, timebase/VFR, playback, subtitle, QC, delivery, Phase 00/01 boundary | Findings by severity; affected requirement IDs; approve/reject recommendation |
| `luna` | Shared dependency direction, adapter ownership, compatibility, lifecycle, SUI-SPEC mapping | Findings by severity; dependency corrections; approve/reject recommendation |
| `gemini-pro` | Workspaces, System/Light/Dark, accessibility, English/Thai, Windows operator workflow | Missing user outcomes or acceptance coverage; affected requirement IDs |
| `sonnet` | Traceability, governance, release allocation, security/recovery ownership, approval sufficiency | Blocking/non-blocking findings; status recommendation; no candidate edits |
| `deepseek` | Independent architecture and test-gap review, VFR, data integrity, hostile ingest, recovery | Independent findings and exact remediation; no candidate edits |
| `haiku` | Release allocation and GitHub issue #19 traceability | Record Release Engineering decision; mirror accepted evidence without uploading local `Plan/` files |

## Review Gates

| Gate | Current state | Remaining evidence |
| --- | --- | --- |
| `VID-PRD-GATE-001` | Machine Pass; owner review pending | Product Lead confirms every mapping is semantically correct, not merely registered |
| `VID-PRD-GATE-002` | Machine Pass for later-spec leakage; release review pending | Release Engineering confirms all Phase 00/01 allocations |
| `VID-PRD-GATE-003` | Pending | Domain Architect: Video and Shared Platform Lead approve ownership boundaries |
| `VID-PRD-GATE-004` | Pending | Security Lead approves downstream threat-model and failure-evidence ownership |
| `VID-PRD-GATE-005` | Pending | UX, Internationalization, and Accessibility leads approve requirement completeness |
| `VID-PRD-GATE-006` | Machine structure Pass; QA review pending | QA and Compatibility Lead confirms measurable downstream evidence ownership |
| `VID-PRD-GATE-007` | Pending | Every required approval binds the exact candidate hash and no blocker remains |

## Approval Boundary

An approval is valid only when it records the assigned approver, `OWNER-AUTH-V0` authority reference, assigned approval role, RFC 3339 timestamp, review evidence, decision, and the exact candidate SHA-256 above. Agent identity, model name, dependency acceptance, machine validation, or an unassigned recommendation does not independently grant approval authority.

Any candidate edit invalidates the hash and requires a new revision, machine result, review packet, and approval binding. The register MUST remain `Drafting` until reviewers are actually evaluating a stable candidate, and MUST remain short of `Frozen` until the atomic freeze transaction is authorized.

## Next Transaction

1. Record formal findings and decisions from the assigned Herdr agents.
2. Resolve every blocking finding in a new candidate revision, or record an authorized bounded exception.
3. Change the candidate to `In Review` only when the assigned reviewers begin evaluating the same hash.
4. Freeze only after all approvals are recorded and the candidate hash remains unchanged.

## Freeze Outcome

All nine decisions were Approved at `2026-08-02T00:18:00+07:00` under the Owner-authorized consolidated-review override and bind candidate SHA-256 `9DCE066CF95A0C13B1169F8BF9F92A4A93429B5F833F5F366CB0B37C55244CAC`. Machine evidence remained Pass 13/13. The review is not represented as independent. Local freeze completed; GitHub issue #19 remains observed OPEN and awaits an authorized `github-agent` sync.
