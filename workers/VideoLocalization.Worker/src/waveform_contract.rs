use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub const OPERATION_ID: &str = "submark.video-localization.waveform.derive";
pub const OPERATION_VERSION: &str = "0.1.0-p00";
pub const IMPLEMENTATION_ID: &str = "VID-IMPL-P00-003A";
pub const IMPLEMENTATION_PROFILE_ID: &str = "VID-WAVEFORM-OPERATION-PROFILE-P00-003A";
pub const ERROR_REF_NAMESPACE: &str = "video.waveform";
pub const ERROR_REF_VERSION: &str = "1.0.0-p00";

pub const CACHE_KEY_SCHEMA_ID: &str = "sui.cache-key.cbor-sha256.v1";
pub const CACHE_KEY_PREFIX: &[u8] = b"SUI-CACHE-KEY-V1\0";

pub const BULK_DESCRIPTOR_PROFILE: &str = "VID-WAVEFORM-BULK-DESCRIPTOR-V1";
pub const BULK_DESCRIPTOR_PREFIX: &[u8] = b"VID-WAVEFORM-BULK-DESCRIPTOR-V1\0";

pub const OPERATION_SCHEMA_ID: &str = "VID-IMPL-P00-003A-WAVEFORM-INPUT";
pub const PARAMETER_SCHEMA_ID: &str = "VID-IMPL-P00-003A-WAVEFORM-PARAMETER";
pub const OUTPUT_SCHEMA_ID: &str = "VID-IMPL-P00-003A-WAVEFORM-ARTIFACT";
pub const ERROR_SCHEMA_ID: &str = "VID-IMPL-P00-003A-WAVEFORM-STRUCTURED-ERROR";
pub const RESOURCE_SCHEMA_ID: &str = "VID-IMPL-P00-003A-WAVEFORM-RESOURCE";
pub const POLICY_SCHEMA_ID: &str = "VID-IMPL-P00-003A-WAVEFORM-POLICY";
pub const BULK_DESCRIPTOR_SCHEMA_ID: &str = "VID-IMPL-P00-003A-WAVEFORM-BULK-DESCRIPTOR";
pub const CONTROL_SCHEMA_ID: &str = "VID-IMPL-P00-003A-WAVEFORM-CONTROL";
pub const WORKER_HELLO_SCHEMA_ID: &str = "VID-IMPL-P00-003A-WAVEFORM-WORKER-HELLO";
pub const OPERATION_PROFILE_SCHEMA_ID: &str = "VID-IMPL-P00-003A-WAVEFORM-OPERATION-PROFILE";

const WAVEFORM_MAX_BULK_PAYLOAD_BYTES: u64 = 536_870_912;
const VID_WAVEFORM_DEADLINE_EXCEEDED_CODE: &str = "VID-WAVEFORM-DEADLINE-EXCEEDED";

