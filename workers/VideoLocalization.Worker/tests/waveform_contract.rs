use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use videolocalization_worker::waveform_contract as wc;

const CONTROL_PROTOCOL_SHA: &str = "2539250e640517eccc9dccf5feebce6cf9ec89acc26cef9ee08edc9d914d9f02";
const SCHEMA_BUNDLE_SHA: &str = "75bb644150e52545bef715fe39bf298f88cc95a1e23e842eb5b1e28e67262e0e";
const SCHEMA_BUNDLE_002B1_SHA: &str = "f78bf8a7af8616b913d03539eb66ace6bf670f80c685e791521e395396f3397f";
const RECORD_CHECKSUMS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "control-protocol",
        "SUI-SPEC-003",
        "1.0",
        "Video Localization/contracts/VID-IMPL-P00-002B1/records/control-protocol.json",
        CONTROL_PROTOCOL_SHA,
    ),
    (
        "operation-profile",
        "VID-WAVEFORM-OPERATION-PROFILE-P00-003A",
        "0.1.0-p00",
        "Video Localization/contracts/VID-IMPL-P00-003A/records/operation-profile.json",
        "2374d0730488c262148516051431b09acb0e2b06fe5ec901d5ffdb2739a889db",
    ),
    (
        "resource-profile",
        "VID-WAVEFORM-RESOURCE-P00-003A",
        "0.1.0-p00",
        "Video Localization/contracts/VID-IMPL-P00-003A/records/resource-profile.json",
        "6da872ad96a9f0840cc62bf2bd27201aaf8d877002d5add749833710d3e5e07d",
    ),
    (
        "policy-profile",
        "VID-WAVEFORM-POLICY-P00-003A",
        "0.1.0-p00",
        "Video Localization/contracts/VID-IMPL-P00-003A/records/policy-profile.json",
        "ad4838a59d06ee676b4b94be43d6f9dd5c7ded88f5c57cfd7f7f56bcae82c368",
    ),
    (
        "retry-profile",
        "VID-WAVEFORM-RETRY-P00-003A",
        "0.1.0-p00",
        "Video Localization/contracts/VID-IMPL-P00-003A/records/retry-profile.json",
        "35de87a3fe7681a485d2b8d4199f5fe89b8d27c38b1500043dd01397d9b3f8e1",
    ),
    (
        "schema-bundle",
        "VID-IMPL-P00-003A",
        "0.1.0-p00",
        "Video Localization/contracts/VID-IMPL-P00-003A/schema-bundle.json",
        SCHEMA_BUNDLE_SHA,
    ),
];

