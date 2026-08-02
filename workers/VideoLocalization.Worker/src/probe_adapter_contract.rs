use std::collections::HashMap;
use std::path::Component;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const OPERATION_ID: &str = "submark.video-localization.media.probe";
pub const OPERATION_VERSION: &str = "1.0.0-p00";
pub const IMPLEMENTATION_ID: &str = "VID-IMPL-P00-002B1";
pub const IMPLEMENTATION_PROFILE_ID: &str = "VID-PROBE-P00-001";
pub const CONTROL_PROTOCOL_VERSION: &str = "1.0.0";
pub const REGISTERED_EXECUTABLE_PATH: &str =
    r"C:\ProgramData\chocolatey\lib\ffmpeg-full\tools\ffmpeg\bin\ffprobe.exe";
pub const REGISTERED_EXECUTABLE_SHA256: &str =
    "9df3b0b5275e830961df6d94e1f7a71121a7abd5ff708e9fec8a0b6084a55015";
pub const IMPLEMENTED_PROFILE_VERSION: &str = "1.0.0-p00";
pub const STRUCTURED_ERROR_SCHEMA_ID: &str = "VID-IMPL-P00-002B1-STRUCTURED-ERROR";
pub const STRUCTURED_ERROR_SCHEMA_VERSION: &str = "1.0.0-p00";
pub const ERROR_REF_NAMESPACE: &str = "video-localization.shared";
pub const ERROR_REF_VERSION: &str = "1.0.0-p00";
pub const OUTPUT_ENVELOPE_BYTES: u64 = 1_048_576;
pub const CONTROL_ENVELOPE_BYTES: u64 = OUTPUT_ENVELOPE_BYTES;
pub const MAX_SOURCE_STREAMS: u32 = 256;
pub const EXPECTED_SCHEMA_REFERENCE_IDS: [&str; 7] = [
    "VID-IMPL-P00-002B1-INPUT",
    "VID-IMPL-P00-002B1-NORMALIZED-OBSERVATION",
    "VID-IMPL-P00-002B1-PARAMETER",
    "VID-IMPL-P00-002B1-STRUCTURED-ERROR",
    "VID-IMPL-P00-002B1-RESOURCE",
    "VID-IMPL-P00-002B1-POLICY",
    "VID-IMPL-P00-002B1-OUTPUT-CONTRACT",
];

pub const PROBE_ARGV_PREFIX: [&str; 12] = [
    "-v",
    "error",
    "-hide_banner",
    "-protocol_whitelist",
    "file",
    "-format_whitelist",
    "matroska",
    "-show_format",
    "-show_streams",
    "-show_chapters",
    "-of",
    "json",
];

pub const SUPPORTED_CAPABILITIES: [&str; 2] = ["cancellation", "deadline"];
pub const UNSUPPORTED_CAPABILITIES: [&str; 8] = [
    "progress",
    "checkpoint",
    "pause-resume",
    "partial-success",
    "acceleration",
    "degraded-output",
    "auto-fallback",
    "network",
];

pub const CONTROL_LIMITS: ControlEnvelopeLimit = ControlEnvelopeLimit {
    wall_clock_seconds: 60,
    child_cpu_seconds: 60,
    resident_memory_bytes: 1_073_741_824,
    control_envelope_bytes: OUTPUT_ENVELOPE_BYTES,
    stream_count: MAX_SOURCE_STREAMS,
};

#[derive(Debug, Clone)]
pub struct ControlEnvelopeLimit {
    pub wall_clock_seconds: u64,
    pub child_cpu_seconds: u64,
    pub resident_memory_bytes: u64,
    pub control_envelope_bytes: u64,
    pub stream_count: u32,
}

#[derive(Debug, Clone)]
pub struct ProbeLaunchPolicy {
    pub use_shell: bool,
    pub use_path_lookup: bool,
    pub user_supplied_options: bool,
}

