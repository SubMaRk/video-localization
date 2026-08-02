pub mod preflight;
pub mod fingerprint;
pub mod probe_adapter_contract;
pub mod probe_adapter_runtime;
pub mod waveform_contract;

pub use preflight::{
    source_ceiling_bytes,
    is_within_source_ceiling,
    FileMetadataSnapshot,
    PreflightDisposition,
    PreflightResult,
    preflight_local_file,
};
pub use fingerprint::{
    FingerprintAlgorithm,
    FingerprintDisposition,
    FingerprintObservation,
    fingerprint_local_file,
    fingerprint_local_file_with_hook,
    SHA2_CRATE_VERSION,
};
pub use probe_adapter_contract::{
    fixed_probe_argv,
    DISPOSITION_MAP,
    PROBE_ARGV_PREFIX,
    validate_probe_prelaunch,
    validate_probe_output_completeness,
    to_structured_error,
    ControlEnvelopeLimit,
    DispositionMapping,
    ProbeLeaseManifest,
    ErrorRef,
    ProbePrelaunchFailure,
    StructuredError,
    ProbeContainmentEvidence,
    ProbeLaunchPolicy,
    ProbePrelaunchRequest,
    StructuredErrorAppliesTo,
    CONTROL_ENVELOPE_BYTES,
    IMPLEMENTATION_ID,
    IMPLEMENTATION_PROFILE_ID,
    OPERATION_ID,
    CONTROL_PROTOCOL_VERSION,
    OPERATION_VERSION,
    REGISTERED_EXECUTABLE_PATH,
    REGISTERED_EXECUTABLE_SHA256,
    IMPLEMENTED_PROFILE_VERSION,
    ERROR_REF_NAMESPACE,
    ERROR_REF_VERSION,
    STRUCTURED_ERROR_SCHEMA_ID,
    STRUCTURED_ERROR_SCHEMA_VERSION,
};
pub use probe_adapter_runtime::{
    execute_bound_probe,
    execute_bound_probe_with_context,
    parse_probe_observation,
    ProbeLineage,
    ProbeOutputContract,
    ProbeObservation,
    ProbeRuntimeContext,
    ProbeRuntimeResult,
    ProbeRunFailure,
    ProbeRunFailureCode,
    RuntimeContainmentEvidence,
};
