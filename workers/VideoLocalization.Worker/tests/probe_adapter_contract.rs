#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use videolocalization_worker::{
    to_structured_error,
    validate_probe_output_completeness,
    validate_probe_prelaunch,
    CONTROL_ENVELOPE_BYTES,
    CONTROL_PROTOCOL_VERSION,
    DISPOSITION_MAP,
    ERROR_REF_NAMESPACE,
    ERROR_REF_VERSION,
    IMPLEMENTATION_ID,
    IMPLEMENTATION_PROFILE_ID,
    OPERATION_ID,
    OPERATION_VERSION,
    ProbeContainmentEvidence,
    ProbeLeaseManifest,
    ProbeLaunchPolicy,
    ProbePrelaunchFailure,
    ProbePrelaunchRequest,
    STRUCTURED_ERROR_SCHEMA_ID,
    STRUCTURED_ERROR_SCHEMA_VERSION,
};

#[derive(Deserialize)]
struct ContractSchemaRecord {
    schema_id: String,
    schema_version: String,
    path: String,
    sha256: String,
}

#[derive(Deserialize)]
struct ContractFixtureRecord {
    role: String,
    path: String,
    sha256: String,
}

#[derive(Deserialize)]
struct ContractSchemaManifest {
    artifact_id: String,
    artifact_version: String,
    operation_id: String,
    operation_version: String,
    implementation_profile_id: String,
    implementation_id: String,
    #[serde(default)]
    fixture_records: Vec<ContractFixtureRecord>,
    schema_records: Vec<ContractSchemaRecord>,
}

#[derive(Deserialize)]
struct OperationDescriptorFixture {
    operation_id: String,
    operation_version: String,
    implementation_id: String,
    implementation_profile_id: String,
    input_schema_id: String,
    normalized_observation_schema_id: String,
    parameter_schema_id: String,
    structured_error_schema_id: String,
    resource_schema_id: String,
    policy_schema_id: String,
    output_contract_schema_id: String,
    supported_capabilities: Vec<String>,
    unsupported_capabilities: Vec<String>,
    transport: String,
}

#[derive(Deserialize)]
struct LimitsFixture {
    control_envelope_bytes: u64,
    wall_time_seconds: u64,
    child_cpu_seconds: u64,
    resident_memory_bytes: u64,
    stream_count: u32,
}

#[derive(Deserialize)]
struct WorkerHelloFixture {
    operation_id: String,
    operation_version: String,
    implementation_id: String,
    implementation_profile_id: String,
    executable_path: String,
    executable_sha256: String,
    component_version: String,
    transport: String,
    platform: String,
    architecture: String,
    control_protocol_min: String,
    control_protocol_max: String,
    limits: LimitsFixture,
    supported: Vec<String>,
    unsupported: Vec<String>,
}

#[derive(Deserialize)]
struct DispositionFixture {
    video_disposition: String,
    shared_terminal_state: String,
    error_category: String,
    retry_class: String,
    operation_stage: String,
    safe_recovery: String,
    identity_refs: Vec<String>,
}

#[derive(Deserialize)]
struct PublicationFixture {
    artifact_id: String,
    artifact_version: String,
    artifact_type: String,
    output_schema_id: String,
    completeness: String,
    integrity: IntegrityFixture,
    size_bytes: u64,
    component_inventory: Vec<String>,
    producing: ProducingFixture,
    input_digests: PublicationInputDigestFixture,
    parameter_digests: PublicationParameterDigestFixture,
    resource_digests: PublicationResourceDigestFixture,
    policy_digests: PublicationPolicyDigestFixture,
    cache_digests: PublicationCacheDigestFixture,
    output_port: String,
    staging: PublicationStagingFixture,
    lineage: PublicationLineageFixture,
    publication_id: String,
    fencing: PublicationFencingFixture,
}

#[derive(Deserialize)]
struct IntegrityFixture {
    algorithm: String,
    value: String,
}

#[derive(Deserialize)]
struct ProducingFixture {
    job_ref: String,
    attempt_ref: String,
    dispatch_ref: String,
    worker_ref: String,
    operation_id: String,
    operation_version: String,
    implementation_id: String,
    implementation_profile_id: String,
}

#[derive(Deserialize)]
struct PublicationInputDigestFixture {
    staged_copy_hash: String,
    manifest_digest: String,
    logical_source_ref: String,
}

#[derive(Deserialize)]
struct PublicationParameterDigestFixture {
    argv: String,
    limits: String,
}