#[derive(Debug, Clone)]
pub struct ProbeContainmentEvidence {
    pub network_denied: bool,
    pub watchdog_enabled: bool,
    pub job_object_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ProbeLeaseManifest {
    pub lease_id: String,
    pub lease_epoch: u64,
    pub minimum_acceptable_lease_epoch: u64,
    pub immutable: bool,
    pub source_sha256: String,
    pub source_length: u64,
    pub staged_copy_identity: String,
    pub staged_copy_sha256: String,
    pub staged_copy_length: u64,
    pub observed_staged_copy_identity: String,
    pub observed_staged_copy_sha256: String,
    pub observed_staged_copy_length: u64,
    pub access_scope: String,
    pub expires_at: String,
    pub manifest_digest: String,
    pub logical_source_ref: String,
    pub observed_fence_token: String,
    pub expected_fence_token: String,
}

#[derive(Debug, Clone)]
pub struct ProbePrelaunchRequest {
    pub operation_id: String,
    pub operation_version: String,
    pub implementation_id: String,
    pub implementation_profile_id: String,
    pub executable_path: String,
    pub executable_sha256: String,
    pub staged_input_path: String,
    pub launch_argv: Vec<String>,
    pub launch_policy: ProbeLaunchPolicy,
    pub schema_digests: HashMap<String, String>,
    pub capabilities: Vec<String>,
    pub transport: String,
    pub evidence: ProbeContainmentEvidence,
    pub output_envelope_bytes: u64,
    pub worker_instance_id: String,
    pub dispatch_id: String,
    pub control_protocol_version: String,
    pub job_id: String,
    pub attempt_id: String,
    pub trace_id: String,
    pub correlation_id: String,
    pub error_instance_id: String,
    pub lease: ProbeLeaseManifest,
}

#[derive(Debug, Clone)]
pub struct ErrorRef {
    pub namespace: &'static str,
    pub code: String,
    pub version: &'static str,
    pub error_instance: String,
}

#[derive(Debug, Clone)]
pub struct StructuredErrorAppliesTo {
    pub job_ref: String,
    pub attempt_ref: String,
    pub dispatch_ref: String,
    pub worker_ref: String,
    pub publication_ref: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StructuredError {
    pub schema_id: &'static str,
    pub schema_version: &'static str,
    pub error_ref: ErrorRef,
    pub trace_id: String,
    pub correlation_id: String,
    pub subject: String,
    pub video_disposition: String,
    pub shared_state: String,
    pub error_category: String,
    pub retry_class: String,
    pub operation_stage: String,
    pub safe_recovery: String,
    pub identity_refs: Vec<String>,
    pub details: Vec<String>,
    pub causes: Vec<String>,
    pub applies_to: StructuredErrorAppliesTo,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProbePrelaunchFailure {
    PathMissing,
    PathInvalid,
    ComponentMismatch,
    OperationIdentityMismatch,
    OperationVersionMismatch,
    ImplementationMismatch,
    ImplementationProfileMismatch,
    WorkerInstanceMismatch,
    DispatchMismatch,
    ControlProtocolMismatch,
    SchemaReferenceMismatch,
    InvalidLaunchPolicy,
    UnsupportedCapability(String),
    MissingRequiredCapability(&'static str),
    SchemaDigestMismatch(String),
    MutableLease,
    StaleLeaseOrFence,
    SourceIdentityMismatch,
    SourceHashMismatch,
    SourceLengthMismatch,
    ExpiredLease,
    ScopeMismatch,
    EnvelopeExceeded,
    PartialCompletenessRejected,
    MissingContainmentEvidence(&'static str),
    TransportUnsupported,
    ArgumentMismatch,
    OutOfPolicyIdentity,
}

impl ProbePrelaunchFailure {
    pub fn code(&self) -> &'static str {
        match self {
            Self::PathMissing => "PathMissing",
            Self::PathInvalid => "PathInvalid",
            Self::ComponentMismatch => "ComponentMismatch",
            Self::OperationIdentityMismatch => "OperationIdentityMismatch",
            Self::OperationVersionMismatch => "OperationVersionMismatch",
            Self::ImplementationMismatch => "ImplementationMismatch",
            Self::ImplementationProfileMismatch => "ImplementationProfileMismatch",
            Self::WorkerInstanceMismatch => "WorkerInstanceMismatch",
            Self::DispatchMismatch => "DispatchMismatch",
            Self::ControlProtocolMismatch => "ControlProtocolMismatch",
            Self::SchemaReferenceMismatch => "SchemaReferenceMismatch",
            Self::InvalidLaunchPolicy => "InvalidLaunchPolicy",
            Self::UnsupportedCapability(_) => "UnsupportedCapability",
            Self::MissingRequiredCapability(_) => "MissingRequiredCapability",
            Self::SchemaDigestMismatch(_) => "SchemaDigestMismatch",
            Self::MutableLease => "MutableLease",
            Self::StaleLeaseOrFence => "StaleLeaseOrFence",
            Self::SourceIdentityMismatch => "SourceIdentityMismatch",
            Self::SourceHashMismatch => "SourceHashMismatch",
            Self::SourceLengthMismatch => "SourceLengthMismatch",
            Self::ExpiredLease => "ExpiredLease",
            Self::ScopeMismatch => "ScopeMismatch",
            Self::EnvelopeExceeded => "EnvelopeExceeded",
            Self::PartialCompletenessRejected => "PartialCompletenessRejected",
            Self::MissingContainmentEvidence(_) => "MissingContainmentEvidence",
            Self::OutOfPolicyIdentity => "OutOfPolicyIdentity",
            Self::TransportUnsupported => "TransportUnsupported",
            Self::ArgumentMismatch => "ArgumentMismatch",
        }
    }
}

#[derive(Debug)]
pub struct DispositionMapping {
    pub video_disposition: &'static str,
    pub shared_terminal_state: &'static str,
    pub error_category: &'static str,
    pub retry_class: &'static str,
    pub operation_stage: &'static str,
    pub safe_recovery: &'static str,
    pub identity_refs: &'static [&'static str],
}

pub const DISPOSITION_MAP: &[DispositionMapping] = &[
    DispositionMapping {
        video_disposition: "VID-INGEST-UNSUPPORTED",
        shared_terminal_state: "Failed",
        error_category: "Unsupported",
        retry_class: "non-retryable",
        operation_stage: "validation",
        safe_recovery: "select a promoted profile or convert externally",
        identity_refs: &["job_id", "attempt_id", "dispatch_id", "worker_id"],
    },
    DispositionMapping {
        video_disposition: "VID-INGEST-MALFORMED",
        shared_terminal_state: "Failed",
        error_category: "MalformedInput",
        retry_class: "non-retryable",
        operation_stage: "validation",
        safe_recovery: "inspect details or use another source",
        identity_refs: &["job_id", "attempt_id", "dispatch_id", "worker_id", "publication_id"],
    },
    DispositionMapping {
        video_disposition: "VID-INGEST-LIMIT",
        shared_terminal_state: "TimedOut",
        error_category: "ResourceLimit",
        retry_class: "non-retryable",
        operation_stage: "containment",
        safe_recovery: "use a reviewed lower-cost operation profile",
        identity_refs: &["attempt_id", "dispatch_id", "worker_id", "publication_id"],
    },
    DispositionMapping {
        video_disposition: "VID-INGEST-PROTECTED",
        shared_terminal_state: "Failed",
        error_category: "ProtectionOrRights",
        retry_class: "non-retryable",
        operation_stage: "validation",
        safe_recovery: "provide a lawful unprotected source",
        identity_refs: &["job_id", "attempt_id", "dispatch_id", "worker_id"],
    },
    DispositionMapping {
        video_disposition: "VID-INGEST-QUARANTINED",
        shared_terminal_state: "Failed",
        error_category: "UnsafeOutput",
        retry_class: "non-retryable",
        operation_stage: "output-validation",
        safe_recovery: "review evidence; never open automatically",
        identity_refs: &["job_id", "attempt_id", "dispatch_id", "publication_id", "worker_id"],
    },
    DispositionMapping {
        video_disposition: "VID-INGEST-CANCELLED",
        shared_terminal_state: "Cancelled",
        error_category: "Cancelled",
        retry_class: "manual",
        operation_stage: "execution",
        safe_recovery: "retry explicitly",
        identity_refs: &["job_id", "attempt_id", "dispatch_id", "worker_id", "publication_id"],
    },
    DispositionMapping {
        video_disposition: "VID-INGEST-WORKER-FAILED",
        shared_terminal_state: "Failed",
        error_category: "WorkerFailure",
        retry_class: "conditional",
        operation_stage: "execution",
        safe_recovery: "retry only when transient failure is verified",
        identity_refs: &["job_id", "attempt_id", "dispatch_id", "worker_id", "publication_id"],
    },
    DispositionMapping {
        video_disposition: "VID-INGEST-IDENTITY-MISMATCH",
        shared_terminal_state: "Failed",
        error_category: "IdentityMismatch",
        retry_class: "non-retryable",
        operation_stage: "prelaunch",
        safe_recovery: "re-probe the current source",
        identity_refs: &["job_id", "attempt_id", "dispatch_id", "worker_id", "publication_id"],
    },
    DispositionMapping {
        video_disposition: "VID-INGEST-REVIEW",
        shared_terminal_state: "Failed",
        error_category: "ManualReview",
        retry_class: "manual",
        operation_stage: "validation",
        safe_recovery: "owner or operator approves a registered profile",
        identity_refs: &["job_id", "attempt_id", "dispatch_id", "worker_id", "publication_id"],
    },
];

pub fn fixed_probe_argv<S: AsRef<str>>(staged_input_path: S) -> Vec<String> {
    let mut argv: Vec<String> = PROBE_ARGV_PREFIX.iter().map(|value| value.to_string()).collect();
    argv.push(staged_input_path.as_ref().to_string());
    argv
}

pub fn validate_probe_prelaunch(
    request: &ProbePrelaunchRequest,
    expected_schema_digests: &HashMap<String, String>,
) -> Result<(), ProbePrelaunchFailure> {
    if request.operation_id != OPERATION_ID {
        return Err(ProbePrelaunchFailure::OperationIdentityMismatch);
    }

    if request.operation_version != OPERATION_VERSION {
        return Err(ProbePrelaunchFailure::OperationVersionMismatch);
    }

    if request.implementation_id != IMPLEMENTATION_ID {
        return Err(ProbePrelaunchFailure::ImplementationMismatch);
    }

    if request.implementation_profile_id != IMPLEMENTATION_PROFILE_ID {
        return Err(ProbePrelaunchFailure::ImplementationProfileMismatch);
    }

    if request.worker_instance_id.trim().is_empty() {
        return Err(ProbePrelaunchFailure::WorkerInstanceMismatch);
    }

    if request.dispatch_id.trim().is_empty() {
        return Err(ProbePrelaunchFailure::DispatchMismatch);
    }

    if request.job_id.trim().is_empty()
        || request.attempt_id.trim().is_empty()
        || request.trace_id.trim().is_empty()
        || request.correlation_id.trim().is_empty()
        || request.error_instance_id.trim().is_empty()
        || request.lease.lease_id.trim().is_empty()
    {
        return Err(ProbePrelaunchFailure::OutOfPolicyIdentity);
    }

    if request.control_protocol_version != CONTROL_PROTOCOL_VERSION {
        return Err(ProbePrelaunchFailure::ControlProtocolMismatch);
    }

    if request.staged_input_path.trim().is_empty() {
        return Err(ProbePrelaunchFailure::PathMissing);
    }

    if !is_local_drive_path(&request.staged_input_path) {
        return Err(ProbePrelaunchFailure::PathInvalid);
    }

    if !is_path_within_scope(&request.staged_input_path, &request.lease.access_scope) {
        return Err(ProbePrelaunchFailure::ScopeMismatch);
    }

    if request.launch_policy.use_shell
        || request.launch_policy.use_path_lookup
        || request.launch_policy.user_supplied_options
    {
        return Err(ProbePrelaunchFailure::InvalidLaunchPolicy);
    }

    if !is_exact_path_match(&request.executable_path, Path::new(REGISTERED_EXECUTABLE_PATH))
        || !is_exact_sha_match(&request.executable_sha256, REGISTERED_EXECUTABLE_SHA256)
    {
        return Err(ProbePrelaunchFailure::ComponentMismatch);
    }

    if request.transport != "local-staged-file" {
        return Err(ProbePrelaunchFailure::TransportUnsupported);
    }

    for required in SUPPORTED_CAPABILITIES {
        if !request.capabilities.iter().any(|value| value == required) {
            return Err(ProbePrelaunchFailure::MissingRequiredCapability(required));
        }
    }

    for capability in request.capabilities.iter() {
        if !SUPPORTED_CAPABILITIES.contains(&capability.as_str()) {
            return Err(ProbePrelaunchFailure::UnsupportedCapability(
                capability.clone(),
            ));
        }
    }

    if !request.lease.immutable {
        return Err(ProbePrelaunchFailure::MutableLease);
    }

    if request.lease.lease_epoch < request.lease.minimum_acceptable_lease_epoch {
        return Err(ProbePrelaunchFailure::StaleLeaseOrFence);
    }

    if request.lease.observed_fence_token != request.lease.expected_fence_token {
        return Err(ProbePrelaunchFailure::StaleLeaseOrFence);
    }

    if !is_hex_string_64(&request.lease.source_sha256)
        || !is_hex_string_64(&request.lease.staged_copy_sha256)
        || !is_hex_string_64(&request.lease.observed_staged_copy_sha256)
        || !is_hex_string_64(&request.lease.manifest_digest)
    {
        return Err(ProbePrelaunchFailure::SourceIdentityMismatch);
    }

    if request.lease.source_sha256 != request.lease.staged_copy_sha256
        || request.lease.source_sha256 != request.lease.observed_staged_copy_sha256
    {
        return Err(ProbePrelaunchFailure::SourceHashMismatch);
    }

    if request.lease.source_length != request.lease.staged_copy_length
        || request.lease.source_length != request.lease.observed_staged_copy_length
    {
        return Err(ProbePrelaunchFailure::SourceLengthMismatch);
    }

    if request.lease.staged_copy_identity != request.lease.observed_staged_copy_identity {
        return Err(ProbePrelaunchFailure::SourceIdentityMismatch);
    }

    if request.lease.logical_source_ref.trim().is_empty() || request.lease.manifest_digest.trim().is_empty() {
        return Err(ProbePrelaunchFailure::SourceIdentityMismatch);
    }

    if !is_valid_rfc3339_utc_expiry(&request.lease.expires_at) {
        return Err(ProbePrelaunchFailure::ExpiredLease);
    }

    if is_expired_lease(&request.lease.expires_at).map_err(|_| ProbePrelaunchFailure::ExpiredLease)? {
        return Err(ProbePrelaunchFailure::ExpiredLease);
    }

    if request.output_envelope_bytes > CONTROL_ENVELOPE_BYTES {
        return Err(ProbePrelaunchFailure::EnvelopeExceeded);
    }

    if !request.evidence.network_denied {
        return Err(ProbePrelaunchFailure::MissingContainmentEvidence(
            "network-denial",
        ));
    }

    if !request.evidence.watchdog_enabled {
        return Err(ProbePrelaunchFailure::MissingContainmentEvidence("watchdog"));
    }

    if !request.evidence.job_object_enabled {
        return Err(ProbePrelaunchFailure::MissingContainmentEvidence("job-object"));
    }

    let expected_argv = fixed_probe_argv(&request.staged_input_path);
    if request.launch_argv != expected_argv {
        return Err(ProbePrelaunchFailure::ArgumentMismatch);
    }

    if request.schema_digests.len() != expected_schema_digests.len() {
        return Err(ProbePrelaunchFailure::SchemaReferenceMismatch);
    }

    if request.schema_digests.len() == 0 {
        return Err(ProbePrelaunchFailure::SchemaReferenceMismatch);
    }

    for expected_schema_id in EXPECTED_SCHEMA_REFERENCE_IDS.iter() {
        if !request.schema_digests.contains_key(*expected_schema_id) {
            return Err(ProbePrelaunchFailure::SchemaReferenceMismatch);
        }
    }

    for (schema_id, expected_digest) in expected_schema_digests.iter() {
        let observed = request
            .schema_digests
            .get(schema_id)
            .ok_or_else(|| ProbePrelaunchFailure::SchemaDigestMismatch(schema_id.clone()))?;

        if !is_hex_equal(observed, expected_digest) {
            return Err(ProbePrelaunchFailure::SchemaDigestMismatch(schema_id.clone()));
        }
    }

    Ok(())
}

pub fn validate_probe_output_completeness(completeness: &str) -> Result<(), ProbePrelaunchFailure> {
    if completeness != "complete" {
        return Err(ProbePrelaunchFailure::PartialCompletenessRejected);
    }
    Ok(())
}

pub fn to_structured_error(
    failure: &ProbePrelaunchFailure,
    request: &ProbePrelaunchRequest,
) -> StructuredError {
    let mapping = disposition_mapping(failure);
    let identity_refs = mapping.identity_refs.iter().map(|identity| (*identity).to_string()).collect();
    let mut details = Vec::new();
    let mut causes = Vec::new();

    details.push(failure.code().to_string());
    if !request.trace_id.is_empty() {
        details.push(request.trace_id.clone());
    }
    causes.push(failure.code().to_string());
    if !request.lease.lease_id.is_empty() {
        causes.push(request.lease.lease_id.clone());
    }

    StructuredError {
        schema_id: STRUCTURED_ERROR_SCHEMA_ID,
        schema_version: STRUCTURED_ERROR_SCHEMA_VERSION,
        error_ref: ErrorRef {
            namespace: ERROR_REF_NAMESPACE,
            code: failure.code().to_string(),
            version: ERROR_REF_VERSION,
            error_instance: request.error_instance_id.clone(),
        },
        trace_id: request.trace_id.clone(),
        correlation_id: request.correlation_id.clone(),
        subject: "prelaunch validation".to_string(),
        video_disposition: mapping.video_disposition.to_string(),
        shared_state: mapping.shared_terminal_state.to_string(),
        error_category: mapping.error_category.to_string(),
        retry_class: mapping.retry_class.to_string(),
        operation_stage: mapping.operation_stage.to_string(),
        safe_recovery: mapping.safe_recovery.to_string(),
        identity_refs,
        details: details.into_iter().take(3).collect(),
        causes: causes.into_iter().take(3).collect(),
        applies_to: StructuredErrorAppliesTo {
            job_ref: request.job_id.clone(),
            attempt_ref: request.attempt_id.clone(),
            dispatch_ref: request.dispatch_id.clone(),
            worker_ref: request.worker_instance_id.clone(),
            publication_ref: None,
        },
    }
}

fn disposition_mapping(failure: &ProbePrelaunchFailure) -> &'static DispositionMapping {
    match failure {
        ProbePrelaunchFailure::OperationIdentityMismatch
        | ProbePrelaunchFailure::OperationVersionMismatch
        | ProbePrelaunchFailure::ImplementationMismatch
        | ProbePrelaunchFailure::ImplementationProfileMismatch
        | ProbePrelaunchFailure::WorkerInstanceMismatch
        | ProbePrelaunchFailure::DispatchMismatch
        | ProbePrelaunchFailure::ControlProtocolMismatch
        | ProbePrelaunchFailure::SchemaReferenceMismatch
        | ProbePrelaunchFailure::SourceIdentityMismatch
        | ProbePrelaunchFailure::SourceHashMismatch
        | ProbePrelaunchFailure::SourceLengthMismatch
        | ProbePrelaunchFailure::OutOfPolicyIdentity => {
            return fallback_disposition("VID-INGEST-IDENTITY-MISMATCH");
        }
        ProbePrelaunchFailure::PathMissing
        | ProbePrelaunchFailure::PathInvalid
        | ProbePrelaunchFailure::ScopeMismatch
        | ProbePrelaunchFailure::ExpiredLease
        | ProbePrelaunchFailure::StaleLeaseOrFence
        | ProbePrelaunchFailure::PartialCompletenessRejected
        | ProbePrelaunchFailure::SchemaDigestMismatch(_)
        | ProbePrelaunchFailure::ComponentMismatch
        | ProbePrelaunchFailure::InvalidLaunchPolicy
        | ProbePrelaunchFailure::TransportUnsupported => {
            fallback_disposition("VID-INGEST-MALFORMED")
        }
        ProbePrelaunchFailure::MissingContainmentEvidence(_) => {
            fallback_disposition("VID-INGEST-PROTECTED")
        }
        ProbePrelaunchFailure::UnsupportedCapability(_) | ProbePrelaunchFailure::MissingRequiredCapability(_) => {
            fallback_disposition("VID-INGEST-UNSUPPORTED")
        }
        ProbePrelaunchFailure::EnvelopeExceeded => {
            fallback_disposition("VID-INGEST-LIMIT")
        }
        ProbePrelaunchFailure::MutableLease => {
            fallback_disposition("VID-INGEST-MALFORMED")
        }
        ProbePrelaunchFailure::ArgumentMismatch => {
            fallback_disposition("VID-INGEST-MALFORMED")
        }
    }
}

fn fallback_disposition(video_disposition: &str) -> &'static DispositionMapping {
    DISPOSITION_MAP
        .iter()
        .find(|entry| entry.video_disposition == video_disposition)
        .expect("all fixture dispositions are complete")
}

fn is_hex_equal(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn is_exact_path_match(path: &str, expected: &Path) -> bool {
    Path::new(path).to_string_lossy().eq_ignore_ascii_case(expected.to_string_lossy().as_ref())
}

fn is_exact_sha_match(candidate: &str, expected: &str) -> bool {
    candidate.eq_ignore_ascii_case(expected)
}

fn is_hex_string_64(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|value| value.is_ascii_hexdigit())
}

fn is_local_drive_path(path: &str) -> bool {
    let mut normalized = Path::new(path).components();
    let Some(Component::Prefix(prefix)) = normalized.next() else {
        return false;
    };
    let prefix = prefix.as_os_str().to_string_lossy();
    if prefix.len() != 2 {
        return false;
    }
    let mut prefix_chars = prefix.chars();
    let drive = prefix_chars.next().unwrap_or_default();
    drive.is_ascii_alphabetic() && prefix_chars.next() == Some(':')
}

fn is_path_within_scope(path: &str, scope: &str) -> bool {
    let normalized_path = normalize_path_components(path);
    let normalized_scope = normalize_path_components(scope);
    let (Some(path_parts), Some(scope_parts)) = (normalized_path, normalized_scope) else {
        return false;
    };

    if path_parts.len() < scope_parts.len() {
        return false;
    }

    if path_parts[..scope_parts.len()] != scope_parts[..] {
        return false;
    }

    true
}

fn normalize_path_components(path: &str) -> Option<Vec<String>> {
    let mut normalized = Vec::new();
    for component in Path::new(path).components() {
        match component {
            Component::Prefix(prefix) => {
                let value = prefix.as_os_str().to_string_lossy().to_ascii_lowercase();
                if value.len() != 2 {
                    return None;
                }
                normalized.push(value);
            }
            Component::Normal(part) => {
                let value = part.to_string_lossy();
                if value == "." {
                    continue;
                }
                if value == ".." {
                    if normalized.is_empty() {
                        return None;
                    }
                    normalized.pop();
                    continue;
                }
                normalized.push(value.to_ascii_lowercase());
            }
            Component::RootDir => {
                continue;
            }
            _ => continue,
        }
    }

    if let Some(first) = normalized.first() {
        if first.len() == 2 && first.ends_with(':') {
            return Some(normalized);
        }
    }
    None
}

fn is_valid_rfc3339_utc_expiry(value: &str) -> bool {
    parse_rfc3339_to_unix_seconds(value).is_some()
}

fn is_expired_lease(value: &str) -> Result<bool, ProbePrelaunchFailure> {
    let observed = parse_rfc3339_to_unix_seconds(value)
        .ok_or(ProbePrelaunchFailure::ExpiredLease)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ProbePrelaunchFailure::ExpiredLease)?
        .as_secs() as i64;
    Ok(observed <= now)
}

