# VID-IMPL-P00-002B1 Record-Digest Profile

| Field | Value |
| --- | --- |
| Profile ID | VID-IMPL-P00-002B1-RECORD-DIGEST-PROFILE |
| Profile version | 1.0.0-p00 |
| Status | candidate_for_review |
| Authority | OWNER-AUTH-V2 |
| Hash algorithm | SHA-256 |
| Hash coverage | Complete record file bytes |
| Encoding | UTF-8 without BOM |
| Line endings | LF (0A) only |
| Indentation | Two ASCII spaces |
| Trailing whitespace | Forbidden |
| Final newline | Exactly one LF |
| Serialization scope | Record artifact files only |
| RFC canonical JSON | Not used |
| Cache-key CBOR | Not used |

## Purpose

This candidate binds the missing 002B1 resource-profile and control-protocol record digests without changing the seven-schema VID-IMPL-P00-002B1 bundle. It is a domain-neutral exact-byte JSON file profile. It is not RFC canonical JSON and it is not sui.cache-key.cbor-sha256.v1.

A digest is SHA-256 over the complete file bytes, including the final LF. Hexadecimal values in the manifest and checkpoint are lowercase or uppercase display forms of the same digest; JSON record content remains exact bytes.

## Frozen byte rules

Each JSON record MUST:

- Be UTF-8 without a BOM.
- Use LF line endings only.
- Use two ASCII spaces for indentation.
- Use the explicit key order declared below.
- Contain no trailing whitespace.
- End with exactly one LF.
- Preserve array order exactly as declared.
- Avoid implicit key sorting, Unicode normalization, number rewriting, or absent-value insertion.
- Be hashed as the complete file byte sequence.

This profile does not define a general canonical JSON algorithm. The existing schema-bundle rule is exact UTF-8 file hashing in lowercase hexadecimal. The existing sui.cache-key.cbor-sha256.v1 rule applies only to cache-key preimages and is not reused here.

## Resource record

Path: Video Localization/contracts/VID-IMPL-P00-002B1/records/resource-profile.json

Top-level key order:

1. resource_id
2. limits
3. transport
4. caps

Nested limits key order:

1. control_envelope_bytes
2. stream_limit
3. wall_time_seconds
4. cpu_time_seconds
5. resident_memory_bytes

The record uses the accepted VID-PROBE-P00-001 profile identity and the exact P00 values already bound by the 002B1 OperationDescriptor, WorkerHello, and worker contract constants:

- Control envelope: 1048576 bytes.
- Stream limit: 256.
- Wall time: 60 seconds.
- Child CPU time: 60 seconds.
- Resident memory: 1073741824 bytes.
- Transport: local-staged-file.
- Capabilities: cancellation, then deadline.

The record MUST validate against VID-IMPL-P00-002B1-RESOURCE.schema.json with no additional properties.

## Control-protocol record

Path: Video Localization/contracts/VID-IMPL-P00-002B1/records/control-protocol.json

Top-level key order:

1. protocol_specification_id
2. protocol_specification_revision
3. control_protocol_version
4. operation
5. worker
6. transport

The record binds:

- SUI-SPEC-003, Frozen revision 1.0.
- Accepted control protocol version 1.0.0.
- Operation submark.video-localization.media.probe@1.0.0-p00.
- Implementation VID-IMPL-P00-002B1, profile VID-PROBE-P00-001.
- The seven existing 002B1 schema reference IDs in schema-bundle order.
- Accepted cancellation/deadline capabilities and rejected capability set.
- Windows x64 worker profile, registered executable identity, accepted limits, and transport.
- local-staged-file transport with network capability unsupported.

worker_instance_id is intentionally excluded because it is a runtime Worker Instance identity and changes across worker restarts; it is not a profile-digest identity.

## Authority references

- _shared/specifications/SUI-SPEC-003-job-worker-artifact-protocol.md, Frozen revision 1.0.
- Video Localization/decisions/VID-DEC-003-initial-support-profiles.md, VID-DEC-003@1.0.
- Video Localization/contracts/VID-IMPL-P00-002B1/schema-bundle.json, unchanged current digest F78BF8A7AF8616B913D03539EB66ACE6BF670F80C685E791521E395396F3397F.
- Video Localization/contracts/VID-IMPL-P00-002B1/schemas/VID-IMPL-P00-002B1-RESOURCE.schema.json.
- Video Localization/contracts/VID-IMPL-P00-002B1/fixtures/operation-descriptor.json.
- Video Localization/contracts/VID-IMPL-P00-002B1/fixtures/worker-hello.json.
- Video Localization/workers/VideoLocalization.Worker/src/probe_adapter_contract.rs.

The candidate does not alter the seven-schema bundle or its current F78BF8A7AF8616B913D03539EB66ACE6BF670F80C685E791521E395396F3397F digest.

## Independent reviewer acceptance criteria

1. Recompute all candidate file hashes over complete bytes and confirm the manifest and checkpoint agree.
2. Confirm UTF-8 without BOM, LF-only line endings, two-space indentation, explicit key order, no trailing whitespace, and exactly one final LF for both JSON records.
3. Validate resource-profile.json against the existing 002B1 RESOURCE schema and require all required fields, exact accepted limits/capabilities, and no additional properties.
4. Confirm every operation, worker, transport, capability, executable, and bounded-limit value matches the cited accepted authorities.
5. Confirm control-protocol.json uses SUI-SPEC-003 revision 1.0 and control version 1.0.0 without placeholders.
6. Confirm the candidate manifest binds both records, the profile document, schema/reference authorities, exact hashes, purpose, and candidate status.
7. Confirm no runtime, builder, implementation, registry, Git, or accepted/frozen artifact was modified by this authoring task.
8. Record an independent review disposition; candidate_for_review MUST remain until owner-authorized acceptance explicitly changes it.

## Disposition

This is an owner-authorized freeze candidate under OWNER-AUTH-V2, not an approval or self-acceptance record.