#[derive(Deserialize)]
struct PublicationResourceDigestFixture {
    resource: String,
    schema: String,
}

#[derive(Deserialize)]
struct PublicationPolicyDigestFixture {
    policy: String,
    schema: String,
}

#[derive(Deserialize)]
struct PublicationCacheDigestFixture {
    schema_bundle: String,
    control_protocol: String,
}

#[derive(Deserialize)]
struct PublicationStagingFixture {
    state: String,
    path: String,
    delete_on_stale: bool,
    quarantine_on_failure: bool,
}

#[derive(Deserialize)]
struct PublicationLineageFixture {
    job_ref: String,
    attempt_ref: String,
    dispatch_ref: String,
    worker_ref: String,
    publication_ref: String,
}

#[derive(Deserialize)]
struct PublicationFencingFixture {
    required: bool,
    active_token: String,
}

#[derive(Deserialize)]
struct LaunchPolicyFixture {
    use_shell: bool,
    use_path_lookup: bool,
    user_supplied_options: bool,
}

#[derive(Deserialize)]
struct EvidenceFixture {
    network_denied: bool,
    watchdog_enabled: bool,
    job_object_enabled: bool,
}

#[derive(Deserialize)]
struct LeaseFixture {
    lease_id: String,
    lease_epoch: u64,
    minimum_acceptable_lease_epoch: u64,
    immutable: bool,
    source_sha256: String,
    source_length: u64,
    staged_copy_identity: String,
    staged_copy_sha256: String,
    staged_copy_length: u64,
    observed_staged_copy_identity: String,
    observed_staged_copy_sha256: String,
    observed_staged_copy_length: u64,
    access_scope: String,
    expires_at: String,
    manifest_digest: String,
    logical_source_ref: String,
    observed_fence_token: String,
    expected_fence_token: String,
}

