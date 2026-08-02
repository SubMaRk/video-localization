# VID-IMPL-P00-002A

Scope: local-file preflight and deterministic SHA-256 candidate observation.

- Add a Rust library boundary under `workers/VideoLocalization.Worker`.
- Validate local file inputs for missing paths, non-file paths, path traversal, ADS/device/network syntax, and two-tebibyte source ceiling before hashing.
- Compute SHA-256 by streaming fixed-size chunks with `sha2 = 0.11.0`.
- Return immutable candidate observation data including byte length, hashes, pre/post file metadata, canonical path, and algorithm version.
- Path/hash/canonicalization metadata are **non-authoritative observations**.
  - Callers must assign stable Video asset identities independently and bind observations to caller-owned asset IDs.
- Hash observation uses a single read handle per attempt; on Windows the read handle shares read-only so write/delete opens are denied during hashing. Metadata for mutation checks comes from that same handle, but hashes/metadata remain non-authoritative observation inputs and must not be treated as durable source identity.
- Prohibit media codec selection, probing, and network access in this slice.