#[derive(Debug, Deserialize, Clone)]
struct ManifestRecord {
    role: String,
    record_id: String,
    record_version: String,
    path: String,
    #[serde(alias = "sha256")]
    hash: String,
    schema_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct ManifestFixtureRecord {
    role: String,
    path: String,
    #[serde(alias = "sha256")]
    hash: String,
}

#[derive(Debug, Deserialize, Clone)]
struct ManifestSchemaRecord {
    schema_id: String,
    schema_version: String,
    path: String,
    sha256: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Manifest {
    record_digest_profile_id: String,
    record_digest_profile_version: String,
    records: Vec<ManifestRecord>,
    #[serde(default)]
    fixture_records: Vec<ManifestFixtureRecord>,
    #[serde(default)]
    schema_records: Vec<ManifestSchemaRecord>,
    authorities: Vec<AuthorityRecord>,
}

#[derive(Debug, Deserialize, Clone)]
struct AuthorityRecord {
    id: String,
    revision: String,
    #[serde(alias = "sha256")]
    hash: String,
}

#[derive(Debug, Deserialize)]
struct SchemaBundle {
    artifact_id: String,
    artifact_version: String,
    operation_id: String,
    operation_version: String,
    implementation_profile_id: String,
    implementation_id: String,
    schema_records: Vec<ManifestSchemaRecord>,
    #[allow(dead_code)]
    #[serde(default)]
    fixture_records: Vec<ManifestFixtureRecord>,
}

#[derive(Debug, Deserialize)]
struct FixtureBundle {
    cache_key_positive_cases: Vec<Value>,
    cache_key_negative_cases: Vec<Value>,
    bulk_descriptor_positive_cases: Vec<Value>,
    bulk_descriptor_negative_cases: Vec<Value>,
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("..")
}

fn read_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read failed for {}: {error}", path.display()))
}

fn read_json(path: &Path) -> Value {
    serde_json::from_str(&read_text(path)).unwrap_or_else(|error| panic!("json parse failed for {}: {error}", path.display()))
}

fn workspace_path(root: &Path, path: &str) -> PathBuf {
    if path.starts_with("Video Localization/") || path.starts_with("Video Localization\\") {
        root.join(path)
    } else {
        root.join("Video Localization").join(path)
    }
}

fn sha256_hex_of_file(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|error| panic!("read failed for {}: {error}", path.display()));
    let mut digest = Sha256::new();
    digest.update(&bytes);
    digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn read_schema_records(root: &Path, path: &str) -> HashMap<String, Value> {
    let bundle: SchemaBundle = serde_json::from_str(&read_text(&root.join(path))).expect("schema-bundle parse");
    let mut values = HashMap::new();
    for entry in bundle.schema_records {
        values.insert(entry.schema_id, read_json(&workspace_path(root, &entry.path)));
    }
    values
}

fn validate_schema_records(records: &[ManifestSchemaRecord], root: &Path) {
    let required_schema_ids: HashSet<&str> = HashSet::from_iter([
        "VID-IMPL-P00-003A-WAVEFORM-INPUT",
        "VID-IMPL-P00-003A-WAVEFORM-ARTIFACT",
        "VID-IMPL-P00-003A-WAVEFORM-PARAMETER",
        "VID-IMPL-P00-003A-WAVEFORM-STRUCTURED-ERROR",
        "VID-IMPL-P00-003A-WAVEFORM-RESOURCE",
        "VID-IMPL-P00-003A-WAVEFORM-POLICY",
        "VID-IMPL-P00-003A-WAVEFORM-BULK-DESCRIPTOR",
        "VID-IMPL-P00-003A-WAVEFORM-CONTROL",
        "VID-IMPL-P00-003A-WAVEFORM-WORKER-HELLO",
        "VID-IMPL-P00-003A-WAVEFORM-OPERATION-PROFILE",
    ]);
    let mut observed: HashSet<String> = HashSet::new();

    assert_eq!(records.len(), required_schema_ids.len());
    for record in records {
        assert!(required_schema_ids.contains(record.schema_id.as_str()));
        observed.insert(record.schema_id.clone());
        let file_hash = sha256_hex_of_file(&workspace_path(root, &record.path));
        assert_eq!(record.sha256.to_lowercase(), file_hash);
        assert_eq!(record.schema_version, "0.1.0-p00");
    }
    assert_eq!(required_schema_ids.len(), observed.len());
}

fn assert_manifest_is_closed(manifest: &Manifest) -> Result<(), String> {
    if manifest.record_digest_profile_id != "VID-WAVEFORM-ADMISSION-MANIFEST-P00-003A" {
        return Err("invalid admission manifest profile".to_string());
    }
    if manifest.record_digest_profile_version != "0.1.0-p00" {
        return Err("invalid admission manifest version".to_string());
    }

    let mut records = HashMap::new();
    for record in &manifest.records {
        records.insert(record.role.as_str(), record);
    }
    let expected_roles: HashSet<&str> = HashSet::from_iter(RECORD_CHECKSUMS.iter().map(|(r, ..)| *r));
    let observed_roles: HashSet<&str> = records.keys().copied().collect();
    if observed_roles != expected_roles {
        return Err("required manifest roles mismatch".to_string());
    }

    for (role, expected_record_id, expected_version, expected_path, expected_hash) in RECORD_CHECKSUMS {
        let record = records
            .get(*role)
            .ok_or_else(|| format!("missing record role {role}"))?;
        if record.record_id != *expected_record_id {
            return Err(format!("record id mismatch for {role}"));
        }
        if record.record_version != *expected_version {
            return Err(format!("record version mismatch for {role}"));
        }
        if record.path != *expected_path {
            return Err(format!("record path mismatch for {role}"));
        }
        if record.hash.to_lowercase() != *expected_hash {
            return Err(format!("record hash mismatch for {role}"));
        }
        if sha256_hex_of_file(&workspace_path(&workspace_root(), &record.path)) != *expected_hash {
            return Err(format!("record file hash mismatch for {role}"));
        }
        if *role == "control-protocol" {
            if record.schema_id.as_deref() != Some("VID-IMPL-P00-003A-WAVEFORM-CONTROL") {
                return Err("control schema id must be VID-IMPL-P00-003A-WAVEFORM-CONTROL".to_string());
            }
        }
    }

    let required_fixture_roles: HashSet<&str> = HashSet::from_iter([
        "operation-descriptor",
        "worker-hello",
        "cache-and-descriptor-fixtures",
    ]);
    let fixture_records: HashMap<&str, &ManifestFixtureRecord> =
        manifest.fixture_records.iter().map(|record| (record.role.as_str(), record)).collect();
    for role in &required_fixture_roles {
        let record = fixture_records
            .get(*role)
            .ok_or_else(|| format!("missing fixture role {role}"))?;
        let expected = record.path.to_lowercase();
        let observed = sha256_hex_of_file(&workspace_path(&workspace_root(), &expected)).to_lowercase();
        if observed != record.hash.to_lowercase() {
            return Err(format!("fixture hash mismatch for {role}"));
        }
    }

    validate_schema_records(&manifest.schema_records, &workspace_root());

    let required_authorities: HashSet<&str> = HashSet::from_iter([
        "SUI-SPEC-003",
        "VID-DEC-003",
        "VID-IMPL-P00-003A-SCHEMA-BUNDLE",
        "VID-IMPL-P00-003A-INPUT",
        "VID-IMPL-P00-003A-ARTIFACT",
        "VID-IMPL-P00-003A-PARAMETER",
        "VID-IMPL-P00-003A-STRUCTURED-ERROR",
        "VID-IMPL-P00-003A-RESOURCE",
        "VID-IMPL-P00-003A-POLICY",
        "VID-IMPL-P00-003A-BULK-DESCRIPTOR",
        "VID-IMPL-P00-003A-CONTROL",
        "VID-IMPL-P00-003A-WORKER-HELLO",
        "VID-IMPL-P00-003A-OPERATION-PROFILE",
        "VID-IMPL-P00-002B1-SCHEMA-BUNDLE",
    ]);
    let authority_ids: HashSet<&str> = manifest.authorities.iter().map(|a| a.id.as_str()).collect();
    if authority_ids != required_authorities {
        return Err("required authorities mismatch".to_string());
    }
    let schema_bundle_authority = manifest
        .authorities
        .iter()
        .find(|item| item.id == "VID-IMPL-P00-002B1-SCHEMA-BUNDLE")
        .ok_or_else(|| "missing VID-IMPL-P00-002B1-SCHEMA-BUNDLE authority".to_string())?;
    if schema_bundle_authority.revision != "0.1.0-p00" {
        return Err("wrong schema-bundle revision".to_string());
    }
    if schema_bundle_authority.hash.to_lowercase() != SCHEMA_BUNDLE_002B1_SHA {
        return Err("wrong 002B1 schema bundle hash".to_string());
    }
    Ok(())
}

fn assert_json_valid(schema: &Value, value: &Value, label: &str) {
    let validator = jsonschema::draft202012::new(schema).unwrap_or_else(|error| panic!("{label} schema failed to compile: {error}"));
    let errors: Vec<_> = validator.iter_errors(value).collect();
    assert!(
        errors.is_empty(),
        "{label} should be valid: {}",
        errors
            .into_iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
}

fn assert_json_rejects(schema: &Value, value: &Value, label: &str) {
    let validator = jsonschema::draft202012::new(schema).unwrap_or_else(|error| panic!("{label} schema failed to compile: {error}"));
    let errors: Vec<_> = validator.iter_errors(value).collect();
    assert!(!errors.is_empty(), "{label} must be rejected by schema");
}

fn cbor_skip_item(bytes: &[u8], index: &mut usize) -> bool {
    if *index >= bytes.len() {
        return false;
    }
    let first = bytes[*index];
    let major = first >> 5;
    let info = first & 0x1f;
    *index += 1;

    let mut read_u64 = |i: &mut usize| -> Option<u64> {
        let len = if info <= 23 {
            info as u64
        } else if info == 24 {
            if *i >= bytes.len() {
                return None;
            }
            let value = bytes[*i] as u64;
            *i += 1;
            value
        } else if info == 25 {
            if *i + 2 > bytes.len() {
                return None;
            }
            let value = ((bytes[*i] as u64) << 8) | bytes[*i + 1] as u64;
            *i += 2;
            value
        } else if info == 26 {
            if *i + 4 > bytes.len() {
                return None;
            }
            let value = ((bytes[*i] as u64) << 24)
                | ((bytes[*i + 1] as u64) << 16)
                | ((bytes[*i + 2] as u64) << 8)
                | (bytes[*i + 3] as u64);
            *i += 4;
            value
        } else if info == 27 {
            if *i + 8 > bytes.len() {
                return None;
            }
            let value = ((bytes[*i] as u64) << 56)
                | ((bytes[*i + 1] as u64) << 48)
                | ((bytes[*i + 2] as u64) << 40)
                | ((bytes[*i + 3] as u64) << 32)
                | ((bytes[*i + 4] as u64) << 24)
                | ((bytes[*i + 5] as u64) << 16)
                | ((bytes[*i + 6] as u64) << 8)
                | (bytes[*i + 7] as u64);
            *i += 8;
            value
        } else {
            return None;
        };
        Some(len)
    };

    let length = match major {
        0 | 1 => return true,
        2 => match read_u64(index) {
            Some(len) => len as usize,
            None => return false,
        },
        3 => match read_u64(index) {
            Some(len) => {
                let len = match usize::try_from(len) {
                    Ok(len) => len,
                    Err(_) => return false,
                };
                *index += len;
                return *index <= bytes.len();
            }
            None => return false,
        },
        4 => match read_u64(index) {
            Some(len) => len as usize,
            None => return false,
        },
        5 => match read_u64(index) {
            Some(len) => len as usize,
            None => return false,
        },
        6 | 7 => return true,
        0b111 | 0b110 => return true,
        _ => return false,
    };

    if major == 4 || major == 5 {
        for _ in 0..length {
            if !cbor_skip_item(bytes, index) {
                return false;
            }
            if major == 5 {
                if !cbor_skip_item(bytes, index) {
                    return false;
                }
            }
        }
        return true;
    }
    false
}

fn cbor_keyed_items(bytes: &[u8], index: &mut usize) -> Option<Vec<(Vec<u8>, usize, usize)>> {
    if *index >= bytes.len() {
        return None;
    }
    let first = bytes[*index];
    let major = first >> 5;
    if major != 5 {
        return None;
    }
    *index += 1;
    let info = first & 0x1f;
    let mut len: usize = match info {
        v if v <= 23 => v as usize,
        24 => {
            if *index >= bytes.len() {
                return None;
            }
            let value = bytes[*index] as usize;
            *index += 1;
            value
        }
        25 => {
            if *index + 1 >= bytes.len() {
                return None;
            }
            let value = ((bytes[*index] as usize) << 8) | bytes[*index + 1] as usize;
            *index += 2;
            value
        }
        26 => {
            if *index + 3 >= bytes.len() {
                return None;
            }
            let value = ((bytes[*index] as usize) << 24)
                | ((bytes[*index + 1] as usize) << 16)
                | ((bytes[*index + 2] as usize) << 8)
                | (bytes[*index + 3] as usize);
            *index += 4;
            value
        }
        27 => {
            if *index + 7 >= bytes.len() {
                return None;
            }
            let value = ((bytes[*index] as usize) << 56)
                | ((bytes[*index + 1] as usize) << 48)
                | ((bytes[*index + 2] as usize) << 40)
                | ((bytes[*index + 3] as usize) << 32)
                | ((bytes[*index + 4] as usize) << 24)
                | ((bytes[*index + 5] as usize) << 16)
                | ((bytes[*index + 6] as usize) << 8)
                | (bytes[*index + 7] as usize);
            *index += 8;
            value
        }
        _ => return None,
    };

    let mut pairs = Vec::new();
    for _ in 0..len {
        let key_start = *index;
        if !cbor_skip_item(bytes, index) {
            return None;
        }
        let key_end = *index;
        let value_start = *index;
        if !cbor_skip_item(bytes, index) {
            return None;
        }
        let value_end = *index;
        pairs.push((bytes[key_start..key_end].to_vec(), value_start, value_end));
    }
    Some(pairs)
}

fn cbor_map_keys_are_sorted(bytes: &[u8], start: usize) -> bool {
    let mut index = start;
    let pairs = match cbor_keyed_items(bytes, &mut index) {
        Some(pairs) => pairs,
        None => return false,
    };

    let key_bytes: Vec<_> = pairs.iter().map(|(key, _, _)| key.clone()).collect();
    if key_bytes.windows(2).any(|window| window[0] > window[1]) {
        return false;
    }
    pairs.iter().all(|(_, value_start, value_end)| {
        if *value_start >= bytes.len() || *value_end > bytes.len() {
            return false;
        }
        if bytes[*value_start] >> 5 == 5 {
            return cbor_map_keys_are_sorted(bytes, *value_start);
        }
        true
    })
}

fn sample_cache_profile() -> wc::WaveformCacheKeyProfile {
    wc::WaveformCacheKeyProfile {
        cache_key_schema: wc::CACHE_KEY_SCHEMA_ID.to_string(),
        operation: wc::WaveformCacheKeyProfileIdentity {
            id: wc::OPERATION_ID.to_string(),
            version: wc::OPERATION_VERSION.to_string(),
        },
        implementation: wc::WaveformCacheKeyProfileIdentity {
            id: wc::IMPLEMENTATION_ID.to_string(),
            version: wc::IMPLEMENTATION_PROFILE_ID.to_string(),
        },
        inputs: vec![wc::WaveformCacheInputPort {
            order: 0,
            kind: "waveform_request".to_string(),
            schema: wc::OPERATION_SCHEMA_ID.to_string(),
            content: "source".to_string(),
            revision: "0.1.0-p00".to_string(),
            sample_range: "[0,4096)".to_string(),
        }],
        parameters: wc::WaveformCacheParameters {
            cache_key_schema: wc::CACHE_KEY_SCHEMA_ID.to_string(),
            cache_key_seed: "waveform-seed-003a".to_string(),
            environment_equivalence: wc::WaveformEnvironmentEquivalence {
                os_family: "windows".to_string(),
                arch_family: "x64".to_string(),
                worker_family: "vid-waveform-worker".to_string(),
                containerization: false,
                capability_bundle: "audio-none".to_string(),
            },
            resource_profile_id: "VID-WAVEFORM-RESOURCE-P00-003A".to_string(),
            policy_profile_id: "VID-WAVEFORM-POLICY-P00-003A".to_string(),
            retry_profile_id: "VID-WAVEFORM-RETRY-P00-003A".to_string(),
            representation: wc::WaveformCacheRepresentation {
                peak: "waveform-peak-v1".to_string(),
                time: "sample".to_string(),
                channel: "stereo-major".to_string(),
                downmix: "matrix-stereo-downmix-v1".to_string(),
                resampling: "resample-none".to_string(),
                stability: "fixed".to_string(),
            },
        },
        dependencies: wc::WaveformCacheDependencies {
            resource_profile: "VID-WAVEFORM-RESOURCE-P00-003A".to_string(),
            policy_profile: "VID-WAVEFORM-POLICY-P00-003A".to_string(),
            retry_profile: "VID-WAVEFORM-RETRY-P00-003A".to_string(),
            operation_profile: "VID-WAVEFORM-OPERATION-PROFILE-P00-003A".to_string(),
            cache_key_schema: wc::CACHE_KEY_SCHEMA_ID.to_string(),
        },
        output_contract: wc::WaveformCacheOutputContract {
            artifact_id: "waveform-artifact-request".to_string(),
            artifact_version: "0.1.0-p00".to_string(),
            output_schema_id: wc::OUTPUT_SCHEMA_ID.to_string(),
        },
        seed: "waveform-seed-003a".to_string(),
        environment_equivalence: wc::WaveformEnvironmentEquivalence {
            os_family: "windows".to_string(),
            arch_family: "x64".to_string(),
            worker_family: "vid-waveform-worker".to_string(),
            containerization: false,
            capability_bundle: "audio-none".to_string(),
        },
        resource: wc::WaveformCacheResource {
            resource_profile_id: "VID-WAVEFORM-RESOURCE-P00-003A".to_string(),
        },
        policy: wc::WaveformCachePolicy {
            network: "denied".to_string(),
            watchdog_required: true,
            job_object_required: true,
            media_parse: false,
        },
        representation: wc::WaveformCacheRepresentation {
            peak: "waveform-peak-v1".to_string(),
            time: "sample".to_string(),
            channel: "stereo-major".to_string(),
            downmix: "matrix-stereo-downmix-v1".to_string(),
            resampling: "resample-none".to_string(),
            stability: "fixed".to_string(),
        },
    }
}

fn sample_bulk_descriptor() -> wc::WaveformBulkDescriptor {
    wc::WaveformBulkDescriptor {
        version: wc::BULK_DESCRIPTOR_PROFILE.to_string(),
        mode: "read-only".to_string(),
        handle_identity: "handle-vid-waveform-003a".to_string(),
        attempt_id: "attempt-001".to_string(),
        lease_id: "lease-001".to_string(),
        cancel_scope: "attempt-001".to_string(),
        expiry: "2099-01-01T00:00:00Z".to_string(),
        integrity: wc::WaveformOutputContractDigest {
            algorithm: "sha-256".to_string(),
            value: "9f1c4d4fdb5fc8aaec58d9ebf8db8a5d4b9df3c8c1f4f5ad1f0d2c9e4a6c3b6d".to_string(),
        },
        chunk_hashes_omitted: true,
        length: 4096,
        payload_schema: wc::OUTPUT_SCHEMA_ID.to_string(),
        source_publication_fence: "fence-001".to_string(),
        representation: wc::WaveformBulkRepresentation {
            peak: "waveform-peak-v1".to_string(),
            time: "sample".to_string(),
            channel: "stereo-major".to_string(),
            downmix: "matrix-stereo-downmix-v1".to_string(),
            resampling: "resample-none".to_string(),
            source_timeline: "timeline-001".to_string(),
            sample_range: "[0,4096)".to_string(),
        },
        descriptor_hash: None,
    }
}

fn cache_key_expected_digest(case: &Value) -> Option<String> {
    case.get("expected_cache_key_digest")
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn bulk_expected_descriptor_digest(case: &Value) -> Option<String> {
    case.get("expected_descriptor_digest")
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn cache_profile_from_case(case: &Value) -> wc::WaveformCacheKeyProfile {
    let mut profile = sample_cache_profile();
    if let Some(seed) = case.get("seed").and_then(Value::as_str) {
        profile.seed = seed.to_string();
    }
    if let Some(resource_profile_id) = case.get("resource_profile_id").and_then(Value::as_str) {
        profile.parameters.resource_profile_id = resource_profile_id.to_string();
        profile.resource.resource_profile_id = resource_profile_id.to_string();
    }
    if let Some(policy_profile_id) = case.get("policy_profile_id").and_then(Value::as_str) {
        profile.parameters.policy_profile_id = policy_profile_id.to_string();
    }
    if let Some(retry_profile_id) = case.get("retry_profile_id").and_then(Value::as_str) {
        profile.parameters.retry_profile_id = retry_profile_id.to_string();
        profile.dependencies.retry_profile = retry_profile_id.to_string();
    }
    if let Some(representation) = case.get("representation").and_then(Value::as_object) {
        profile.representation.peak = representation
            .get("peak")
            .and_then(Value::as_str)
            .unwrap_or(&profile.representation.peak)
            .to_string();
        profile.representation.time = representation
            .get("time")
            .and_then(Value::as_str)
            .unwrap_or(&profile.representation.time)
            .to_string();
        profile.representation.channel = representation
            .get("channel")
            .and_then(Value::as_str)
            .unwrap_or(&profile.representation.channel)
            .to_string();
        profile.representation.downmix = representation
            .get("downmix")
            .and_then(Value::as_str)
            .unwrap_or(&profile.representation.downmix)
            .to_string();
        profile.representation.resampling = representation
            .get("resampling")
            .and_then(Value::as_str)
            .unwrap_or(&profile.representation.resampling)
            .to_string();
        profile.representation.stability = representation
            .get("stability")
            .and_then(Value::as_str)
            .unwrap_or(&profile.representation.stability)
            .to_string();
    }

    if let Some(inputs) = case.get("input_ports").and_then(Value::as_array) {
        profile.inputs = inputs
            .iter()
            .map(|input| wc::WaveformCacheInputPort {
                order: input.get("order").and_then(Value::as_u64).unwrap_or(0),
                kind: input.get("type").and_then(Value::as_str).unwrap_or("waveform_request").to_string(),
                schema: input
                    .get("schema")
                    .and_then(Value::as_str)
                    .unwrap_or(&wc::OPERATION_SCHEMA_ID)
                    .to_string(),
                content: input.get("content").and_then(Value::as_str).unwrap_or("source").to_string(),
                revision: input
                    .get("revision")
                    .and_then(Value::as_str)
                    .unwrap_or("0.1.0-p00")
                    .to_string(),
                sample_range: input
                    .get("sample_range")
                    .and_then(Value::as_str)
                    .unwrap_or("[0,4096)")
                    .to_string(),
            })
            .collect();
    }

    if profile.inputs.is_empty() {
        profile.inputs = vec![wc::WaveformCacheInputPort {
            order: 0,
            kind: "waveform_request".to_string(),
            schema: wc::OPERATION_SCHEMA_ID.to_string(),
            content: "source".to_string(),
            revision: "0.1.0-p00".to_string(),
            sample_range: "[0,4096)".to_string(),
        }];
    }

    profile
}

fn fixture_case_payload(length: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; length];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = (index % 251) as u8;
    }
    bytes
}

fn sample_retry_profile() -> wc::WaveformRetryProfile {
    wc::WaveformRetryProfile {
        max_total_attempts: 2,
        base_wait_millis: 500,
        max_wait_millis: 500,
        backoff: "fixed".to_string(),
        cumulative_wall_seconds: 30,
        cumulative_cpu_seconds: 30,
        cumulative_ram_bytes: 268_435_456,
        cumulative_staging_bytes: 536_870_912,
        cost_ceiling: 2,
        applicable_codes: vec![
            "VID-WAVEFORM-WORKER-TRANSIENT".to_string(),
            "VID-WAVEFORM-STORAGE-TRANSIENT".to_string(),
        ],
    }
}

fn sample_publication_attempt() -> (wc::WaveformPublicationAttempt, Vec<u8>) {
    let mut descriptor = sample_bulk_descriptor();
    let payload = fixture_case_payload(descriptor.length as usize);
    descriptor.integrity.value = wc::sha256_hex_of_bytes(&payload);
    let digest = wc::bulk_descriptor_digest(&descriptor).expect("descriptor digest");
    descriptor.descriptor_hash = Some(digest);

    let attempt = wc::WaveformPublicationAttempt {
        attempt_id: "attempt-001".to_string(),
        lease: wc::WaveformPublicationLease {
            lease_id: "lease-001".to_string(),
            lease_epoch: 1,
            minimum_acceptable_lease_epoch: 1,
            expected_fence_token: "fence-001".to_string(),
            observed_fence_token: "fence-001".to_string(),
            staged_copy_hash: wc::sha256_hex_of_bytes(&payload),
            staged_copy_length: descriptor.length,
            cache_authority: false,
        },
        descriptor,
    };
    (attempt, payload)
}

#[test]
fn manifest_and_schema_bundle_are_authoritative_and_fail_closed() {
    let manifest_path = workspace_root().join("Video Localization/contracts/VID-IMPL-P00-003A/records/manifest.json");
    let manifest: Manifest = serde_json::from_str(&read_text(&manifest_path)).expect("manifest parse");
    assert!(
        assert_manifest_is_closed(&manifest).is_ok(),
        "manifest not fail-closed against required exact records"
    );
}

#[test]
fn manifest_missing_or_mismatched_bindings_fail_closed() {
    let manifest_path = workspace_root().join("Video Localization/contracts/VID-IMPL-P00-003A/records/manifest.json");
    let manifest: Manifest = serde_json::from_str(&read_text(&manifest_path)).expect("manifest parse");
    let mut missing_record = manifest.clone();
    if let Some(index) = missing_record.records.iter().position(|record| record.role == "operation-profile") {
        missing_record.records.remove(index);
    }
    assert!(assert_manifest_is_closed(&missing_record).is_err());

    let mut mismatched = manifest.clone();
    if let Some(record) = mismatched.records.iter_mut().find(|record| record.role == "resource-profile") {
        record.hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();
    }
    assert!(assert_manifest_is_closed(&mismatched).is_err());
}

#[test]
fn schema_bundle_hashes_and_required_schema_ids_are_exact() {
    let bundle = read_json(
        &workspace_root().join("Video Localization/contracts/VID-IMPL-P00-003A/schema-bundle.json"),
    );
    let schema_bundle: SchemaBundle = serde_json::from_value(bundle).expect("schema-bundle parse");
    assert!(schema_bundle
        .artifact_id
        .eq_ignore_ascii_case("VID-IMPL-P00-003A"));
    assert!(schema_bundle
        .implementation_id
        .eq_ignore_ascii_case("VID-IMPL-P00-003A"));

    validate_schema_records(&schema_bundle.schema_records, &workspace_root());

    let records: HashSet<&str> = schema_bundle.schema_records.iter().map(|r| r.schema_id.as_str()).collect();
    let expected: HashSet<&str> = HashSet::from_iter([
        "VID-IMPL-P00-003A-WAVEFORM-INPUT",
        "VID-IMPL-P00-003A-WAVEFORM-ARTIFACT",
        "VID-IMPL-P00-003A-WAVEFORM-PARAMETER",
        "VID-IMPL-P00-003A-WAVEFORM-STRUCTURED-ERROR",
        "VID-IMPL-P00-003A-WAVEFORM-RESOURCE",
        "VID-IMPL-P00-003A-WAVEFORM-POLICY",
        "VID-IMPL-P00-003A-WAVEFORM-BULK-DESCRIPTOR",
        "VID-IMPL-P00-003A-WAVEFORM-CONTROL",
        "VID-IMPL-P00-003A-WAVEFORM-WORKER-HELLO",
        "VID-IMPL-P00-003A-WAVEFORM-OPERATION-PROFILE",
    ]);
    assert_eq!(records, expected);
}

#[test]
fn operation_and_worker_hello_profile_bindings_are_exact() {
    let root = workspace_root();
    let schema_records = read_schema_records(&root, "Video Localization/contracts/VID-IMPL-P00-003A/schema-bundle.json");

    let op_descriptor = read_json(&root.join("Video Localization/contracts/VID-IMPL-P00-003A/fixtures/operation-descriptor.json"));
    let op_profile = read_json(&root.join("Video Localization/contracts/VID-IMPL-P00-003A/records/operation-profile.json"));
    let worker_hello = read_json(&root.join("Video Localization/contracts/VID-IMPL-P00-003A/fixtures/worker-hello.json"));
    let hello_text = worker_hello.to_string().to_ascii_lowercase();

    let operation_schema = schema_records
        .get("VID-IMPL-P00-003A-WAVEFORM-INPUT")
        .expect("input schema")
        .clone();
    let hello_schema = schema_records
        .get("VID-IMPL-P00-003A-WAVEFORM-WORKER-HELLO")
        .expect("hello schema")
        .clone();
    let profile_schema = schema_records
        .get("VID-IMPL-P00-003A-WAVEFORM-OPERATION-PROFILE")
        .expect("operation profile schema")
        .clone();

    assert_json_valid(&operation_schema, &op_descriptor, "operation descriptor");
    assert_json_valid(&hello_schema, &worker_hello, "worker hello");
    assert_json_valid(&profile_schema, &op_profile, "operation profile");

    assert_eq!(op_descriptor["operation_id"], json!(wc::OPERATION_ID));
    assert_eq!(op_descriptor["operation_version"], json!(wc::OPERATION_VERSION));
    assert_eq!(op_descriptor["implementation_id"], json!(wc::IMPLEMENTATION_ID));
    assert_eq!(op_profile["implementation_id"], json!(wc::IMPLEMENTATION_ID));

    let output_ports = op_descriptor["output_ports"].as_array().expect("output_ports array");
    assert_eq!(output_ports.len(), 3);
    assert_eq!(output_ports[0]["port_type"], json!("waveform_artifact"));
    assert_eq!(output_ports[0]["max_bytes"], json!(65536));
    assert_eq!(output_ports[1]["port_type"], json!("waveform_error"));
    assert_eq!(output_ports[1]["max_bytes"], json!(32768));
    assert_eq!(output_ports[2]["port_type"], json!("waveform_peak_bulk"));
    assert_eq!(output_ports[2]["max_bytes"], json!(65536));

    let profile_ports = op_profile["ports"].as_array().expect("profile ports");
    assert_eq!(profile_ports.len(), 4);
    assert_eq!(profile_ports[0]["role"], json!("waveform_request"));
    assert_eq!(profile_ports[0]["max_bytes"], json!(262144));
    assert_eq!(profile_ports[1]["role"], json!("waveform_artifact"));
    assert_eq!(profile_ports[1]["max_bytes"], json!(65536), "artifact ceiling must remain at 65536");
    assert_eq!(profile_ports[2]["role"], json!("waveform_error"));
    assert_eq!(profile_ports[2]["max_bytes"], json!(32768));
    assert_eq!(profile_ports[3]["role"], json!("waveform_peak_bulk"));
    assert_eq!(profile_ports[3]["max_bytes"], json!(65536));
    assert_eq!(op_profile["policy"]["bulk_descriptor_profile"], json!(wc::BULK_DESCRIPTOR_PROFILE));

    assert!(op_descriptor["staged_lease"]["observed_fence_token"]
        == op_descriptor["staged_lease"]["expected_fence_token"]);
    assert_eq!(op_profile["operation_id"], json!(wc::OPERATION_ID));
    assert_eq!(op_profile["operation_version"], json!(wc::OPERATION_VERSION));

    let mut leaked = false;
    ["ffprobe", "ffmpeg", "chocolatey", "programdata"].iter().for_each(|needle| {
        if hello_text.contains(needle) {
            leaked = true;
        }
    });
    assert!(!leaked, "worker-hello must not leak ffprobe/ffmpeg execution identity");
}

#[test]
fn worker_hello_rejects_unauthorized_executable_identity() {
    let root = workspace_root();
    let schema_records = read_schema_records(&root, "Video Localization/contracts/VID-IMPL-P00-003A/schema-bundle.json");
    let worker_hello = read_json(&root.join("Video Localization/contracts/VID-IMPL-P00-003A/fixtures/worker-hello.json"));
    let schema = schema_records
        .get("VID-IMPL-P00-003A-WAVEFORM-WORKER-HELLO")
        .expect("worker hello schema");

    let mut bad = worker_hello.clone();
    if let Some(object) = bad.as_object_mut() {
        object.insert(
            "executable_path".to_string(),
            json!("C:\\ProgramData\\ffprobe.exe"),
        );
        object.insert("executable_sha256".to_string(), json!("000000000000000000000000000000000000000000000000000000000000000000"));
        object.insert("component_version".to_string(), json!("not-authorized"));
    }
    assert_json_rejects(schema, &bad, "unauthorized worker hello executable fields");
}

#[test]
fn cache_key_preimage_is_stable_with_sorted_cbor_key_bytes() {
    let mut profile_a = sample_cache_profile();
    let mut profile_b = sample_cache_profile();
    let extra_input = wc::WaveformCacheInputPort {
        order: 1,
        kind: "waveform_request".to_string(),
        schema: wc::OPERATION_SCHEMA_ID.to_string(),
        content: "source".to_string(),
        revision: "0.1.0-p00".to_string(),
        sample_range: "[0,4096)".to_string(),
    };
    profile_a.inputs.push(extra_input.clone());
    profile_b.inputs.push(extra_input);
    profile_b.inputs.swap(0, 1);
    let preimage_a = wc::cache_key_preimage(&profile_a).expect("preimage A");
    let preimage_b = wc::cache_key_preimage(&profile_b).expect("preimage B");
    assert_eq!(
        wc::cache_key_digest(&profile_a).expect("digest A"),
        wc::cache_key_digest(&profile_b).expect("digest B"),
    );
    assert_eq!(preimage_a, preimage_b);
    assert!(cbor_map_keys_are_sorted(&preimage_a, 0), "top-level and nested cbor maps must use RFC-8949 ordering");
}

#[test]
fn bulk_descriptor_positive_and_zero_length_boundary() {
    let mut descriptor = sample_bulk_descriptor();
    let digest = wc::bulk_descriptor_digest(&descriptor).expect("descriptor digest");
    descriptor.descriptor_hash = Some(digest);
    assert!(wc::validate_bulk_descriptor(&descriptor).is_ok());

    descriptor.length = 0;
    descriptor.descriptor_hash = Some(wc::bulk_descriptor_digest(&descriptor).expect("zero-length digest"));
    assert!(wc::validate_bulk_descriptor(&descriptor).is_ok(), "zero-length bulk payload is boundary-accepted");

    descriptor.length = 536_870_913;
    descriptor.descriptor_hash = Some("0".repeat(64));
    assert!(wc::validate_bulk_descriptor(&descriptor).is_err());
}

#[test]
fn state_mapping_matrix_covers_all_codes_and_recovery_boundaries() {
    let mapping_codes: HashSet<&str> = wc::VID_WAVEFORM_STATE_MAPPINGS
        .iter()
        .map(|entry| entry.video_disposition)
        .collect();

    let expected_codes: HashSet<&str> = HashSet::from_iter([
        "VID-WAVEFORM-CANCELLED",
        "VID-WAVEFORM-DEADLINE-EXCEEDED",
        "VID-WAVEFORM-RESOURCE-LIMIT",
        "VID-WAVEFORM-LEASE-EXPIRED",
        "VID-WAVEFORM-FENCE-MISMATCH",
        "VID-WAVEFORM-CACHE-STALE",
        "VID-WAVEFORM-CACHE-CORRUPT",
        "VID-WAVEFORM-INPUT-SCHEMA-INVALID",
        "VID-WAVEFORM-OUTPUT-SCHEMA-INVALID",
        "VID-WAVEFORM-INTEGRITY-MISMATCH",
        "VID-WAVEFORM-MALFORMED-DESCRIPTOR",
        "VID-WAVEFORM-PROTOCOL-INCOMPATIBLE",
        "VID-WAVEFORM-HANDLE-INCOMPATIBLE",
        "VID-WAVEFORM-PUBLICATION-FAILED",
        "VID-WAVEFORM-STORAGE-PERMANENT",
        "VID-WAVEFORM-JOURNAL-INVALID",
        "VID-WAVEFORM-PARTIAL-OUTPUT",
        "VID-WAVEFORM-METADATA-INVALID",
        "VID-WAVEFORM-IDENTITY-MISMATCH",
        "VID-WAVEFORM-POLICY-DENIED",
        "VID-WAVEFORM-WORKER-TRANSIENT",
        "VID-WAVEFORM-STORAGE-TRANSIENT",
        "VID-WAVEFORM-WORKER-PERMANENT",
        "VID-WAVEFORM-REVIEW",
    ]);
    assert_eq!(mapping_codes, expected_codes);
    let timed_out = wc::state_mapping_for_code("VID-WAVEFORM-DEADLINE-EXCEEDED")
        .expect("deadline mapping exists");
    assert_eq!(timed_out.shared_terminal_state, "TimedOut");
    assert_eq!(timed_out.retry_class, "non-retryable");
    assert_eq!(
        wc::state_mapping_for_code("VID-WAVEFORM-WORKER-TRANSIENT")
            .expect("transient retry mapping")
            .retry_class,
        "transient"
    );
    assert_eq!(
        wc::state_mapping_for_code("VID-WAVEFORM-STORAGE-TRANSIENT")
            .expect("storage transient mapping")
            .retry_class,
        "transient"
    );

    for mapping in wc::VID_WAVEFORM_STATE_MAPPINGS {
        assert!(!mapping.shared_terminal_state.is_empty());
        assert!(!mapping.retry_class.is_empty());
        assert!(!mapping.operation_stage.is_empty());
        assert!(!mapping.error_category.is_empty());
        assert!(!mapping.safe_recovery.is_empty());
        assert!(!mapping.identity_refs.is_empty());
        assert!(mapping.identity_refs.len() <= 5);
        assert!(!mapping.video_disposition.is_empty());
    }
}

#[test]
fn schema_mutation_matrix_rejects_required_and_unknown_fields() {
    let root = workspace_root();
    let schema_map = read_schema_records(&root, "Video Localization/contracts/VID-IMPL-P00-003A/schema-bundle.json");
    let fixture_path = root.join("Video Localization/contracts/VID-IMPL-P00-003A/fixtures/operation-descriptor.json");
    let worker_hello_path = root.join("Video Localization/contracts/VID-IMPL-P00-003A/fixtures/worker-hello.json");
    let cache_fixture_path = root.join("Video Localization/contracts/VID-IMPL-P00-003A/fixtures/cache-key-and-descriptor-fixtures.json");
    let operation_descriptor = read_json(&fixture_path);
    let worker_hello = read_json(&worker_hello_path);
    let cache_fixtures: FixtureBundle = serde_json::from_str(&read_text(&cache_fixture_path)).expect("fixture parse");

    let input_schema = schema_map
        .get("VID-IMPL-P00-003A-WAVEFORM-INPUT")
        .expect("input schema");
    let operation_profile_schema = schema_map
        .get("VID-IMPL-P00-003A-WAVEFORM-OPERATION-PROFILE")
        .expect("profile schema");
    let hello_schema = schema_map
        .get("VID-IMPL-P00-003A-WAVEFORM-WORKER-HELLO")
        .expect("hello schema");
    let structured_schema = schema_map
        .get("VID-IMPL-P00-003A-WAVEFORM-STRUCTURED-ERROR")
        .expect("structured schema");
    let bulk_schema = schema_map
        .get("VID-IMPL-P00-003A-WAVEFORM-BULK-DESCRIPTOR")
        .expect("bulk schema");
    let artifact_schema = schema_map
        .get("VID-IMPL-P00-003A-WAVEFORM-ARTIFACT")
        .expect("artifact schema");

    assert_json_valid(input_schema, &operation_descriptor, "valid operation descriptor");
    assert_json_valid(hello_schema, &worker_hello, "valid worker hello");
    if root.join("Video Localization/contracts/VID-IMPL-P00-003A/records/operation-profile.json").exists() {
        let profile = read_json(&root.join("Video Localization/contracts/VID-IMPL-P00-003A/records/operation-profile.json"));
        assert_json_valid(operation_profile_schema, &profile, "valid operation profile");
    }

    let mut missing = operation_descriptor.clone();
    let missing_obj = missing
        .as_object_mut()
        .expect("object operation descriptor");
    missing_obj.remove("operation_id");
    assert_json_rejects(input_schema, &missing, "missing required operation_id");

    let mut unknown_root = operation_descriptor.clone();
    unknown_root
        .as_object_mut()
        .expect("object op")
        .insert("forbidden_root".to_string(), json!("rejected"));
    assert_json_rejects(input_schema, &unknown_root, "unknown root property");

    let mut nested_unknown = operation_descriptor.clone();
    if let Some(port) = nested_unknown["input_ports"][0].as_object_mut() {
        port.insert("forbidden_nested".to_string(), json!(0));
    }
    assert_json_rejects(input_schema, &nested_unknown, "unknown nested property");

    let mut bad_hello = worker_hello.clone();
    bad_hello
        .as_object_mut()
        .expect("hello object")
        .insert("executable_path".to_string(), json!(r"C:\\ProgramData\\chocolatey\\ffprobe.exe"));
    assert_json_rejects(hello_schema, &bad_hello, "hello executable path forbidden");

    let profile_schema_value = read_json(&root.join("Video Localization/contracts/VID-IMPL-P00-003A/records/operation-profile.json"));
    let mut bad_profile = profile_schema_value;
    bad_profile
        .as_object_mut()
        .expect("object profile")
        .remove("implementation_profile_id");
    assert_json_rejects(operation_profile_schema, &bad_profile, "missing profile id");

    let mut malformed = Value::Null;
    if let Some(first_case) = cache_fixtures.bulk_descriptor_positive_cases.first() {
        malformed = first_case["descriptor"].clone();
    }
    malformed
        .as_object_mut()
        .expect("malformed object")
        .insert("descriptor_hash".to_string(), json!(""));
    assert_json_rejects(bulk_schema, &malformed, "bulk descriptor hash required");

    let malformed = serde_json::json!({
        "error_ref": {"namespace":"video.waveform","code":"VID-WAVEFORM-FAIL","version":"1"},
        "schema_id":"VID-IMPL-P00-003A-WAVEFORM-STRUCTURED-ERROR",
        "schema_version":"0.1.0-p00",
        "trace_id":"trace",
        "correlation_id":"corr",
        "subject":"subject",
        "video_disposition":"VID-WAVEFORM-REVIEW",
        "shared_state":"failed",
        "error_category":"manual",
        "retry_class":"manual",
        "operation_stage":"review",
        "safe_recovery":"retry",
        "identity_refs":["job","attempt","dispatch","worker","publication"],
        "details":["x","y","z","w"],
        "applies_to":{"job_ref":"j","attempt_ref":"a","dispatch_ref":"d","worker_ref":"w"}
    });
    assert_json_rejects(structured_schema, &malformed, "structured error with too many details");

    let artifact = serde_json::json!({
        "artifact_id":"artifact-id",
        "artifact_version":"0.1.0-p00",
        "artifact_type":"waveform_artifact",
        "output_schema_id":"VID-IMPL-P00-003A-WAVEFORM-ARTIFACT",
        "completeness":"complete",
        "integrity":{"algorithm":"sha-256","value":"A".repeat(64)},
        "size_bytes":1,
        "component_inventory":["waveform"],
        "producing":{
            "job_ref":"job",
            "attempt_ref":"attempt",
            "dispatch_ref":"dispatch",
            "worker_ref":"worker",
            "operation_id":wc::OPERATION_ID,
            "operation_version":wc::OPERATION_VERSION,
            "implementation_id":wc::IMPLEMENTATION_ID,
            "implementation_profile_id":wc::IMPLEMENTATION_PROFILE_ID
        },
        "input_digests":{
            "staged_copy_hash":"B".repeat(64),
            "manifest_digest":"C".repeat(64),
            "logical_source_ref":"source",
            "staged_copy_hash_schema_ref":"VID-IMPL-P00-002B1-RESOURCE"
        },
        "parameter_digests":{
            "argv":"D".repeat(64),
            "limits":"E".repeat(64),
            "cache_key":"F".repeat(64)
        },
        "resource_digests":{"resource":"A".repeat(64),"schema":"B".repeat(64)},
        "policy_digests":{"policy":"C".repeat(64),"schema":"D".repeat(64)},
        "cache_digests":{"schema_bundle":"E".repeat(64),"control_protocol":"F".repeat(64),"cache_key_schema":"cache-schema-id"},
        "output_port":"waveform_output_port",
        "staging":{"state":"staged-private","path":"C:/tmp/staged","delete_on_stale":true,"quarantine_on_failure":true},
        "lineage":{"job_ref":"job","attempt_ref":"attempt","dispatch_ref":"dispatch","worker_ref":"worker","publication_ref":"publication"},
        "publication_id":"publication",
        "fencing":{"required":true,"active_token":"token"},
        "cache_key":{"schema":"cache-key-schema","digest":"A".repeat(64)},
        "bulk_descriptor_ref":{"reference":"desc-ref","digest":"B".repeat(64),"schema":"waveform-bulk-descriptor"}
    });
    assert_json_valid(artifact_schema, &artifact, "valid artifact contract");
    let mut bad_artifact = artifact;
    bad_artifact
        .as_object_mut()
        .expect("artifact object")
        .insert("completeness".to_string(), json!("partial"));
    assert_json_rejects(artifact_schema, &bad_artifact, "artifact partial completeness");
}

#[test]
fn fixture_cache_key_positive_and_negative_cases_are_executed() {
    let root = workspace_root();
    let contract_fixture_path = root.join("Video Localization/contracts/VID-IMPL-P00-003A/fixtures/cache-key-and-descriptor-fixtures.json");
    let test_fixture_path = root.join("Video Localization/testing/implementation/VID-IMPL-P00-003A/waveform-fixtures.json");
    let contract_fixtures: FixtureBundle = serde_json::from_str(&read_text(&contract_fixture_path)).expect("contract fixture parse");
    let test_fixtures: FixtureBundle = serde_json::from_str(&read_text(&test_fixture_path)).expect("implementation fixture parse");

    assert_eq!(contract_fixtures.cache_key_positive_cases.len(), test_fixtures.cache_key_positive_cases.len());
    assert_eq!(contract_fixtures.cache_key_negative_cases.len(), test_fixtures.cache_key_negative_cases.len());

    for case in &contract_fixtures.cache_key_positive_cases {
        let profile = cache_profile_from_case(case);
        let preimage = wc::cache_key_preimage(&profile).expect("cache preimage");
        let digest = wc::cache_key_digest(&profile).expect("cache digest");
        if let Some(expected_prefix) = case.get("expected_cache_key_prefix").and_then(Value::as_str) {
            assert_eq!(expected_prefix.as_bytes(), wc::CACHE_KEY_PREFIX);
        }
        if let Some(expected_digest) = cache_key_expected_digest(&case) {
            assert_eq!(expected_digest.to_lowercase(), digest);
        }
    }

    for case in &contract_fixtures.cache_key_negative_cases {
        let mut profile = cache_profile_from_case(case);
        let invalid_seed = case.get("invalid_seed").and_then(Value::as_bool).unwrap_or(false);
        let missing_seed = case.get("missing_seed").and_then(Value::as_bool).unwrap_or(false);
        if invalid_seed {
            profile.seed = String::new();
        }
        if missing_seed {
            profile.seed = " ".to_string();
        }
        assert!(wc::cache_key_digest(&profile).is_err());
    }

    assert_eq!(contract_fixtures.cache_key_positive_cases, test_fixtures.cache_key_positive_cases);
    assert_eq!(contract_fixtures.cache_key_negative_cases, test_fixtures.cache_key_negative_cases);
}

#[test]
fn fixture_bulk_descriptor_positive_and_negative_cases_are_executed() {
    let root = workspace_root();
    let contract_fixture_path = root.join("Video Localization/contracts/VID-IMPL-P00-003A/fixtures/cache-key-and-descriptor-fixtures.json");
    let test_fixture_path = root.join("Video Localization/testing/implementation/VID-IMPL-P00-003A/waveform-fixtures.json");
    let contract_fixtures: FixtureBundle = serde_json::from_str(&read_text(&contract_fixture_path)).expect("contract fixture parse");
    let test_fixtures: FixtureBundle = serde_json::from_str(&read_text(&test_fixture_path)).expect("implementation fixture parse");

    assert_eq!(contract_fixtures.bulk_descriptor_positive_cases.len(), test_fixtures.bulk_descriptor_positive_cases.len());
    assert_eq!(contract_fixtures.bulk_descriptor_negative_cases.len(), test_fixtures.bulk_descriptor_negative_cases.len());

    for case in &contract_fixtures.bulk_descriptor_positive_cases {
        let descriptor: wc::WaveformBulkDescriptor =
            serde_json::from_value(case["descriptor"].clone()).expect("descriptor parse");
        let payload = fixture_case_payload(descriptor.length as usize);
        let report = wc::validate_bulk_payload_and_descriptor_linkage(&payload, &descriptor).expect("positive linkage");
        assert_eq!(wc::sha256_hex_of_bytes(&payload), report.payload_digest);
        if let Some(expected_descriptor_digest) = bulk_expected_descriptor_digest(&case) {
            assert_eq!(
                expected_descriptor_digest.to_lowercase(),
                descriptor
                    .descriptor_hash
                    .as_ref()
                    .expect("descriptor hash present")
                    .to_lowercase()
            );
            assert_eq!(expected_descriptor_digest.to_lowercase(), report.descriptor_digest);
        }
    }

    for case in &contract_fixtures.bulk_descriptor_negative_cases {
        let mut descriptor: wc::WaveformBulkDescriptor =
            serde_json::from_value(case["descriptor"].clone()).expect("descriptor parse");
        let reason = case.get("reason").and_then(Value::as_str).unwrap_or("");
        let mut payload = fixture_case_payload(descriptor.length as usize);
        if reason == "missing-descriptor-hash" {
            descriptor.descriptor_hash = None;
        }
        if reason == "chunk-list" {
            descriptor.chunk_hashes_omitted = false;
        }
        if reason == "mismatched-length" {
            payload.truncate((descriptor.length.saturating_sub(1)).try_into().unwrap_or(0));
        }
        assert!(wc::validate_bulk_payload_and_descriptor_linkage(&payload, &descriptor).is_err());
    }

    assert_eq!(contract_fixtures.bulk_descriptor_positive_cases, test_fixtures.bulk_descriptor_positive_cases);
    assert_eq!(contract_fixtures.bulk_descriptor_negative_cases, test_fixtures.bulk_descriptor_negative_cases);
}

#[test]
fn retry_profile_enforces_attempt_ceiling_and_cumulative_limits() {
    let profile = sample_retry_profile();
    let base_usage = wc::WaveformRetryUsage {
        attempts: 1,
        wall_seconds: 1,
        cpu_seconds: 1,
        ram_bytes: 1,
        staging_bytes: 1,
        cost: 1,
    };
    let decision = profile
        .evaluate_retry_decision("VID-WAVEFORM-WORKER-TRANSIENT", &base_usage)
        .expect("allowed first retry");
    assert_eq!(decision.next_attempt, 2);
    assert_eq!(decision.wait_millis, 500);

    let attempt_ceiling = wc::WaveformRetryUsage {
        attempts: 2,
        wall_seconds: 1,
        cpu_seconds: 1,
        ram_bytes: 1,
        staging_bytes: 1,
        cost: 1,
    };
    assert!(profile.evaluate_retry_decision("VID-WAVEFORM-WORKER-TRANSIENT", &attempt_ceiling).is_err());

    assert!(profile.validate_cumulative_usage(&wc::WaveformRetryUsage {
        attempts: 1,
        wall_seconds: profile.cumulative_wall_seconds + 1,
        cpu_seconds: 1,
        ram_bytes: 1,
        staging_bytes: 1,
        cost: 1,
    }).is_err());

    assert!(profile.validate_cumulative_usage(&wc::WaveformRetryUsage {
        attempts: 1,
        wall_seconds: 1,
        cpu_seconds: profile.cumulative_cpu_seconds + 1,
        ram_bytes: 1,
        staging_bytes: 1,
        cost: 1,
    }).is_err());

    assert!(profile.validate_cumulative_usage(&wc::WaveformRetryUsage {
        attempts: 1,
        wall_seconds: 1,
        cpu_seconds: 1,
        ram_bytes: profile.cumulative_ram_bytes + 1,
        staging_bytes: 1,
        cost: 1,
    }).is_err());

    assert!(profile.validate_cumulative_usage(&wc::WaveformRetryUsage {
        attempts: 1,
        wall_seconds: 1,
        cpu_seconds: 1,
        ram_bytes: 1,
        staging_bytes: profile.cumulative_staging_bytes + 1,
        cost: 1,
    }).is_err());

    assert!(profile.validate_cumulative_usage(&wc::WaveformRetryUsage {
        attempts: 1,
        wall_seconds: 1,
        cpu_seconds: 1,
        ram_bytes: 1,
        staging_bytes: 1,
        cost: profile.cost_ceiling + 1,
    }).is_err());

    assert!(profile
        .evaluate_retry_decision("VID-WAVEFORM-REVIEW", &base_usage)
        .is_err());
}

#[test]
fn terminal_reconciliation_map_requires_authoritative_deadline_and_receipt() {
    let (attempt, payload) = sample_publication_attempt();
    let payload_digest = wc::sha256_hex_of_bytes(&payload);
    let whole_object = wc::publication_whole_object_hash(&attempt, &payload_digest)
        .expect("whole object digest");
    let receipt = wc::validate_publication_lifecycle(&attempt, &payload, &whole_object)
        .expect("publication commit receipt");
    assert!(wc::reconcile_terminal_state(Some("VID-WAVEFORM-DEADLINE-EXCEEDED"), Some(10), Some(9), Some(receipt.journal_identity())).is_err());
    let timed_out = wc::reconcile_terminal_state(
        Some("VID-WAVEFORM-DEADLINE-EXCEEDED"),
        Some(10),
        Some(11),
        Some(receipt.journal_identity()),
    )
    .expect("timed out mapping");
    assert_eq!(timed_out, wc::WaveformTerminalState::Terminal("TimedOut".to_string()));
    let timed_out =
        wc::reconcile_terminal_state(
            Some("VID-WAVEFORM-DEADLINE-EXCEEDED"),
            Some(10),
            Some(11),
            None,
        )
        .expect("timed out mapping without receipt");
    assert_eq!(timed_out, wc::WaveformTerminalState::Terminal("TimedOut".to_string()));
    assert!(wc::reconcile_terminal_state(Some("VID-WAVEFORM-DEADLINE-EXCEEDED"), None, None, Some(receipt.journal_identity())).is_err());
    assert!(wc::reconcile_terminal_state(
        Some("VID-WAVEFORM-WORKER-TRANSIENT"),
        Some(10),
        Some(11),
        Some(receipt.journal_identity()),
    )
    .is_err());
    assert!(wc::reconcile_terminal_state(Some("VID-WAVEFORM-DEADLINE-EXCEEDED"), Some(10), None, Some(receipt.journal_identity())).is_err());
    assert!(wc::reconcile_terminal_state(
        Some("VID-WAVEFORM-DOES-NOT-EXIST"),
        Some(10),
        Some(11),
        Some(receipt.journal_identity()),
    ).is_err());
    assert_eq!(
        wc::reconcile_terminal_state(None, Some(100), Some(50), Some(receipt.journal_identity()))
            .expect("receipt success"),
        wc::WaveformTerminalState::Succeeded(receipt.clone())
    );
    assert_eq!(
        wc::reconcile_terminal_state(None, None, None, Some(receipt.journal_identity()))
            .expect("deadline-less success"),
        wc::WaveformTerminalState::Succeeded(receipt.clone())
    );
    assert!(
        wc::reconcile_terminal_state(None, Some(10), Some(20), None).is_err(),
        "expired deadline cannot succeed without authoritative receipt"
    );
    assert!(
        wc::reconcile_terminal_state(None, Some(100), Some(50), Some("e3f4a5b6c7d8e90123456789abcdef0123456789abcdef0123456789abcdef0123"))
            .is_err(),
        "unrecorded journal identity must fail"
    );

    let mut mismatched = receipt.journal_identity().as_bytes().to_vec();
    if let Some(first_byte) = mismatched.first_mut() {
        *first_byte = if *first_byte == b'0' { b'1' } else { b'0' };
    }
    let mismatched_identity = String::from_utf8(mismatched).expect("utf8 bytes");
    assert!(
        wc::reconcile_terminal_state(None, Some(100), Some(50), Some(&mismatched_identity))
            .is_err(),
        "mismatched journal identity must fail"
    );
    let cloned_receipt = receipt.clone();
    assert!(wc::revoke_publication_commit(&receipt));
    assert!(wc::reconcile_terminal_state(None, Some(100), Some(50), Some(cloned_receipt.journal_identity()))
        .is_err(), "removed journal entry must fail");
}

#[test]
fn publication_lifecycle_validates_atomic_commit_and_lineage_controls() {
    let (attempt, payload) = sample_publication_attempt();
    let payload_digest = wc::sha256_hex_of_bytes(&payload);
    let whole_object = wc::publication_whole_object_hash(&attempt, &payload_digest).expect("whole object digest");
    let receipt = wc::validate_publication_lifecycle(&attempt, &payload, &whole_object).expect("publication commit receipt");
    assert_eq!(wc::reconcile_terminal_state(None, Some(100), Some(50), Some(receipt.journal_identity()))
        .expect("succeeded with receipt")
        .clone(),
        wc::WaveformTerminalState::Succeeded(receipt.clone())
    );

    let mut cache_authority = attempt.clone();
    cache_authority.lease.cache_authority = true;
    assert!(wc::validate_publication_lifecycle(&cache_authority, &payload, &whole_object).is_err());

    let mut stale_lease = attempt.clone();
    stale_lease.lease.lease_epoch = stale_lease.lease.minimum_acceptable_lease_epoch.saturating_sub(1);
    assert!(wc::validate_publication_lifecycle(&stale_lease, &payload, &whole_object).is_err());

    let mut bad_fence = attempt.clone();
    bad_fence.lease.observed_fence_token = "bad-token".to_string();
    assert!(wc::validate_publication_lifecycle(&bad_fence, &payload, &whole_object).is_err());

    let mut mismatch_attempt = attempt.clone();
    mismatch_attempt.descriptor.attempt_id = "other-attempt".to_string();
    assert!(wc::validate_publication_lifecycle(&mismatch_attempt, &payload, &whole_object).is_err());

    let mut mismatch_lease = attempt.clone();
    mismatch_lease.descriptor.lease_id = "other-lease".to_string();
    assert!(wc::validate_publication_lifecycle(&mismatch_lease, &payload, &whole_object).is_err());

    let mut malformed_hash = attempt.clone();
    malformed_hash.lease.staged_copy_hash = "not-hex".to_string();
    assert!(wc::validate_publication_lifecycle(&malformed_hash, &payload, &whole_object).is_err());

    let mut staged_hash_mismatch = attempt.clone();
    staged_hash_mismatch.lease.staged_copy_hash = "f".repeat(64);
    let expected = wc::publication_whole_object_hash(&staged_hash_mismatch, &payload_digest)
        .expect("valid staged hash whole object");
    assert!(wc::validate_publication_lifecycle(&staged_hash_mismatch, &payload, &expected).is_err());

    let mut source_mutated = attempt.clone();
    let mut mutated_payload = payload.clone();
    mutated_payload[0] = mutated_payload[0].wrapping_add(1);
    let mutated_payload_digest = wc::sha256_hex_of_bytes(&mutated_payload);
    let expected_for_source_mutation =
        wc::publication_whole_object_hash(&source_mutated, &mutated_payload_digest)
            .expect("source mutation whole object");
    assert!(wc::validate_publication_lifecycle(&source_mutated, &mutated_payload, &expected_for_source_mutation).is_err());

    let mut mutated_payload = payload.clone();
    mutated_payload[0] = mutated_payload[0].wrapping_add(1);
    let mutated_digest = wc::sha256_hex_of_bytes(&mutated_payload);
    let mutated_object = wc::publication_whole_object_hash(&attempt, &mutated_digest).expect("mutated whole object digest");
    assert!(wc::validate_publication_lifecycle(&attempt, &mutated_payload, &mutated_object).is_err());
}