#[derive(Deserialize)]
struct PrelaunchRequestFixture {
    id: String,
    operation_id: String,
    operation_version: String,
    implementation_id: String,
    implementation_profile_id: String,
    executable_path: String,
    executable_sha256: String,
    staged_input_path: String,
    launch_argv: Vec<String>,
    launch_policy: LaunchPolicyFixture,
    capabilities: Vec<String>,
    transport: String,
    output_envelope_bytes: u64,
    evidence: EvidenceFixture,
    lease: LeaseFixture,
    worker_instance_id: String,
    dispatch_id: String,
    control_protocol_version: String,
    job_id: String,
    attempt_id: String,
    trace_id: String,
    correlation_id: String,
    #[serde(default)]
    error_instance_id: Option<String>,
    schema_digest_overrides: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
struct PositiveFixtureBundle {
    id: String,
    request: Option<PrelaunchRequestFixture>,
}

#[derive(Deserialize)]
struct NegativeFixtureBundle {
    id: String,
    expected_failure: Option<String>,
    request: Option<PrelaunchRequestFixture>,
}

#[derive(Deserialize)]
struct PrelaunchFixtureBundle {
    positive_fixtures: Vec<PositiveFixtureBundle>,
    negative_fixtures: Vec<NegativeFixtureBundle>,
}

#[test]
fn contract_manifest_hashes_match_files() {
    let workspace_root = workspace_root();
    let manifest_path = workspace_root.join("contracts/VID-IMPL-P00-002B1/schema-bundle.json");
    let manifest: ContractSchemaManifest =
        serde_json::from_str(&read_text(&manifest_path)).expect("manifest JSON parse");

    assert_eq!(manifest.artifact_id, "VID-IMPL-P00-002B1");
    assert_eq!(manifest.artifact_version, "1.0.0-p00");
    assert_eq!(manifest.operation_id, OPERATION_ID);
    assert_eq!(manifest.operation_version, OPERATION_VERSION);
    assert_eq!(manifest.implementation_profile_id, IMPLEMENTATION_PROFILE_ID);
    assert_eq!(manifest.implementation_id, IMPLEMENTATION_ID);
    assert!(!manifest.schema_records.is_empty());

    for schema in manifest.schema_records {
        let schema_path = workspace_root.join(schema.path);
        let observed = sha256_hex_of_file(&schema_path);
        assert_eq!(
            observed.to_ascii_lowercase(),
            schema.sha256.to_ascii_lowercase(),
            "schema hash mismatch for {}",
            schema.schema_id
        );
    }

    assert_eq!(manifest.fixture_records.len(), 4);
    for fixture in manifest.fixture_records {
        let fixture_path = workspace_root.join(fixture.path);
        let observed = sha256_hex_of_file(&fixture_path);
        assert_eq!(
            observed.to_ascii_lowercase(),
            fixture.sha256.to_ascii_lowercase(),
            "fixture hash mismatch for {}",
            fixture.role
        );
    }
}

#[test]
fn operation_descriptor_and_worker_hello_are_bound_and_exact() {
    let workspace_root = workspace_root();
    let manifest_path = workspace_root.join("contracts/VID-IMPL-P00-002B1/schema-bundle.json");
    let manifest: ContractSchemaManifest =
        serde_json::from_str(&read_text(&manifest_path)).expect("manifest JSON parse");
    let operation: OperationDescriptorFixture =
        serde_json::from_str(&read_text(&workspace_root.join(
            "contracts/VID-IMPL-P00-002B1/fixtures/operation-descriptor.json",
        )))
        .expect("operation descriptor fixture parse");
    let hello: WorkerHelloFixture = serde_json::from_str(&read_text(&workspace_root.join(
        "contracts/VID-IMPL-P00-002B1/fixtures/worker-hello.json",
    )))
    .expect("worker hello fixture parse");

    let schema_ids: Vec<String> = manifest
        .schema_records
        .iter()
        .map(|record| record.schema_id.clone())
        .collect();
    assert!(schema_ids.contains(&operation.input_schema_id));
    assert!(schema_ids.contains(&operation.normalized_observation_schema_id));
    assert!(schema_ids.contains(&operation.parameter_schema_id));
    assert!(schema_ids.contains(&operation.structured_error_schema_id));
    assert!(schema_ids.contains(&operation.resource_schema_id));
    assert!(schema_ids.contains(&operation.policy_schema_id));
    assert!(schema_ids.contains(&operation.output_contract_schema_id));

    assert_eq!(operation.operation_id, OPERATION_ID);
    assert_eq!(operation.operation_version, OPERATION_VERSION);
    assert_eq!(operation.implementation_id, IMPLEMENTATION_ID);
    assert_eq!(operation.implementation_profile_id, IMPLEMENTATION_PROFILE_ID);
    assert_eq!(operation.transport, "local-staged-file");

    assert_eq!(hello.operation_id, OPERATION_ID);
    assert_eq!(hello.operation_version, OPERATION_VERSION);
    assert_eq!(hello.implementation_id, IMPLEMENTATION_ID);
    assert_eq!(hello.implementation_profile_id, IMPLEMENTATION_PROFILE_ID);
    assert_eq!(hello.platform, "Windows");
    assert_eq!(hello.architecture, "x64");
    assert_eq!(hello.transport, "local-staged-file");
    assert_eq!(hello.supported, vec!["cancellation", "deadline"]);
    assert_eq!(
        hello.unsupported,
        vec![
            "progress",
            "checkpoint",
            "pause-resume",
            "partial-success",
            "degraded-output",
            "acceleration",
            "auto-fallback",
            "network"
        ]
    );
    assert_eq!(hello.limits.control_envelope_bytes, CONTROL_ENVELOPE_BYTES);
    assert!(hello.limits.stream_count > 0);
    assert_eq!(hello.control_protocol_min, CONTROL_PROTOCOL_VERSION);
    assert_eq!(hello.control_protocol_max, CONTROL_PROTOCOL_VERSION);
    assert!(hello.control_protocol_min <= hello.control_protocol_max);
}

#[test]
fn disposition_matrix_covers_all_video_outcomes() {
    let workspace_root = workspace_root();
    let mappings: Vec<DispositionFixture> = serde_json::from_str(&read_text(
        &workspace_root.join("contracts/VID-IMPL-P00-002B1/fixtures/disposition-mapping.json"),
    ))
    .expect("disposition mapping parse");

    let required = [
        "VID-INGEST-UNSUPPORTED",
        "VID-INGEST-MALFORMED",
        "VID-INGEST-LIMIT",
        "VID-INGEST-PROTECTED",
        "VID-INGEST-QUARANTINED",
        "VID-INGEST-CANCELLED",
        "VID-INGEST-WORKER-FAILED",
        "VID-INGEST-IDENTITY-MISMATCH",
        "VID-INGEST-REVIEW",
    ];

    let mapped: Vec<String> = mappings.iter().map(|row| row.video_disposition.clone()).collect();
    for expected in required {
        assert!(mapped.contains(&expected.to_string()), "missing disposition {}", expected);
    }

    assert_eq!(mappings.len(), DISPOSITION_MAP.len());

    for map in &mappings {
        let fixture_state = DISPOSITION_MAP
            .iter()
            .find(|entry| entry.video_disposition == map.video_disposition)
            .expect("fixture disposition in code map");

        assert_eq!(fixture_state.shared_terminal_state, map.shared_terminal_state);
        assert!(!map.error_category.is_empty());
        assert!(!map.retry_class.is_empty());
        assert!(!map.operation_stage.is_empty());
        assert!(!map.safe_recovery.is_empty());
        assert!(!map.identity_refs.is_empty());
    }
}

#[test]
fn publication_fixture_models_quarantine_and_fence_behavior() {
    let workspace_root = workspace_root();
    let publication: PublicationFixture = serde_json::from_str(&read_text(
        &workspace_root.join("contracts/VID-IMPL-P00-002B1/fixtures/publication-fixture.json"),
    ))
    .expect("publication fixture parse");

    assert_eq!(publication.artifact_id, "artifact-vid-impl-002b1-probe-observation-v1");
    assert_eq!(publication.artifact_version, "1.0.0-p00");
    assert_eq!(publication.artifact_type, "probe_observation");
    assert_eq!(publication.completeness, "complete");
    assert_eq!(publication.integrity.algorithm, "sha-256");
    assert_eq!(publication.size_bytes, 2048);
    assert_eq!(publication.output_port, "probe-observation-output-port");
    assert_eq!(publication.staging.state, "staged-private");
    assert!(publication.staging.delete_on_stale);
    assert!(publication.staging.quarantine_on_failure);
    assert_eq!(publication.lineage.publication_ref, publication.publication_id);
    assert!(publication.fencing.required);
    assert!(!publication.fencing.active_token.is_empty());
    assert!(!publication.producing.operation_id.is_empty());
    assert!(!publication.producing.operation_version.is_empty());
    assert!(!publication.producing.implementation_id.is_empty());
    assert!(!publication.producing.implementation_profile_id.is_empty());
    assert!(!publication.staging.path.is_empty());
    assert!(!publication.integrity.value.is_empty());
    assert_eq!(publication.output_schema_id, "VID-IMPL-P00-002B1-NORMALIZED-OBSERVATION");
}

#[test]
fn prelaunch_positive_and_negative_fixtures_are_validated() {
    let workspace_root = workspace_root();
    let schema_manifest = read_contract_manifest(&workspace_root);
    let expected_digests = schema_digests(&schema_manifest);
    let prelaunch_fixtures_path =
        workspace_root.join("testing/implementation/VID-IMPL-P00-002B1/prelaunch-fixtures.json");
    let fixtures: PrelaunchFixtureBundle = serde_json::from_str(&read_text(&prelaunch_fixtures_path))
        .expect("prelaunch fixture parse");

    for fixture in fixtures.positive_fixtures {
        if let Some(request) = fixture.request {
            let request = fixture_to_request(request, &expected_digests);
            assert!(
                validate_probe_prelaunch(&request, &expected_digests).is_ok(),
                "positive fixture failed"
            );
        }
    }

    for fixture in fixtures.negative_fixtures {
        if let (Some(expected_failure), Some(request)) = (fixture.expected_failure, fixture.request) {
            let request = fixture_to_request(request, &expected_digests);
            let failure = validate_probe_prelaunch(&request, &expected_digests).err();
            assert!(failure.is_some(), "negative fixture should fail");
            let failure_code = failure.expect("failure").code().to_string();
            assert_eq!(failure_code, expected_failure);
        }
    }
}

#[test]
fn prelaunch_failures_map_to_complete_structured_error() {
    let workspace_root = workspace_root();
    let schema_manifest = read_contract_manifest(&workspace_root);
    let expected_digests = schema_digests(&schema_manifest);
    let prelaunch_fixtures_path =
        workspace_root.join("testing/implementation/VID-IMPL-P00-002B1/prelaunch-fixtures.json");
    let fixtures: PrelaunchFixtureBundle = serde_json::from_str(&read_text(&prelaunch_fixtures_path))
        .expect("prelaunch fixture parse");

    for fixture in fixtures.negative_fixtures {
        if let (Some(expected_failure), Some(request)) = (fixture.expected_failure, fixture.request) {
            let request = fixture_to_request(request, &expected_digests);
            let failure = match validate_probe_prelaunch(&request, &expected_digests) {
                Ok(_) => panic!(
                    "negative fixture {} was unexpectedly accepted",
                    fixture.id
                ),
                Err(failure) => failure,
            };
            assert_eq!(failure.code(), expected_failure);

            let structured = to_structured_error(&failure, &request);
            let observed_disposition = DISPOSITION_MAP
                .iter()
                .find(|mapping| mapping.video_disposition == structured.video_disposition)
                .expect("fixture disposition exists");

            assert_eq!(structured.schema_id, STRUCTURED_ERROR_SCHEMA_ID);
            assert_eq!(structured.schema_version, STRUCTURED_ERROR_SCHEMA_VERSION);
            assert_eq!(structured.error_ref.namespace, ERROR_REF_NAMESPACE);
            assert_eq!(structured.error_ref.version, ERROR_REF_VERSION);
            assert_eq!(structured.error_ref.code, expected_failure);
            assert_eq!(structured.error_ref.error_instance, request.error_instance_id);
            assert_eq!(structured.trace_id, request.trace_id);
            assert_eq!(structured.applies_to.job_ref, request.job_id);
            assert_eq!(structured.applies_to.attempt_ref, request.attempt_id);
            assert_eq!(structured.applies_to.dispatch_ref, request.dispatch_id);
            assert_eq!(structured.applies_to.worker_ref, request.worker_instance_id);
            assert_eq!(structured.applies_to.publication_ref, None);
            assert_eq!(structured.video_disposition, observed_disposition.video_disposition);
            assert_eq!(structured.error_category, observed_disposition.error_category);
            assert!(!structured.shared_state.is_empty());
            assert!(!structured.error_category.is_empty());
            assert!(!structured.retry_class.is_empty());
            assert!(!structured.operation_stage.is_empty());
            assert!(!structured.safe_recovery.is_empty());
            assert!(!structured.identity_refs.is_empty());
            assert!(structured.identity_refs.len() <= 5);
            assert!(structured.details.len() <= 3);
            assert!(structured.causes.len() <= 3);
        }
    }
}

#[test]
fn publication_completeness_rejects_partial_values() {
    let _workspace_root = workspace_root();
    assert!(validate_probe_output_completeness("complete").is_ok());

    let fixture = PublicationFixture {
        artifact_id: "artifact-vid-impl-002b1-probe-observation-v1".to_string(),
        artifact_version: "1.0.0-p00".to_string(),
        artifact_type: "probe_observation".to_string(),
        output_schema_id: "VID-IMPL-P00-002B1-NORMALIZED-OBSERVATION".to_string(),
        completeness: "partial".to_string(),
        integrity: IntegrityFixture {
            algorithm: "sha-256".to_string(),
            value: "00".repeat(32),
        },
        size_bytes: 2048,
        component_inventory: vec!["probe-adapter".to_string()],
        producing: ProducingFixture {
            job_ref: "job-id".to_string(),
            attempt_ref: "attempt-id".to_string(),
            dispatch_ref: "dispatch-id".to_string(),
            worker_ref: "worker-id".to_string(),
            operation_id: OPERATION_ID.to_string(),
            operation_version: OPERATION_VERSION.to_string(),
            implementation_id: IMPLEMENTATION_ID.to_string(),
            implementation_profile_id: IMPLEMENTATION_PROFILE_ID.to_string(),
        },
        input_digests: PublicationInputDigestFixture {
            staged_copy_hash: "00".repeat(32),
            manifest_digest: "00".repeat(32),
            logical_source_ref: "src:fixture".to_string(),
        },
        parameter_digests: PublicationParameterDigestFixture {
            argv: "00".repeat(32),
            limits: "00".repeat(32),
        },
        resource_digests: PublicationResourceDigestFixture {
            resource: "00".repeat(32),
            schema: "00".repeat(32),
        },
        policy_digests: PublicationPolicyDigestFixture {
            policy: "00".repeat(32),
            schema: "00".repeat(32),
        },
        cache_digests: PublicationCacheDigestFixture {
            schema_bundle: "00".repeat(32),
            control_protocol: "00".repeat(32),
        },
        output_port: "probe-observation-output-port".to_string(),
        staging: PublicationStagingFixture {
            state: "staged-private".to_string(),
            path: "C:/temp/staging".to_string(),
            delete_on_stale: true,
            quarantine_on_failure: true,
        },
        lineage: PublicationLineageFixture {
            job_ref: "job-id".to_string(),
            attempt_ref: "attempt-id".to_string(),
            dispatch_ref: "dispatch-id".to_string(),
            worker_ref: "worker-id".to_string(),
            publication_ref: "publication-id".to_string(),
        },
        publication_id: "publication-id".to_string(),
        fencing: PublicationFencingFixture {
            required: true,
            active_token: "token".to_string(),
        },
    };

    assert!(
        validate_probe_output_completeness(&fixture.completeness).is_err(),
        "partial completeness should be rejected"
    );
    assert_eq!(
        validate_probe_output_completeness("partial").err(),
        Some(ProbePrelaunchFailure::PartialCompletenessRejected)
    );
}

fn fixture_to_request(
    request: PrelaunchRequestFixture,
    expected_digests: &HashMap<String, String>,
) -> ProbePrelaunchRequest {
    let mut digests = expected_digests.clone();
    if let Some(overrides) = request.schema_digest_overrides {
        for (key, value) in overrides {
            digests.insert(key, value);
        }
    }

    ProbePrelaunchRequest {
        operation_id: request.operation_id,
        operation_version: request.operation_version,
        implementation_id: request.implementation_id,
        implementation_profile_id: request.implementation_profile_id,
        executable_path: request.executable_path,
        executable_sha256: request.executable_sha256,
        staged_input_path: request.staged_input_path.clone(),
        launch_argv: request.launch_argv,
        launch_policy: ProbeLaunchPolicy {
            use_shell: request.launch_policy.use_shell,
            use_path_lookup: request.launch_policy.use_path_lookup,
            user_supplied_options: request.launch_policy.user_supplied_options,
        },
        schema_digests: digests,
        capabilities: request.capabilities,
        transport: request.transport,
        evidence: ProbeContainmentEvidence {
            network_denied: request.evidence.network_denied,
            watchdog_enabled: request.evidence.watchdog_enabled,
            job_object_enabled: request.evidence.job_object_enabled,
        },
        output_envelope_bytes: request.output_envelope_bytes,
        worker_instance_id: request.worker_instance_id,
        dispatch_id: request.dispatch_id,
        control_protocol_version: request.control_protocol_version,
        job_id: request.job_id,
        attempt_id: request.attempt_id,
        trace_id: request.trace_id,
        correlation_id: request.correlation_id,
        error_instance_id: request
            .error_instance_id
            .unwrap_or_else(|| format!("err-inst-{}", request.id)),
        lease: ProbeLeaseManifest {
            lease_id: request.lease.lease_id,
            lease_epoch: request.lease.lease_epoch,
            minimum_acceptable_lease_epoch: request.lease.minimum_acceptable_lease_epoch,
            immutable: request.lease.immutable,
            source_sha256: request.lease.source_sha256,
            source_length: request.lease.source_length,
            staged_copy_identity: request.lease.staged_copy_identity,
            staged_copy_sha256: request.lease.staged_copy_sha256,
            staged_copy_length: request.lease.staged_copy_length,
            observed_staged_copy_identity: request.lease.observed_staged_copy_identity,
            observed_staged_copy_sha256: request.lease.observed_staged_copy_sha256,
            observed_staged_copy_length: request.lease.observed_staged_copy_length,
            access_scope: request.lease.access_scope,
            expires_at: request.lease.expires_at,
            manifest_digest: request.lease.manifest_digest,
            logical_source_ref: request.lease.logical_source_ref,
            observed_fence_token: request.lease.observed_fence_token,
            expected_fence_token: request.lease.expected_fence_token,
        },
    }
}

fn read_contract_manifest(root: &Path) -> ContractSchemaManifest {
    let manifest_path = root.join("contracts/VID-IMPL-P00-002B1/schema-bundle.json");
    serde_json::from_str(&read_text(&manifest_path)).expect("manifest parse")
}

fn schema_digests(manifest: &ContractSchemaManifest) -> HashMap<String, String> {
    let mut digests = HashMap::new();
    for record in &manifest.schema_records {
        digests.insert(record.schema_id.clone(), record.sha256.clone());
    }
    digests
}

fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("..").join("..")
}

fn read_text(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn sha256_hex_of_file(path: &Path) -> String {
    let mut hasher = Sha256::new();
    let bytes = fs::read(path).expect("schema file should exist");
    hasher.update(&bytes);
    hasher
        .finalize()
        .iter()
        .map(|value| format!("{:02x}", value))
        .collect::<String>()
}
