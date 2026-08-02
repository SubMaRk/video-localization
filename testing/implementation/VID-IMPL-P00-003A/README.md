# VID-IMPL-P00-003A

Scope: waveform contract/cache/foundation for deterministic cache-key and whole-object bulk descriptor behavior.

## Synthetic fixture notes

- No media parsing or decoding occurs.
- Fixtures exercise negative and positive contract-only vectors:
  - deterministic cache-key inputs/outputs
  - bulk descriptor hashing and validation
  - negative mutation vectors for malformed descriptor requirements

## Canonical artifacts under test

- `contracts/VID-IMPL-P00-003A/schema-bundle.json`
- `contracts/VID-IMPL-P00-003A/records/manifest.json`
- `contracts/VID-IMPL-P00-003A/fixtures/operation-descriptor.json`
- `contracts/VID-IMPL-P00-003A/fixtures/worker-hello.json`
- `contracts/VID-IMPL-P00-003A/fixtures/cache-key-and-descriptor-fixtures.json`
- `contracts/VID-IMPL-P00-003A/records/operation-profile.json`
- `contracts/VID-IMPL-P00-003A/records/resource-profile.json`
- `contracts/VID-IMPL-P00-003A/records/policy-profile.json`
- `contracts/VID-IMPL-P00-003A/records/retry-profile.json`
- `Video Localization/workers/VideoLocalization.Worker/src/waveform_contract.rs`
- `Video Localization/workers/VideoLocalization.Worker/tests/waveform_contract.rs`

## Negative fixture matrix

- request port/identity mismatch conditions for schema validation
- non-zero cache key ordering checks
- descriptor without required `chunk_hashes_omitted=true`
- malformed descriptor hashes and representation constraints