#[derive(Debug, Clone)]
pub struct WaveformRetryProfile {
    pub max_total_attempts: u64,
    pub base_wait_millis: u64,
    pub max_wait_millis: u64,
    pub backoff: String,
    pub cumulative_wall_seconds: u64,
    pub cumulative_cpu_seconds: u64,
    pub cumulative_ram_bytes: u64,
    pub cumulative_staging_bytes: u64,
    pub cost_ceiling: u64,
    pub applicable_codes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WaveformRetryUsage {
    pub attempts: u64,
    pub wall_seconds: u64,
    pub cpu_seconds: u64,
    pub ram_bytes: u64,
    pub staging_bytes: u64,
    pub cost: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveformRetryDecision {
    pub next_attempt: u64,
    pub wait_millis: u64,
}

impl WaveformRetryProfile {
    pub fn validate_cumulative_usage(&self, usage: &WaveformRetryUsage) -> Result<(), String> {
        if usage.wall_seconds > self.cumulative_wall_seconds {
            return Err("cumulative wall-time would exceed profile ceiling".to_string());
        }
        if usage.cpu_seconds > self.cumulative_cpu_seconds {
            return Err("cumulative cpu-time would exceed profile ceiling".to_string());
        }
        if usage.ram_bytes > self.cumulative_ram_bytes {
            return Err("cumulative resident-memory footprint would exceed profile ceiling".to_string());
        }
        if usage.staging_bytes > self.cumulative_staging_bytes {
            return Err("cumulative staging usage would exceed profile ceiling".to_string());
        }
        if usage.cost > self.cost_ceiling {
            return Err("cumulative cost would exceed profile ceiling".to_string());
        }
        Ok(())
    }

    pub fn evaluate_retry_decision(
        &self,
        status_code: &str,
        usage: &WaveformRetryUsage,
    ) -> Result<WaveformRetryDecision, String> {
        if !self.applicable_codes.iter().any(|code| code == status_code) {
            return Err("status is not retry-eligible in profile".to_string());
        }
        if usage.attempts >= self.max_total_attempts {
            return Err("retry attempt ceiling reached".to_string());
        }
        self.validate_cumulative_usage(usage)?;

        let base_wait = match self.backoff.as_str() {
            "fixed" => self.base_wait_millis,
            _ => self.base_wait_millis,
        };
        let wait_millis = base_wait.min(self.max_wait_millis);
        if wait_millis == 0 {
            return Err("retry wait must be non-zero".to_string());
        }

        Ok(WaveformRetryDecision {
            next_attempt: usage.attempts + 1,
            wait_millis,
        })
    }
}

#[derive(Debug, Clone)]
pub struct WaveformPublicationLease {
    pub lease_id: String,
    pub lease_epoch: u64,
    pub minimum_acceptable_lease_epoch: u64,
    pub expected_fence_token: String,
    pub observed_fence_token: String,
    pub staged_copy_hash: String,
    pub staged_copy_length: u64,
    pub cache_authority: bool,
}

#[derive(Debug, Clone)]
pub struct WaveformPublicationAttempt {
    pub attempt_id: String,
    pub lease: WaveformPublicationLease,
    pub descriptor: WaveformBulkDescriptor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaveformPublicationCommitReceipt {
    journal_identity: String,
}

impl WaveformPublicationCommitReceipt {
    pub fn journal_identity(&self) -> &str {
        &self.journal_identity
    }
}

#[derive(Debug, Clone)]
struct PublicationCommitJournalEntry {
    operation_id: String,
    operation_version: String,
    implementation_id: String,
    implementation_profile_id: String,
    attempt_id: String,
    lease_id: String,
    lease_epoch: u64,
    minimum_acceptable_lease_epoch: u64,
    expected_fence_token: String,
    observed_fence_token: String,
    source_publication_fence: String,
    descriptor_digest: String,
    staged_copy_hash: String,
    staged_copy_length: u64,
    payload_digest: String,
    whole_object_hash: String,
    journal_identity: String,
}

static PUBLICATION_COMMIT_JOURNAL: OnceLock<Mutex<HashMap<String, PublicationCommitJournalEntry>>> =
    OnceLock::new();

fn publication_commit_journal() -> &'static Mutex<HashMap<String, PublicationCommitJournalEntry>> {
    PUBLICATION_COMMIT_JOURNAL.get_or_init(|| Mutex::new(HashMap::new()))
}

fn publication_journal_identity(
    attempt: &WaveformPublicationAttempt,
    payload_digest: &str,
    descriptor_digest: &str,
    whole_object_hash: &str,
) -> String {
    let bytes = cbor_text_map(vec![
        ("attempt_id", cbor_text(&attempt.attempt_id)),
        ("attempt_descriptor", cbor_text(&attempt.descriptor.attempt_id)),
        ("attempt_lease_id", cbor_text(&attempt.lease.lease_id)),
        ("attempt_lease_epoch", cbor_uint(attempt.lease.lease_epoch)),
        (
            "operation_id",
            cbor_text(crate::waveform_contract::OPERATION_ID),
        ),
        ("operation_version", cbor_text(crate::waveform_contract::OPERATION_VERSION)),
        (
            "payload_digest",
            cbor_text(&payload_digest.to_lowercase()),
        ),
        ("descriptor_digest", cbor_text(descriptor_digest)),
        ("descriptor_source_fence", cbor_text(&attempt.descriptor.source_publication_fence)),
        ("expected_fence_token", cbor_text(&attempt.lease.expected_fence_token)),
        ("implementation_id", cbor_text(crate::waveform_contract::IMPLEMENTATION_ID)),
        (
            "implementation_profile_id",
            cbor_text(crate::waveform_contract::IMPLEMENTATION_PROFILE_ID),
        ),
        ("journal_identity_domain", cbor_text("VID-WAVEFORM-COMMIT-JOURNAL")),
        (
            "minimum_acceptable_lease_epoch",
            cbor_uint(attempt.lease.minimum_acceptable_lease_epoch),
        ),
        (
            "observed_fence_token",
            cbor_text(&attempt.lease.observed_fence_token),
        ),
        ("staged_copy_hash", cbor_text(&attempt.lease.staged_copy_hash)),
        ("staged_copy_length", cbor_uint(attempt.lease.staged_copy_length)),
        ("whole_object_hash", cbor_text(whole_object_hash)),
    ]);
    let mut hasher = Sha256::new();
    hasher.update(b"VID-WAVEFORM-JOURNAL-IDENTITY-V1\0");
    hasher.update(bytes);
    digest_to_hex(hasher.finalize().as_slice())
}

fn validate_commit_journal_state(
    identity: &str,
) -> Result<WaveformPublicationCommitReceipt, String> {
    if !is_hex64(identity) {
        return Err("commit journal identity must be a 64-hex receipt id".to_string());
    }

    let journal = publication_commit_journal()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let entry = journal
        .get(identity)
        .ok_or_else(|| "commit journal entry is not registered".to_string())?;

    let sample_attempt = WaveformPublicationAttempt {
        attempt_id: entry.attempt_id.clone(),
        lease: WaveformPublicationLease {
            lease_id: entry.lease_id.clone(),
            lease_epoch: entry.lease_epoch,
            minimum_acceptable_lease_epoch: entry.minimum_acceptable_lease_epoch,
            expected_fence_token: entry.expected_fence_token.clone(),
            observed_fence_token: entry.observed_fence_token.clone(),
            staged_copy_hash: entry.staged_copy_hash.clone(),
            staged_copy_length: entry.staged_copy_length,
            cache_authority: false,
        },
        descriptor: WaveformBulkDescriptor {
            version: BULK_DESCRIPTOR_PROFILE.to_string(),
            mode: "read-only".to_string(),
            handle_identity: String::new(),
            attempt_id: entry.attempt_id.clone(),
            lease_id: entry.lease_id.clone(),
            cancel_scope: String::new(),
            expiry: String::new(),
            integrity: WaveformOutputContractDigest {
                algorithm: "sha-256".to_string(),
                value: entry.payload_digest.clone(),
            },
            chunk_hashes_omitted: true,
            length: entry.staged_copy_length,
            payload_schema: OUTPUT_SCHEMA_ID.to_string(),
            source_publication_fence: entry.source_publication_fence.clone(),
            representation: WaveformBulkRepresentation {
                peak: String::new(),
                time: String::new(),
                channel: String::new(),
                downmix: String::new(),
                resampling: String::new(),
                source_timeline: String::new(),
                sample_range: String::new(),
            },
            descriptor_hash: Some(entry.descriptor_digest.clone()),
        },
    };

    let expected_identity = {
        let mut attempt = sample_attempt;
        attempt.lease.staged_copy_hash = entry.staged_copy_hash.to_lowercase();
        publication_journal_identity(
            &attempt,
            &entry.payload_digest,
            &entry.descriptor_digest,
            &entry.whole_object_hash,
        )
    };

    if expected_identity.to_lowercase() != entry.journal_identity.to_lowercase() {
        return Err("commit journal state mismatch".to_string());
    }
    if entry.journal_identity.to_lowercase() != identity.to_lowercase() {
        return Err("commit journal identity mismatch".to_string());
    }

    Ok(WaveformPublicationCommitReceipt {
        journal_identity: identity.to_string(),
    })
}

pub fn revoke_publication_commit(
    receipt: &WaveformPublicationCommitReceipt,
) -> bool {
    let mut journal = publication_commit_journal()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    journal.remove(&receipt.journal_identity).is_some()
}

#[derive(Debug, Clone)]
enum PublicationLifecycleProgress {
    StagedValidated(WaveformBulkIntegrityReport),
    StagedValidatedCommitted(WaveformPublicationCommitReceipt),
}

#[derive(Debug, Clone)]
pub struct WaveformBulkIntegrityReport {
    pub payload_digest: String,
    pub descriptor_preimage: Vec<u8>,
    pub descriptor_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaveformTerminalState {
    Succeeded(WaveformPublicationCommitReceipt),
    Terminal(String),
}

pub fn sha256_hex_of_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn reconcile_terminal_state(
    disposition_code: Option<&str>,
    admitted_deadline_millis: Option<u64>,
    current_time_millis: Option<u64>,
    commit_journal_identity: Option<&str>,
) -> Result<WaveformTerminalState, String> {
    let deadline_exceeded = match (admitted_deadline_millis, current_time_millis) {
        (Some(deadline_millis), Some(now_millis)) => now_millis >= deadline_millis,
        (Some(_), None) => return Err("current time required when admitted deadline is present".to_string()),
        _ => false,
    };

    let Some(code) = disposition_code else {
        if deadline_exceeded {
            return Err("cannot map to success after expired deadline".to_string());
        }
        let Some(receipt_identity) = commit_journal_identity else {
            return Err("Succeeded requires an authoritative commit journal identity".to_string());
        };
        let authoritative_receipt = validate_commit_journal_state(receipt_identity)?;
        return Ok(WaveformTerminalState::Succeeded(authoritative_receipt));
    };

    if deadline_exceeded && code != VID_WAVEFORM_DEADLINE_EXCEEDED_CODE {
        return Err("TimedOut must be terminal only on deadline code".to_string());
    }
    if code == VID_WAVEFORM_DEADLINE_EXCEEDED_CODE && !deadline_exceeded {
        return Err("deadline code requires expired admitted deadline".to_string());
    }

    let mapping = state_mapping_for_code(code)
        .ok_or_else(|| "unknown disposition code".to_string())?;

    Ok(WaveformTerminalState::Terminal(mapping.shared_terminal_state.to_string()))
}

pub fn validate_bulk_payload_and_descriptor_linkage(
    payload: &[u8],
    descriptor: &WaveformBulkDescriptor,
) -> Result<WaveformBulkIntegrityReport, String> {
    if descriptor.length != payload.len() as u64 {
        return Err("payload length mismatch".to_string());
    }
    if descriptor.integrity.algorithm.to_lowercase() != "sha-256" {
        return Err("integrity.algorithm must be sha-256".to_string());
    }

    let payload_digest = sha256_hex_of_bytes(payload);
    if payload_digest != descriptor.integrity.value.to_lowercase() {
        return Err("payload integrity hash mismatch".to_string());
    }

    let descriptor_preimage = bulk_descriptor_preimage(descriptor)?;
    let descriptor_digest = {
        let mut hasher = Sha256::new();
        hasher.update(BULK_DESCRIPTOR_PREFIX);
        hasher.update(&descriptor_preimage);
        digest_to_hex(hasher.finalize().as_slice())
    };

    if descriptor.descriptor_hash.is_none() {
        return Err("descriptor_hash is required".to_string());
    }
    if descriptor_digest.to_lowercase() != descriptor
        .descriptor_hash
        .as_ref()
        .ok_or_else(|| "descriptor hash required".to_string())?
        .to_lowercase()
    {
        return Err("descriptor hash mismatch".to_string());
    }

    Ok(WaveformBulkIntegrityReport {
        payload_digest,
        descriptor_preimage,
        descriptor_digest,
    })
}

pub fn publication_whole_object_hash(
    attempt: &WaveformPublicationAttempt,
    payload_digest: &str,
) -> Result<String, String> {
    let descriptor_digest = bulk_descriptor_digest(&attempt.descriptor)?;

    let bytes = cbor_text_map(vec![
        ("attempt_id", cbor_text(&attempt.attempt_id)),
        ("descriptor_digest", cbor_text(&descriptor_digest)),
        ("lease_id", cbor_text(&attempt.lease.lease_id)),
        ("lease_epoch", cbor_uint(attempt.lease.lease_epoch)),
        ("payload_digest", cbor_text(payload_digest)),
        ("staged_copy_length", cbor_uint(attempt.lease.staged_copy_length)),
        ("staged_copy_hash", cbor_text(&attempt.lease.staged_copy_hash)),
        (
            "minimum_acceptable_lease_epoch",
            cbor_uint(attempt.lease.minimum_acceptable_lease_epoch),
        ),
        ("expected_fence_token", cbor_text(&attempt.lease.expected_fence_token)),
        ("observed_fence_token", cbor_text(&attempt.lease.observed_fence_token)),
        ("descriptor_source_fence", cbor_text(&attempt.descriptor.source_publication_fence)),
        ("descriptor_lease_id", cbor_text(&attempt.descriptor.lease_id)),
        ("descriptor_attempt_id", cbor_text(&attempt.descriptor.attempt_id)),
    ]);
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(digest_to_hex(hasher.finalize().as_slice()))
}

pub fn validate_publication_lifecycle(
    attempt: &WaveformPublicationAttempt,
    payload: &[u8],
    expected_whole_object_hash: &str,
 ) -> Result<WaveformPublicationCommitReceipt, String> {
    validate_opaque_handle_identity(&attempt.descriptor.handle_identity, "publication descriptor")?;
    if attempt.lease.cache_authority {
        return Err("cache authority cannot satisfy publication lineage".to_string());
    }

    let report = validate_bulk_payload_and_descriptor_linkage(payload, &attempt.descriptor)?;
    if !is_hex64(&attempt.lease.staged_copy_hash) {
        return Err("staged copy hash must be 64-hex".to_string());
    }
    if report.payload_digest != attempt.lease.staged_copy_hash.to_lowercase() {
        return Err("payload digest mismatch staged copy hash".to_string());
    }

    if attempt.attempt_id != attempt.descriptor.attempt_id {
        return Err("attempt_id mismatch between attempt and descriptor".to_string());
    }
    if attempt.lease.lease_id != attempt.descriptor.lease_id {
        return Err("lease id mismatch between attempt and descriptor".to_string());
    }
    if attempt.lease.staged_copy_length != attempt.descriptor.length {
        return Err("staging length mismatch descriptor length".to_string());
    }
    if attempt.lease.lease_epoch < attempt.lease.minimum_acceptable_lease_epoch {
        return Err("lease is stale".to_string());
    }
    if attempt.descriptor.source_publication_fence != attempt.lease.expected_fence_token {
        return Err("descriptor source publication fence mismatch".to_string());
    }
    if attempt.lease.expected_fence_token != attempt.lease.observed_fence_token {
        return Err("fence token mismatch".to_string());
    }

    let mut lifecycle = PublicationLifecycleProgress::StagedValidated(report);
    lifecycle = match &lifecycle {
        PublicationLifecycleProgress::StagedValidated(staged_report) => {
            let actual_whole_object_hash = publication_whole_object_hash(attempt, &staged_report.payload_digest)?;
            if actual_whole_object_hash.to_lowercase() != expected_whole_object_hash.to_lowercase() {
                return Err("whole object hash mismatch".to_string());
            }
            let descriptor_digest = bulk_descriptor_digest(&attempt.descriptor)?;
            let journal_identity = publication_journal_identity(
                attempt,
                &staged_report.payload_digest,
                &descriptor_digest,
                &actual_whole_object_hash,
            );
            let mut journal = publication_commit_journal()
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            if journal.contains_key(&journal_identity) {
                return Err("publication lineage already committed".to_string());
            }
            if journal.values().any(|entry| entry.attempt_id == attempt.attempt_id) {
                return Err("publication attempt reuse is not allowed".to_string());
            }
            let entry = PublicationCommitJournalEntry {
                operation_id: OPERATION_ID.to_string(),
                operation_version: OPERATION_VERSION.to_string(),
                implementation_id: IMPLEMENTATION_ID.to_string(),
                implementation_profile_id: IMPLEMENTATION_PROFILE_ID.to_string(),
                attempt_id: attempt.attempt_id.clone(),
                lease_id: attempt.lease.lease_id.clone(),
                lease_epoch: attempt.lease.lease_epoch,
                minimum_acceptable_lease_epoch: attempt.lease.minimum_acceptable_lease_epoch,
                expected_fence_token: attempt.lease.expected_fence_token.clone(),
                observed_fence_token: attempt.lease.observed_fence_token.clone(),
                source_publication_fence: attempt.descriptor.source_publication_fence.clone(),
                descriptor_digest,
                staged_copy_hash: attempt.lease.staged_copy_hash.to_lowercase(),
                staged_copy_length: attempt.lease.staged_copy_length,
                payload_digest: staged_report.payload_digest.to_lowercase(),
                whole_object_hash: actual_whole_object_hash.clone(),
                journal_identity: journal_identity.clone(),
            };
            journal.insert(journal_identity.clone(), entry);

            let receipt = WaveformPublicationCommitReceipt {
                journal_identity,
            };

            PublicationLifecycleProgress::StagedValidatedCommitted(receipt)
        }
        _ => return Err("invalid publication lifecycle transition".to_string()),
    };

    match lifecycle {
        PublicationLifecycleProgress::StagedValidatedCommitted(receipt) => Ok(receipt),
        _ => Err("invalid publication lifecycle transition".to_string()),
    }
}

#[derive(Debug, Clone)]
pub struct WaveformStateMapping {
    pub video_disposition: &'static str,
    pub shared_terminal_state: &'static str,
    pub error_category: &'static str,
    pub retry_class: &'static str,
    pub operation_stage: &'static str,
    pub safe_recovery: &'static str,
    pub identity_refs: &'static [&'static str],
}

pub const VID_WAVEFORM_STATE_MAPPINGS: &[WaveformStateMapping] = &[
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-CANCELLED",
        shared_terminal_state: "Cancelled",
        error_category: "Cancelled",
        retry_class: "manual",
        operation_stage: "execution",
        safe_recovery: "retry explicitly in user-directed workflow",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-DEADLINE-EXCEEDED",
        shared_terminal_state: "TimedOut",
        error_category: "ResourceLimit",
        retry_class: "non-retryable",
        operation_stage: "execution",
        safe_recovery: "use a reviewed lower-cost profile",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-RESOURCE-LIMIT",
        shared_terminal_state: "Failed",
        error_category: "ResourceLimit",
        retry_class: "non-retryable",
        operation_stage: "resource-management",
        safe_recovery: "select an approved lower resource profile",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-LEASE-EXPIRED",
        shared_terminal_state: "Failed",
        error_category: "ExpiredLease",
        retry_class: "non-retryable",
        operation_stage: "publication",
        safe_recovery: "renew lease and replay source staging",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-FENCE-MISMATCH",
        shared_terminal_state: "Failed",
        error_category: "IdentityMismatch",
        retry_class: "non-retryable",
        operation_stage: "publication",
        safe_recovery: "repair publication/lease binding context",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-CACHE-STALE",
        shared_terminal_state: "Failed",
        error_category: "CacheStale",
        retry_class: "manual",
        operation_stage: "cache-lookup",
        safe_recovery: "force recompute through a new attempt",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-CACHE-CORRUPT",
        shared_terminal_state: "Failed",
        error_category: "DataCorruption",
        retry_class: "manual",
        operation_stage: "validation",
        safe_recovery: "recompute cache line with new attempt",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-INPUT-SCHEMA-INVALID",
        shared_terminal_state: "Failed",
        error_category: "MalformedInput",
        retry_class: "non-retryable",
        operation_stage: "validation",
        safe_recovery: "repair input schema or retry with corrected admission data",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-OUTPUT-SCHEMA-INVALID",
        shared_terminal_state: "Failed",
        error_category: "MalformedOutput",
        retry_class: "non-retryable",
        operation_stage: "validation",
        safe_recovery: "repair validator or runtime adapter mapping",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-INTEGRITY-MISMATCH",
        shared_terminal_state: "Failed",
        error_category: "IntegrityMismatch",
        retry_class: "manual",
        operation_stage: "validation",
        safe_recovery: "rebuild artifact from the same source",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-MALFORMED-DESCRIPTOR",
        shared_terminal_state: "Failed",
        error_category: "MalformedOutput",
        retry_class: "non-retryable",
        operation_stage: "validation",
        safe_recovery: "repair descriptor construction and retry",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-PROTOCOL-INCOMPATIBLE",
        shared_terminal_state: "Failed",
        error_category: "ProtocolMismatch",
        retry_class: "non-retryable",
        operation_stage: "negotiation",
        safe_recovery: "select a compatible operation protocol",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-HANDLE-INCOMPATIBLE",
        shared_terminal_state: "Failed",
        error_category: "HandleMismatch",
        retry_class: "non-retryable",
        operation_stage: "validation",
        safe_recovery: "repair handle/lease lineage",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-PUBLICATION-FAILED",
        shared_terminal_state: "Failed",
        error_category: "PublicationFailure",
        retry_class: "manual",
        operation_stage: "publication",
        safe_recovery: "repair publication path and retry with same lineage",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-STORAGE-PERMANENT",
        shared_terminal_state: "Failed",
        error_category: "StorageFault",
        retry_class: "non-retryable",
        operation_stage: "storage",
        safe_recovery: "repair storage and verify permissions",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-JOURNAL-INVALID",
        shared_terminal_state: "Failed",
        error_category: "MalformedOutput",
        retry_class: "manual",
        operation_stage: "reconciliation",
        safe_recovery: "repair journal or operator review",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-PARTIAL-OUTPUT",
        shared_terminal_state: "Failed",
        error_category: "MalformedOutput",
        retry_class: "non-retryable",
        operation_stage: "validation",
        safe_recovery: "recompute entire output only",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-METADATA-INVALID",
        shared_terminal_state: "Failed",
        error_category: "MalformedInput",
        retry_class: "non-retryable",
        operation_stage: "validation",
        safe_recovery: "repair metadata bindings and retry",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-IDENTITY-MISMATCH",
        shared_terminal_state: "Failed",
        error_category: "IdentityMismatch",
        retry_class: "non-retryable",
        operation_stage: "admission",
        safe_recovery: "repair identity bindings and retry",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-POLICY-DENIED",
        shared_terminal_state: "Failed",
        error_category: "PolicyDenied",
        retry_class: "manual",
        operation_stage: "policy",
        safe_recovery: "repair policy binding and retry",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-WORKER-TRANSIENT",
        shared_terminal_state: "RetryWaiting",
        error_category: "TransientWorker",
        retry_class: "transient",
        operation_stage: "execution",
        safe_recovery: "retry after bounded delay",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-STORAGE-TRANSIENT",
        shared_terminal_state: "RetryWaiting",
        error_category: "TransientStorage",
        retry_class: "transient",
        operation_stage: "storage",
        safe_recovery: "retry after bounded delay",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-WORKER-PERMANENT",
        shared_terminal_state: "Failed",
        error_category: "WorkerPermanent",
        retry_class: "non-retryable",
        operation_stage: "execution",
        safe_recovery: "operator review and repair implementation",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
    WaveformStateMapping {
        video_disposition: "VID-WAVEFORM-REVIEW",
        shared_terminal_state: "Failed",
        error_category: "ManualReview",
        retry_class: "manual",
        operation_stage: "review",
        safe_recovery: "operator manual review",
        identity_refs: &["job", "attempt", "dispatch", "worker", "publication"],
    },
];

pub fn state_mapping_for_code(code: &str) -> Option<&'static WaveformStateMapping> {
    VID_WAVEFORM_STATE_MAPPINGS.iter().find(|entry| entry.video_disposition == code)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaveformOutputContractDigest {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaveformCacheKeyProfile {
    pub cache_key_schema: String,
    pub operation: WaveformCacheKeyProfileIdentity,
    pub implementation: WaveformCacheKeyProfileIdentity,
    pub inputs: Vec<WaveformCacheInputPort>,
    pub parameters: WaveformCacheParameters,
    pub dependencies: WaveformCacheDependencies,
    pub output_contract: WaveformCacheOutputContract,
    pub seed: String,
    pub environment_equivalence: WaveformEnvironmentEquivalence,
    pub resource: WaveformCacheResource,
    pub policy: WaveformCachePolicy,
    pub representation: WaveformCacheRepresentation,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaveformCacheKeyProfileIdentity {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaveformCacheInputPort {
    pub order: u64,
    #[serde(rename = "type")]
    pub kind: String,
    pub schema: String,
    pub content: String,
    pub revision: String,
    pub sample_range: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaveformCacheParameters {
    pub cache_key_schema: String,
    pub cache_key_seed: String,
    pub environment_equivalence: WaveformEnvironmentEquivalence,
    pub resource_profile_id: String,
    pub policy_profile_id: String,
    pub retry_profile_id: String,
    pub representation: WaveformCacheRepresentation,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaveformCacheDependencies {
    pub resource_profile: String,
    pub policy_profile: String,
    pub retry_profile: String,
    pub operation_profile: String,
    pub cache_key_schema: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaveformCacheOutputContract {
    pub artifact_id: String,
    pub artifact_version: String,
    pub output_schema_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaveformEnvironmentEquivalence {
    pub os_family: String,
    pub arch_family: String,
    pub worker_family: String,
    pub containerization: bool,
    pub capability_bundle: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaveformCacheResource {
    pub resource_profile_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaveformCachePolicy {
    pub network: String,
    pub watchdog_required: bool,
    pub job_object_required: bool,
    pub media_parse: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaveformCacheRepresentation {
    pub peak: String,
    pub time: String,
    pub channel: String,
    pub downmix: String,
    pub resampling: String,
    pub stability: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaveformBulkDescriptor {
    pub version: String,
    pub mode: String,
    pub handle_identity: String,
    pub attempt_id: String,
    pub lease_id: String,
    pub cancel_scope: String,
    pub expiry: String,
    pub integrity: WaveformOutputContractDigest,
    pub chunk_hashes_omitted: bool,
    pub length: u64,
    pub payload_schema: String,
    pub source_publication_fence: String,
    pub representation: WaveformBulkRepresentation,
    pub descriptor_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WaveformBulkRepresentation {
    pub peak: String,
    pub time: String,
    pub channel: String,
    pub downmix: String,
    pub resampling: String,
    pub source_timeline: String,
    pub sample_range: String,
}

pub fn cache_key_digest(profile: &WaveformCacheKeyProfile) -> Result<String, String> {
    let preimage = cache_key_preimage(profile)?;
    let mut hasher = Sha256::new();
    hasher.update(CACHE_KEY_PREFIX);
    hasher.update(preimage);
    Ok(digest_to_hex(hasher.finalize().as_slice()))
}

pub fn cache_key_preimage(profile: &WaveformCacheKeyProfile) -> Result<Vec<u8>, String> {
    validate_cache_key_profile(profile)?;

    let mut inputs = profile.inputs.clone();
    inputs.sort_by(|left, right| {
        (
            left.order,
            left.schema.as_str(),
            left.kind.as_str(),
            left.content.as_str(),
            left.revision.as_str(),
            left.sample_range.as_str(),
        )
            .cmp(&(
                right.order,
                right.schema.as_str(),
                right.kind.as_str(),
                right.content.as_str(),
                right.revision.as_str(),
                right.sample_range.as_str(),
            ))
    });

    let encoded_inputs = inputs
        .iter()
        .map(encode_cache_input_port)
        .collect::<Vec<_>>();
    let output_contract = cbor_text_map(vec![
        ("artifact_id", cbor_text(&profile.output_contract.artifact_id)),
        ("artifact_version", cbor_text(&profile.output_contract.artifact_version)),
        ("output_schema_id", cbor_text(&profile.output_contract.output_schema_id)),
    ]);
    let operation = cbor_text_map(vec![
        ("id", cbor_text(&profile.operation.id)),
        ("version", cbor_text(&profile.operation.version)),
    ]);
    let implementation = cbor_text_map(vec![
        ("id", cbor_text(&profile.implementation.id)),
        ("version", cbor_text(&profile.implementation.version)),
    ]);
    let parameters = cbor_text_map(vec![
        ("cache_key_schema", cbor_text(&profile.parameters.cache_key_schema)),
        ("cache_key_seed", cbor_text(&profile.parameters.cache_key_seed)),
        (
            "environment_equivalence",
            cbor_text_map(vec![
                ("arch_family", cbor_text(&profile.parameters.environment_equivalence.arch_family)),
                ("capability_bundle", cbor_text(&profile.parameters.environment_equivalence.capability_bundle)),
                ("containerization", cbor_bool(profile.parameters.environment_equivalence.containerization)),
                ("os_family", cbor_text(&profile.parameters.environment_equivalence.os_family)),
                ("worker_family", cbor_text(&profile.parameters.environment_equivalence.worker_family)),
            ]),
        ),
        ("policy_profile_id", cbor_text(&profile.parameters.policy_profile_id)),
        ("representation", {
            let rep = &profile.parameters.representation;
            cbor_text_map(vec![
                ("channel", cbor_text(&rep.channel)),
                ("downmix", cbor_text(&rep.downmix)),
                ("peak", cbor_text(&rep.peak)),
                ("resampling", cbor_text(&rep.resampling)),
                ("stability", cbor_text(&rep.stability)),
                ("time", cbor_text(&rep.time)),
            ])
        }),
        ("resource_profile_id", cbor_text(&profile.parameters.resource_profile_id)),
        ("retry_profile_id", cbor_text(&profile.parameters.retry_profile_id)),
    ]);
    let dependencies = cbor_text_map(vec![
        ("cache_key_schema", cbor_text(&profile.dependencies.cache_key_schema)),
        ("operation_profile", cbor_text(&profile.dependencies.operation_profile)),
        ("policy_profile", cbor_text(&profile.dependencies.policy_profile)),
        ("retry_profile", cbor_text(&profile.dependencies.retry_profile)),
        ("resource_profile", cbor_text(&profile.dependencies.resource_profile)),
    ]);
    let environment_equivalence = cbor_text_map(vec![
        ("arch_family", cbor_text(&profile.environment_equivalence.arch_family)),
        ("capability_bundle", cbor_text(&profile.environment_equivalence.capability_bundle)),
        ("containerization", cbor_bool(profile.environment_equivalence.containerization)),
        ("os_family", cbor_text(&profile.environment_equivalence.os_family)),
        ("worker_family", cbor_text(&profile.environment_equivalence.worker_family)),
    ]);
    let resource = cbor_text_map(vec![(
        "resource_profile_id",
        cbor_text(&profile.resource.resource_profile_id),
    )]);
    let policy = cbor_text_map(vec![
        ("job_object_required", cbor_bool(profile.policy.job_object_required)),
        ("media_parse", cbor_bool(profile.policy.media_parse)),
        ("network", cbor_text(&profile.policy.network)),
        ("watchdog_required", cbor_bool(profile.policy.watchdog_required)),
    ]);
    let representation = cbor_text_map(vec![
        ("channel", cbor_text(&profile.representation.channel)),
        ("downmix", cbor_text(&profile.representation.downmix)),
        ("peak", cbor_text(&profile.representation.peak)),
        ("resampling", cbor_text(&profile.representation.resampling)),
        ("stability", cbor_text(&profile.representation.stability)),
        ("time", cbor_text(&profile.representation.time)),
    ]);

    let encoded_inputs = cbor_array(encoded_inputs);
    let preimage = cbor_text_map(vec![
        ("cache_key_schema", cbor_text(&profile.cache_key_schema)),
        ("dependencies", dependencies),
        ("environment_equivalence", environment_equivalence),
        ("implementation", implementation),
        ("inputs", encoded_inputs),
        ("operation", operation),
        ("output_contract", output_contract),
        ("parameters", parameters),
        ("policy", policy),
        ("representation", representation),
        ("resource", resource),
        ("seed", cbor_text(&profile.seed)),
    ]);
    Ok(preimage)

}

pub fn bulk_descriptor_preimage(descriptor: &WaveformBulkDescriptor) -> Result<Vec<u8>, String> {
    validate_bulk_descriptor_minimal(descriptor)?;
    Ok(write_bulk_descriptor_bytes(descriptor, false))
}

pub fn bulk_descriptor_canonical_bytes(descriptor: &WaveformBulkDescriptor) -> Result<Vec<u8>, String> {
    validate_bulk_descriptor_minimal(descriptor)?;
    Ok(write_bulk_descriptor_bytes(descriptor, true))
}

pub fn bulk_descriptor_digest(descriptor: &WaveformBulkDescriptor) -> Result<String, String> {
    let preimage = bulk_descriptor_preimage(descriptor)?;
    let mut hasher = Sha256::new();
    hasher.update(BULK_DESCRIPTOR_PREFIX);
    hasher.update(&preimage);
    Ok(digest_to_hex(hasher.finalize().as_slice()))
}

#[doc(hidden)]
pub fn bulk_descriptor_digest_unchecked(descriptor: &WaveformBulkDescriptor) -> String {
    let mut hasher = Sha256::new();
    hasher.update(BULK_DESCRIPTOR_PREFIX);
    hasher.update(write_bulk_descriptor_bytes(descriptor, false));
    digest_to_hex(hasher.finalize().as_slice())
}

fn validate_bulk_descriptor_minimal(descriptor: &WaveformBulkDescriptor) -> Result<(), String> {
    validate_opaque_handle_identity(&descriptor.handle_identity, "descriptor")?;

    if descriptor.version != BULK_DESCRIPTOR_PROFILE {
        return Err("bulk descriptor version invalid".to_string());
    }
    if descriptor.mode != "read-only" {
        return Err("bulk descriptor mode must be read-only".to_string());
    }
    if !descriptor.chunk_hashes_omitted {
        return Err("chunk_hashes_omitted must be true".to_string());
    }
    if descriptor.integrity.algorithm.to_lowercase() != "sha-256" {
        return Err("integrity.algorithm must be sha-256".to_string());
    }
    if !is_hex64(&descriptor.integrity.value) {
        return Err("integrity.value must be a 64-hex SHA-256 digest".to_string());
    }
    if descriptor.length > WAVEFORM_MAX_BULK_PAYLOAD_BYTES {
        return Err("descriptor length exceeds maximum".to_string());
    }
    for value in [
        &descriptor.version,
        &descriptor.mode,
        &descriptor.handle_identity,
        &descriptor.attempt_id,
        &descriptor.lease_id,
        &descriptor.cancel_scope,
        &descriptor.expiry,
        &descriptor.payload_schema,
        &descriptor.source_publication_fence,
        &descriptor.representation.peak,
        &descriptor.representation.time,
        &descriptor.representation.channel,
        &descriptor.representation.downmix,
        &descriptor.representation.resampling,
        &descriptor.representation.source_timeline,
        &descriptor.representation.sample_range,
    ] {
        if value.trim().is_empty() {
            return Err("descriptor identity fields must be explicit".to_string());
        }
    }
    Ok(())
}

fn validate_opaque_handle_identity(identity: &str, context: &str) -> Result<(), String> {
    if identity.trim().is_empty() {
        return Err(format!("{} handle identity must be non-empty", context));
    }

    if !identity.chars().all(|character| {
        character.is_ascii_alphanumeric()
            || character == '-'
            || character == '_'
            || character == '.'
    }) {
        return Err(format!(
            "{context} handle identity must be an opaque, non-path token"
        ));
    }

    if identity.chars().any(|character| character.is_whitespace() || character.is_control()) {
        return Err(format!(
            "{context} handle identity must not contain whitespace or control characters"
        ));
    }

    if identity.contains("..") {
        return Err(format!(
            "{context} handle identity must not include traversal segments"
        ));
    }

    if identity.contains("://") || identity.starts_with("file:") {
        return Err(format!("{context} handle identity must not use URI schemes"));
    }

    let mut chars = identity.chars();
    if chars.next().is_some() {
        let mut chars = identity.bytes();
        let first = chars.next().unwrap_or_default();
        if first.is_ascii_alphabetic() && chars.next() == Some(b':') {
            return Err(format!("{context} handle identity must not use drive path syntax"));
        }
    }

    Ok(())
}

pub fn validate_bulk_descriptor(descriptor: &WaveformBulkDescriptor) -> Result<(), String> {
    validate_bulk_descriptor_minimal(descriptor)?;

    let expected = bulk_descriptor_digest(descriptor)?;
    let observed = descriptor
        .descriptor_hash
        .as_ref()
        .ok_or_else(|| "descriptor_hash is required".to_string())?;
    if !is_hex64(observed) {
        return Err("descriptor_hash must be a 64-hex digest".to_string());
    }
    if observed.to_lowercase() != expected.to_lowercase() {
        return Err("descriptor_hash mismatch".to_string());
    }
    Ok(())
}

fn write_bulk_descriptor_bytes(descriptor: &WaveformBulkDescriptor, include_hash: bool) -> Vec<u8> {
    let integrity = cbor_text_map(vec![
        ("algorithm", cbor_text(&descriptor.integrity.algorithm)),
        ("value", cbor_text(&descriptor.integrity.value)),
    ]);
    let representation = cbor_text_map(vec![
        ("channel", cbor_text(&descriptor.representation.channel)),
        ("downmix", cbor_text(&descriptor.representation.downmix)),
        ("peak", cbor_text(&descriptor.representation.peak)),
        ("resampling", cbor_text(&descriptor.representation.resampling)),
        ("sample_range", cbor_text(&descriptor.representation.sample_range)),
        ("source_timeline", cbor_text(&descriptor.representation.source_timeline)),
        ("time", cbor_text(&descriptor.representation.time)),
    ]);
    let entries = vec![
        (cbor_uint_key(0), cbor_text(&descriptor.version)),
        (cbor_uint_key(1), cbor_text(&descriptor.mode)),
        (cbor_uint_key(2), cbor_text(&descriptor.handle_identity)),
        (cbor_uint_key(3), cbor_text(&descriptor.attempt_id)),
        (cbor_uint_key(4), cbor_text(&descriptor.lease_id)),
        (cbor_uint_key(5), cbor_text(&descriptor.cancel_scope)),
        (cbor_uint_key(6), cbor_text(&descriptor.expiry)),
        (cbor_uint_key(7), integrity),
        (cbor_uint_key(8), cbor_bool(descriptor.chunk_hashes_omitted)),
        (cbor_uint_key(9), cbor_uint(descriptor.length)),
        (cbor_uint_key(10), cbor_text(&descriptor.payload_schema)),
        (cbor_uint_key(11), cbor_text(&descriptor.source_publication_fence)),
        (cbor_uint_key(12), representation),
    ];

    if include_hash {
        let mut entries = entries;
        entries.push((
            cbor_uint_key(13),
            cbor_text(descriptor.descriptor_hash.as_deref().unwrap_or("")),
        ));
        cbor_canonical_map(entries)
    } else {
        cbor_canonical_map(entries)
    }
}

fn validate_cache_key_profile(profile: &WaveformCacheKeyProfile) -> Result<(), String> {
    if profile.cache_key_schema != CACHE_KEY_SCHEMA_ID {
        return Err("cache_key_schema must be the canonical cache-key schema".to_string());
    }
    if profile.operation.id.trim().is_empty() || profile.operation.version.trim().is_empty() {
        return Err("operation.id and operation.version must be explicit".to_string());
    }
    if profile.implementation.id.trim().is_empty() || profile.implementation.version.trim().is_empty() {
        return Err("implementation.id and implementation.version must be explicit".to_string());
    }
    if profile.inputs.is_empty() {
        return Err("at least one input port is required".to_string());
    }
    if profile
        .inputs
        .iter()
        .any(|input| input.order == u64::MAX)
    {
        return Err("invalid input order".to_string());
    }
    if profile.inputs.iter().any(|input| input.kind.trim().is_empty() || input.schema.trim().is_empty() || input.content.trim().is_empty() || input.revision.trim().is_empty() || input.sample_range.trim().is_empty()) {
        return Err("input port fields must be explicit".to_string());
    }

    for field in [
        &profile.parameters.cache_key_schema,
        &profile.parameters.cache_key_seed,
        &profile.parameters.resource_profile_id,
        &profile.parameters.policy_profile_id,
        &profile.parameters.retry_profile_id,
        &profile.representation.peak,
        &profile.representation.time,
        &profile.representation.channel,
        &profile.representation.downmix,
        &profile.representation.resampling,
        &profile.representation.stability,
        &profile.output_contract.artifact_id,
        &profile.output_contract.artifact_version,
        &profile.output_contract.output_schema_id,
    ] {
        if field.trim().is_empty() {
            return Err("cache key binding field must be explicit".to_string());
        }
    }
    if profile.seed.trim().is_empty() {
        return Err("cache key seed must be explicit".to_string());
    }
    if profile.parameters.cache_key_schema != CACHE_KEY_SCHEMA_ID {
        return Err("parameters.cache_key_schema mismatch".to_string());
    }
    if profile.dependencies.cache_key_schema != CACHE_KEY_SCHEMA_ID {
        return Err("dependencies.cache_key_schema mismatch".to_string());
    }
    if !profile.parameters.resource_profile_id.eq(&profile.dependencies.resource_profile)
        || !profile.parameters.policy_profile_id.eq(&profile.dependencies.policy_profile)
        || !profile.parameters.retry_profile_id.eq(&profile.dependencies.retry_profile)
        || profile.dependencies.operation_profile.trim().is_empty()
    {
        return Err("cache profile dependency coupling mismatch".to_string());
    }
    if profile.output_contract.artifact_version.is_empty()
        || profile.output_contract.output_schema_id.is_empty()
    {
        return Err("output contract identity must be explicit".to_string());
    }
    if !matches!(profile.policy.network.as_str(), "denied") {
        return Err("network policy must be denied".to_string());
    }
    if !profile.resource.resource_profile_id.as_str().is_empty() && profile.policy.media_parse {
        return Err("media_parse must be false".to_string());
    }
    Ok(())
}

fn is_hex64(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|v| v.is_ascii_hexdigit())
}

fn cbor_write_text(bytes: &mut Vec<u8>, value: &str) {
    cbor_write_bytes(bytes, value.as_bytes(), 3);
}

fn cbor_write_bool(bytes: &mut Vec<u8>, value: bool) {
    bytes.push(if value { 0xf5 } else { 0xf4 });
}

fn cbor_write_map(bytes: &mut Vec<u8>, length: usize) {
    cbor_write_length(bytes, 5, length as u64);
}

fn cbor_write_array(bytes: &mut Vec<u8>, length: usize) {
    cbor_write_length(bytes, 4, length as u64);
}

fn cbor_write_length(bytes: &mut Vec<u8>, major: u8, length: u64) {
    if length < 24 {
        bytes.push((major << 5) | length as u8);
        return;
    }
    if length < 256 {
        bytes.push((major << 5) | 24);
        bytes.push(length as u8);
        return;
    }
    if length < 65536 {
        bytes.push((major << 5) | 25);
        bytes.push(((length >> 8) & 0xff) as u8);
        bytes.push((length & 0xff) as u8);
        return;
    }
    bytes.push((major << 5) | 26);
    bytes.push(((length >> 24) & 0xff) as u8);
    bytes.push(((length >> 16) & 0xff) as u8);
    bytes.push(((length >> 8) & 0xff) as u8);
    bytes.push((length & 0xff) as u8);
}

fn cbor_write_uint(bytes: &mut Vec<u8>, value: u64) {
    cbor_write_length(bytes, 0, value);
}

fn cbor_write_bytes(bytes: &mut Vec<u8>, payload: &[u8], major: u8) {
    cbor_write_length(bytes, major, payload.len() as u64);
    bytes.extend_from_slice(payload);
}

fn digest_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|value| format!("{value:02x}")).collect()
}

fn cbor_text_map(entries: Vec<(&str, Vec<u8>)>) -> Vec<u8> {
    let keyed = entries
        .into_iter()
        .map(|(key, value)| (cbor_text_key(key), value))
        .collect();
    cbor_canonical_map(keyed)
}

#[cfg(test)]
pub fn test_cbor_text_map(entries: Vec<(&str, Vec<u8>)>) -> Vec<u8> {
    cbor_text_map(entries)
}

fn cbor_text_key(value: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    cbor_write_text(&mut bytes, value);
    bytes
}

fn cbor_text(value: &str) -> Vec<u8> {
    cbor_text_key(value)
}

fn cbor_uint_key(value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    cbor_write_uint(&mut bytes, value);
    bytes
}

fn cbor_uint(value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    cbor_write_uint(&mut bytes, value);
    bytes
}

fn cbor_array(values: Vec<Vec<u8>>) -> Vec<u8> {
    let mut bytes = Vec::new();
    cbor_write_array(&mut bytes, values.len());
    for value in values {
        bytes.extend_from_slice(&value);
    }
    bytes
}

fn cbor_bool(value: bool) -> Vec<u8> {
    let mut bytes = Vec::new();
    cbor_write_bool(&mut bytes, value);
    bytes
}

fn cbor_map_uint(entries: Vec<(u64, Vec<u8>)>) -> Vec<u8> {
    let keyed = entries
        .into_iter()
        .map(|(key, value)| (cbor_uint_key(key), value))
        .collect();
    cbor_canonical_map(keyed)
}

fn cbor_canonical_map(entries: Vec<(Vec<u8>, Vec<u8>)>) -> Vec<u8> {
    let mut entries = entries;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    let mut bytes = Vec::new();
    cbor_write_map(&mut bytes, entries.len());
    for (key, value) in entries {
        bytes.extend_from_slice(&key);
        bytes.extend_from_slice(&value);
    }
    bytes
}

fn encode_cache_input_port(port: &WaveformCacheInputPort) -> Vec<u8> {
    cbor_text_map(vec![
        ("content", cbor_text(&port.content)),
        ("order", cbor_uint(port.order)),
        ("revision", cbor_text(&port.revision)),
        ("sample_range", cbor_text(&port.sample_range)),
        ("schema", cbor_text(&port.schema)),
        ("type", cbor_text(&port.kind)),
    ])
}