fn parse_rfc3339_to_unix_seconds(value: &str) -> Option<i64> {
    let value = value.trim();
    if value.is_empty() || !value.ends_with('Z') {
        return None;
    }

    let t_pos = value.find('T')?;
    let date_text = &value[..t_pos];
    let time_and_fraction = &value[t_pos + 1..value.len() - 1];

    let date_parts: Vec<_> = date_text.split('-').collect();
    if date_parts.len() != 3 {
        return None;
    }

    let year = date_parts.first()?.parse::<i32>().ok()?;
    let month = date_parts.get(1)?.parse::<i32>().ok()?;
    let day = date_parts.get(2)?.parse::<i32>().ok()?;

    if !(1..=12).contains(&month) {
        return None;
    }

    let days_in_month = [
        31,
        if is_leap_year(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let month_index = (month - 1) as usize;
    let day_limit = days_in_month[month_index as usize];
    if day <= 0 || day > day_limit {
        return None;
    }

    let time_text = time_and_fraction.split('.').next().unwrap_or("");
    let time_fields: Vec<_> = time_text.split(':').collect();
    if time_fields.len() != 3 {
        return None;
    }

    let hour = time_fields.first()?.parse::<i64>().ok()?;
    let minute = time_fields.get(1)?.parse::<i64>().ok()?;
    let second = time_fields.get(2)?.parse::<i64>().ok()?;
    if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) || !(0..=59).contains(&second) {
        return None;
    }

    let mut days = 0i64;
    let mut y = 1970;
    while y < year {
        days += if is_leap_year(y) { 366 } else { 365 };
        y += 1;
    }

    for m in 1..month {
        days += days_in_month[(m - 1) as usize] as i64;
    }
    days += (day - 1) as i64;

    Some(days * 86_400 + hour * 3_600 + minute * 60 + second)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
