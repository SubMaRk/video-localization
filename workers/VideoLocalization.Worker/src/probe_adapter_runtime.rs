use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use std::collections::HashMap;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{self, Read};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(windows)]
use std::ffi::c_void;
#[cfg(windows)]
use std::mem::size_of;
#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;
#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(windows)]
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject, JOBOBJECT_BASIC_LIMIT_INFORMATION,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
};
#[cfg(windows)]
use windows::core::PCWSTR;

    use crate::{
        fingerprint::fingerprint_local_file_with_hook,
        probe_adapter_contract::{
            fixed_probe_argv, validate_probe_prelaunch, CONTROL_ENVELOPE_BYTES, CONTROL_LIMITS,
            CONTROL_PROTOCOL_VERSION, MAX_SOURCE_STREAMS, OPERATION_ID, OPERATION_VERSION, ProbePrelaunchFailure,
            ProbePrelaunchRequest, REGISTERED_EXECUTABLE_PATH, REGISTERED_EXECUTABLE_SHA256,
            DispositionMapping, DISPOSITION_MAP, ERROR_REF_NAMESPACE, ERROR_REF_VERSION,
            ErrorRef, STRUCTURED_ERROR_SCHEMA_ID, STRUCTURED_ERROR_SCHEMA_VERSION, StructuredError,
            StructuredErrorAppliesTo,
        },
};

pub const PROBE_EXECUTABLE: &str = REGISTERED_EXECUTABLE_PATH;
pub const PROBE_EXECUTABLE_VERSION: &str = "ffprobe version 8.1.2-full_build-www.gyan.dev";
pub const PROBE_EXECUTABLE_CONFIGURATION: &str =
    "configuration: --enable-gpl --enable-version3 --enable-static --disable-w32threads --disable-autodetect --enable-cairo --enable-fontconfig --enable-iconv --enable-gnutls --enable-lcms2 --enable-libxml2 --enable-gmp --enable-bzlib --enable-lzma --enable-libsnappy --enable-zlib --enable-librist --enable-libsrt --enable-libssh --enable-libzmq --enable-avisynth --enable-libbluray --enable-libcaca --enable-libdvdnav --enable-libdvdread --enable-sdl2 --enable-libaribb24 --enable-libaribcaption --enable-libdav1d --enable-libdavs2 --enable-libopenjpeg --enable-libquirc --enable-libuavs3d --enable-libxevd --enable-libzvbi --enable-liboapv --enable-libqrencode --enable-librav1e --enable-libsvtav1 --enable-libvvenc --enable-libwebp --enable-libx264 --enable-libx265 --enable-libxavs2 --enable-libxeve --enable-libxvid --enable-libaom --enable-libjxl --enable-libsvtjpegxs --enable-libvpx --enable-mediafoundation --enable-libass --enable-frei0r --enable-libfreetype --enable-libfribidi --enable-libharfbuzz --enable-liblensfun --enable-libvidstab --enable-libvmaf --enable-libzimg --enable-amf --enable-cuda-llvm --enable-cuvid --enable-dxva2 --enable-d3d11va --enable-d3d12va --enable-ffnvcodec --enable-libvpl --enable-nvdec --enable-nvenc --enable-vaapi --enable-libshaderc --enable-vulkan --enable-libplacebo --enable-opencl --enable-libcdio --enable-openal --enable-libgme --enable-libmodplug --enable-libopenmpt --enable-libopencore-amrwb --enable-libmp3lame --enable-libshine --enable-libtheora --enable-libtwolame --enable-libvo-amrwbenc --enable-libcodec2 --enable-libilbc --enable-libgsm --enable-liblc3 --enable-libopencore-amrnb --enable-libopus --enable-libspeex --enable-libvorbis --enable-ladspa --enable-libbs2b --enable-libflite --enable-libmysofa --enable-librubberband --enable-libsoxr --enable-chromaprint --enable-whisper";
pub const PROBE_STDOUT_BYTES_LIMIT: u64 = CONTROL_ENVELOPE_BYTES;
pub const PROBE_STDERR_BYTES_LIMIT: u64 = CONTROL_ENVELOPE_BYTES;
pub const PROBE_MAX_STREAMS: u64 = MAX_SOURCE_STREAMS as u64;
const VID_002B1_SCHEMA_BUNDLE_SHA256: &str = "f78bf8a7af8616b913d03539eb66ace6bf670f80c685e791521e395396f3397f";
const VID_002B1_RESOURCE_PROFILE_SHA256: &str = "9daf34a362fb007c286ef0cd327984cb3c476a6b6ad853f5fae0d1f32754724b";
const VID_002B1_CONTROL_PROTOCOL_SHA256: &str = "2539250e640517eccc9dccf5feebce6cf9ec89acc26cef9ee08edc9d914d9f02";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeRunFailureCode {
    PublicationLeaseExpired,
    PrelaunchValidationFailed,
    ExecutableIdentityMismatch,
    ExecutableVersionMismatch,
    ExecutableConfigurationMismatch,
    ExecutableAccessFailed,
    SourcePrelaunchFailed,
    SourcePostlaunchMismatch,
    SourceMutationDetected,
    SourceIdentityMismatch,
    SourceLengthMismatch,
    NetworkDenialUnavailable,
    NetworkDenialSetupFailed,
    NetworkDenialVerifyFailed,
    JobObjectUnavailable,
    JobObjectSetupFailed,
    JobObjectVerifyFailed,
    WatchdogUnavailable,
    WatchdogTimeout,
    WatchdogCancelled,
    OutputStdoutOversized,
    OutputStderrOversized,
    ChildLaunchFailed,
    ChildCrashed,
    OutputCaptureFailed,
    JsonMalformed,
    JsonStreamLimitExceeded,
    PublicationFenceMismatch,
    OutputLineageInvalid,
    OutputSerializationFailed,
    ProcessCancellation,
    ArgumentMismatch,
}

#[derive(Debug, Clone)]
pub struct ProbeRunFailure {
    pub code: ProbeRunFailureCode,
    pub message: String,
    pub structured_error: Option<StructuredError>,
}

impl fmt::Display for ProbeRunFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for ProbeRunFailure {}

impl ProbeRunFailure {
    fn bind_request_context(
        mut self,
        request: &ProbePrelaunchRequest,
        publication_ref: Option<&str>,
    ) -> ProbeRunFailure {
        self.structured_error = Some(to_structured_runtime_error(
            &self.code,
            &self.message,
            request,
            publication_ref,
        ));
        self
    }

    fn with_synthetic_request_context(
        mut self,
        error_instance: &str,
        publication_ref: Option<&str>,
    ) -> ProbeRunFailure {
        self.structured_error = Some(to_structured_runtime_error_with_synthetic_identity(
            &self.code,
            &self.message,
            error_instance,
            publication_ref,
        ));
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeContainmentEvidence {
    pub network_denial_rule_in: String,
    pub network_denial_rule_out: String,
    pub network_rule_scope: String,
    pub job_object_established: bool,
    pub watchdog_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeNormalizedObservation {
    pub format: String,
    pub duration_seconds: f64,
    pub streams: u64,
    pub stream_count: u64,
    pub has_subtitles: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeObservation {
    pub observation_id: String,
    pub schema_id: String,
    pub schema_version: String,
    pub probe_observation: ProbeNormalizedObservation,
    pub completeness: String,
    pub artifact_hash: String,
    pub artifact_size: u64,
    pub lineage_ref: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeLineage {
    pub job_ref: String,
    pub attempt_ref: String,
    pub dispatch_ref: String,
    pub worker_ref: String,
    pub publication_ref: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeOutputIntegrity {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeProducing {
    pub job_ref: String,
    pub attempt_ref: String,
    pub dispatch_ref: String,
    pub worker_ref: String,
    pub operation_id: String,
    pub operation_version: String,
    pub implementation_id: String,
    pub implementation_profile_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeResourceInput {
    pub staged_copy_hash: String,
    pub manifest_digest: String,
    pub logical_source_ref: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeParameterDigests {
    pub argv: String,
    pub limits: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeResourceDigests {
    pub resource: String,
    pub schema: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbePolicyDigests {
    pub policy: String,
    pub schema: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeCacheDigests {
    pub schema_bundle: String,
    pub control_protocol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeStaging {
    pub state: String,
    pub path: String,
    pub delete_on_stale: bool,
    pub quarantine_on_failure: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeFencing {
    pub required: bool,
    pub active_token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeOutputContract {
    pub artifact_id: String,
    pub artifact_version: String,
    pub artifact_type: String,
    pub output_schema_id: String,
    pub completeness: String,
    pub integrity: ProbeOutputIntegrity,
    pub size_bytes: u64,
    pub component_inventory: Vec<String>,
    pub producing: ProbeProducing,
    pub input_digests: ProbeResourceInput,
    pub parameter_digests: ProbeParameterDigests,
    pub resource_digests: ProbeResourceDigests,
    pub policy_digests: ProbePolicyDigests,
    pub cache_digests: ProbeCacheDigests,
    pub output_port: String,
    pub staging: ProbeStaging,
    pub lineage: ProbeLineage,
    pub publication_id: String,
    pub fencing: ProbeFencing,
}

#[derive(Debug)]
pub struct ProbeRuntimeResult {
    pub normalized_observation: ProbeObservation,
    pub output_contract: ProbeOutputContract,
    pub containment: RuntimeContainmentEvidence,
    pub captured_stdout: String,
    pub captured_stderr: String,
    pub pre_launch_hash: String,
    pub post_launch_hash: String,
}

#[derive(Clone)]
pub struct ProbeRuntimeContext {
    pub request: Option<ProbePrelaunchRequest>,
    pub schema_digests: HashMap<String, String>,
    pub publication_id: Option<String>,
    pub publication_fence_token: Option<String>,
    pub output_staging_path: Option<String>,
    pub output_artifact_id: Option<String>,
    pub output_port: Option<String>,
    pub watchdog_seconds: Option<u64>,
}

impl ProbeRuntimeContext {
    pub fn new() -> Self {
        Self {
            request: None,
            schema_digests: HashMap::new(),
            publication_id: None,
            publication_fence_token: None,
            output_staging_path: None,
            output_artifact_id: None,
            output_port: None,
            watchdog_seconds: None,
        }
    }

    pub fn with_request(mut self, request: ProbePrelaunchRequest) -> Self {
        self.request = Some(request);
        self
    }

    pub fn with_schema_digests(mut self, digests: HashMap<String, String>) -> Self {
        self.schema_digests = digests;
        self
    }

    pub fn with_publication(mut self, publication_id: impl Into<String>, fence_token: impl Into<String>) -> Self {
        self.publication_id = Some(publication_id.into());
        self.publication_fence_token = Some(fence_token.into());
        self
    }

    pub fn with_output_staging(mut self, staging_path: impl Into<String>) -> Self {
        self.output_staging_path = Some(staging_path.into());
        self
    }

    pub fn with_output_artifact_id(mut self, artifact_id: impl Into<String>) -> Self {
        self.output_artifact_id = Some(artifact_id.into());
        self
    }

    pub fn with_output_port(mut self, output_port: impl Into<String>) -> Self {
        self.output_port = Some(output_port.into());
        self
    }

    pub fn with_watchdog_seconds(mut self, seconds: u64) -> Self {
        self.watchdog_seconds = Some(seconds);
        self
    }
}

#[derive(Clone)]
struct CapturedOutput {
    bytes: Vec<u8>,
    truncated: bool,
}

pub trait ProbeProcessFactory: Send + Sync {
    fn spawn(&self, executable: &str, argv: &[String]) -> io::Result<Child>;
}

#[derive(Default)]
pub struct DefaultProbeProcessFactory;

impl ProbeProcessFactory for DefaultProbeProcessFactory {
    fn spawn(&self, executable: &str, argv: &[String]) -> io::Result<Child> {
        let mut command = Command::new(executable);
        command.args(argv);
        command.stdin(Stdio::null());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command.spawn()
    }
}

#[cfg(windows)]
pub trait NetworkDenialController: Send + Sync {
    fn establish(&self, executable: &str) -> Result<NetworkDenyGuard, ProbeRunFailure>;
}

#[cfg(not(windows))]
pub trait NetworkDenialController: Send + Sync {
    fn establish(&self, _executable: &str) -> Result<NetworkDenyGuard, ProbeRunFailure>;
}

#[derive(Default)]
pub struct DefaultNetworkDenialController;

#[cfg(windows)]
impl NetworkDenialController for DefaultNetworkDenialController {
    fn establish(&self, executable: &str) -> Result<NetworkDenyGuard, ProbeRunFailure> {
        let token = unique_token("vid-impl-002b2-net");
        let rule_in = format!("{token}-in");
        let rule_out = format!("{token}-out");

        add_network_rule(&rule_in, executable, true).map_err(|error| ProbeRunFailure {
            code: ProbeRunFailureCode::NetworkDenialSetupFailed,
            message: format!("failed to create inbound deny rule: {error}"),
            structured_error: None,
        })?;

        if let Err(error) = add_network_rule(&rule_out, executable, false) {
            let _ = remove_network_rule(&rule_in);
            return Err(ProbeRunFailure {
                code: ProbeRunFailureCode::NetworkDenialSetupFailed,
                message: format!("failed to create outbound deny rule: {error}"),
                structured_error: None,
            });
        }

        if !verify_network_rule(&rule_in).map_err(|error| ProbeRunFailure {
            code: ProbeRunFailureCode::NetworkDenialVerifyFailed,
            message: format!("inbound rule lookup failed: {error}"),
            structured_error: None,
        })? {
            let _ = remove_network_rule(&rule_in);
            let _ = remove_network_rule(&rule_out);
            return Err(ProbeRunFailure {
                code: ProbeRunFailureCode::NetworkDenialVerifyFailed,
                message: format!("inbound deny rule not visible: {rule_in}"),
                structured_error: None,
            });
        }

        if !verify_network_rule(&rule_out).map_err(|error| ProbeRunFailure {
            code: ProbeRunFailureCode::NetworkDenialVerifyFailed,
            message: format!("outbound rule lookup failed: {error}"),
            structured_error: None,
        })? {
            let _ = remove_network_rule(&rule_in);
            let _ = remove_network_rule(&rule_out);
            return Err(ProbeRunFailure {
                code: ProbeRunFailureCode::NetworkDenialVerifyFailed,
                message: format!("outbound deny rule not visible: {rule_out}"),
                structured_error: None,
            });
        }

        Ok(NetworkDenyGuard {
            managed: true,
            rule_in,
            rule_out,
            executable: executable.to_string(),
        })
    }
}

#[cfg(not(windows))]
impl NetworkDenialController for DefaultNetworkDenialController {
    fn establish(&self, _executable: &str) -> Result<NetworkDenyGuard, ProbeRunFailure> {
        Err(ProbeRunFailure {
            code: ProbeRunFailureCode::NetworkDenialUnavailable,
            message: "network denial is only implemented on Windows".to_string(),
            structured_error: None,
        })
    }
}

#[derive(Debug)]
pub struct NetworkDenyGuard {
    managed: bool,
    rule_in: String,
    rule_out: String,
    executable: String,
}

impl Drop for NetworkDenyGuard {
    fn drop(&mut self) {
        if self.managed {
            let _ = remove_network_rule(&self.rule_in);
            let _ = remove_network_rule(&self.rule_out);
            self.rule_in.clear();
            self.rule_out.clear();
            self.executable.clear();
        }
    }
}

#[cfg(windows)]
pub trait JobObjectController: Send + Sync {
    fn create(&self, limits: &ProbeChildLimits) -> Result<JobObjectGuard, ProbeRunFailure>;
    fn assign_child(&self, job: &JobObjectGuard, child: &Child) -> Result<(), ProbeRunFailure>;
}

#[cfg(not(windows))]
pub trait JobObjectController: Send + Sync {
    fn create(&self, _limits: &ProbeChildLimits) -> Result<JobObjectGuard, ProbeRunFailure>;
    fn assign_child(&self, _job: &JobObjectGuard, _child: &Child) -> Result<(), ProbeRunFailure>;
}

#[cfg(windows)]
#[derive(Default)]
pub struct DefaultJobObjectController;

#[cfg(windows)]
impl JobObjectController for DefaultJobObjectController {
    fn create(&self, limits: &ProbeChildLimits) -> Result<JobObjectGuard, ProbeRunFailure> {
        let process_cpu_ticks = wall_clock_ticks_to_100ns(limits.child_cpu_seconds);
        let handle = unsafe {
            CreateJobObjectW(None, PCWSTR::null()).map_err(|error| ProbeRunFailure {
                code: ProbeRunFailureCode::JobObjectSetupFailed,
                message: format!("CreateJobObjectW failed: {error}"),
                structured_error: None,
            })?
        };

        let limits_extension = JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
            BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION {
                // Child CPU budget is enforced via process time (100ns ticks); wall-clock remains
                // enforced separately by the watchdog.
                PerProcessUserTimeLimit: process_cpu_ticks,
                PerJobUserTimeLimit: 0,
                ActiveProcessLimit: limits.active_process_limit,
                LimitFlags: windows::Win32::System::JobObjects::JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
                    | windows::Win32::System::JobObjects::JOB_OBJECT_LIMIT_ACTIVE_PROCESS
                    | windows::Win32::System::JobObjects::JOB_OBJECT_LIMIT_JOB_MEMORY
                    | windows::Win32::System::JobObjects::JOB_OBJECT_LIMIT_PROCESS_TIME,
                ..Default::default()
            },
            JobMemoryLimit: usize::try_from(limits.resident_memory_bytes).unwrap_or(usize::MAX),
            ..Default::default()
        };

        unsafe {
            SetInformationJobObject(
                handle,
                windows::Win32::System::JobObjects::JobObjectExtendedLimitInformation,
                &limits_extension as *const _ as *const c_void,
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
            .map_err(|error| ProbeRunFailure {
                code: ProbeRunFailureCode::JobObjectSetupFailed,
                message: format!("SetInformationJobObject failed: {error}"),
                structured_error: None,
            })?;
        }

        Ok(JobObjectGuard { handle })
    }

    fn assign_child(&self, job: &JobObjectGuard, child: &Child) -> Result<(), ProbeRunFailure> {
        let process_handle = HANDLE(child.as_raw_handle() as *mut c_void);
        unsafe {
            AssignProcessToJobObject(job.handle, process_handle).map_err(|error| ProbeRunFailure {
                code: ProbeRunFailureCode::JobObjectSetupFailed,
                message: format!("AssignProcessToJobObject failed: {error}"),
                structured_error: None,
            })?;
        }

        Ok(())
    }
}

#[cfg(not(windows))]
impl JobObjectController for DefaultJobObjectController {
    fn create(&self, _limits: &ProbeChildLimits) -> Result<JobObjectGuard, ProbeRunFailure> {
        Err(ProbeRunFailure {
            code: ProbeRunFailureCode::JobObjectUnavailable,
            message: "job object is only implemented on Windows".to_string(),
            structured_error: None,
        })
    }

    fn assign_child(&self, _job: &JobObjectGuard, _child: &Child) -> Result<(), ProbeRunFailure> {
        Err(ProbeRunFailure {
            code: ProbeRunFailureCode::JobObjectUnavailable,
            message: "job object is only implemented on Windows".to_string(),
            structured_error: None,
        })
    }
}

#[derive(Default)]
pub struct JobObjectGuard {
    handle: HANDLE,
}

impl Drop for JobObjectGuard {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            let _ = unsafe { CloseHandle(self.handle) };
            self.handle = HANDLE::default();
        }
    }
}

impl JobObjectGuard {
    pub fn inert() -> Self {
        Self { handle: HANDLE::default() }
    }

    pub fn handle(&self) -> HANDLE {
        self.handle
    }
}

pub trait WatchdogController: Send + Sync {
    fn establish(&self, timeout_seconds: u64) -> Result<WatchdogGuard, ProbeRunFailure>;
}

#[derive(Default)]
pub struct DefaultWatchdogController;

impl WatchdogController for DefaultWatchdogController {
    fn establish(&self, timeout_seconds: u64) -> Result<WatchdogGuard, ProbeRunFailure> {
        if timeout_seconds == 0 {
            return Err(ProbeRunFailure {
                code: ProbeRunFailureCode::WatchdogUnavailable,
                message: "timeout_seconds must be >0".to_string(),
                structured_error: None,
            });
        }

        Ok(WatchdogGuard { timeout_seconds })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WatchdogGuard {
    timeout_seconds: u64,
}

impl WatchdogGuard {
    pub fn timeout_seconds(&self) -> u64 {
        self.timeout_seconds
    }
}

#[derive(Debug, Clone)]
pub struct ProbeChildLimits {
    pub wall_clock_seconds: u64,
    pub child_cpu_seconds: u64,
    pub resident_memory_bytes: u64,
    pub active_process_limit: u32,
}

#[derive(Debug)]
pub struct ProbeInputEvidenceGuard {
    _handle: Option<std::fs::File>,
}

pub fn execute_bound_probe(context: ProbeRuntimeContext) -> Result<ProbeRuntimeResult, ProbeRunFailure> {
    execute_bound_probe_with_context(
        context,
        &DefaultProbeProcessFactory,
        &DefaultNetworkDenialController,
        &DefaultJobObjectController,
        &DefaultWatchdogController,
        |_| false,
    )
}

pub fn execute_bound_probe_with_context<
    PF,
    NC,
    JC,
    WD,
    C,
> (
    context: ProbeRuntimeContext,
    process_factory: &PF,
    network_controller: &NC,
    job_controller: &JC,
    watchdog_controller: &WD,
    is_cancelled: C,
) -> Result<ProbeRuntimeResult, ProbeRunFailure>
where
    PF: ProbeProcessFactory,
    NC: NetworkDenialController,
    JC: JobObjectController,
    WD: WatchdogController,
    C: FnMut(u64) -> bool,
{
    let request = context.request.ok_or_else(|| ProbeRunFailure {
        code: ProbeRunFailureCode::PrelaunchValidationFailed,
        message: "missing prelaunch request".to_string(),
        structured_error: None,
    }
    .with_synthetic_request_context("error-missing-request", None))?;
    let publication_id = context
        .publication_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ProbeRunFailure {
            code: ProbeRunFailureCode::PrelaunchValidationFailed,
            message: "missing publication_id".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None))?
        .to_string();
    let publication_fence = context
        .publication_fence_token
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ProbeRunFailure {
            code: ProbeRunFailureCode::PublicationFenceMismatch,
            message: "missing publication_fence_token".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None))?
        .to_string();

    let expected_argv = fixed_probe_argv(&request.staged_input_path);
    if request.launch_argv != expected_argv {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::ArgumentMismatch,
            message: "launch_argv is not exact fixed argv".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None));
    }

    validate_probe_prelaunch(&request, &context.schema_digests).map_err(|failure| {
        let mapped = map_prelaunch_failure(&failure);
        let message = format!("prelaunch validation failed: {failure:?}");
        ProbeRunFailure {
            code: mapped,
            message,
            structured_error: None,
        }
        .bind_request_context(&request, None)
    })?;

    verify_registered_executable_identity(&request.executable_path, &request.executable_sha256)
        .map_err(|failure| failure.bind_request_context(&request, None))?;
    verify_registered_executable_metadata(&request.executable_path)
        .map_err(|failure| failure.bind_request_context(&request, None))?;

    let source_lock = hold_source_input_readonly(&request.staged_input_path).map_err(|error| {
        ProbeRunFailure {
            code: ProbeRunFailureCode::SourcePrelaunchFailed,
            message: format!("cannot hold source input read lock: {error}"),
            structured_error: None,
        }
        .bind_request_context(&request, None)
    })?;

    let pre_launch = live_fingerprint(&request.staged_input_path).map_err(|failure| {
        match failure {
            FingerprintFailure::IdentityMismatch => ProbeRunFailure {
                code: ProbeRunFailureCode::SourceIdentityMismatch,
                message: "source identity does not match staged lease prelaunch".to_string(),
                structured_error: None,
            },
            FingerprintFailure::LengthMismatch => ProbeRunFailure {
                code: ProbeRunFailureCode::SourceLengthMismatch,
                message: "source length changed before launch".to_string(),
                structured_error: None,
            },
            FingerprintFailure::Mutation | FingerprintFailure::AccessFailed => ProbeRunFailure {
                code: ProbeRunFailureCode::SourcePrelaunchFailed,
                message: "source prelaunch fingerprint check failed".to_string(),
                structured_error: None,
            },
        }
        .bind_request_context(&request, None)
    })?;
    if pre_launch.0 != request.lease.staged_copy_sha256 || pre_launch.1 != request.lease.staged_copy_length {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::SourceMutationDetected,
            message: "live source hash/length differs from lease before launch".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None));
    }

    let output_artifact_id = context
        .output_artifact_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ProbeRunFailure {
            code: ProbeRunFailureCode::PrelaunchValidationFailed,
            message: "missing output_artifact_id".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None))?
        .to_owned();
    let output_port = context
        .output_port
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ProbeRunFailure {
            code: ProbeRunFailureCode::PrelaunchValidationFailed,
            message: "missing output_port".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None))?
        .to_owned();

    if publication_fence != request.lease.expected_fence_token {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::PublicationFenceMismatch,
            message: "publication fence token mismatch".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None));
    }

    validate_output_port_and_artifact_lineage(&output_port, &output_artifact_id, &request)
        .map_err(|failure| failure.bind_request_context(&request, None))?;

    let network_guard = network_controller
        .establish(&request.executable_path)
        .map_err(|failure| map_containment_failure(failure).bind_request_context(&request, None))?;
    let watchdog_seconds = context
        .watchdog_seconds
        .unwrap_or(CONTROL_LIMITS.wall_clock_seconds);
    let watchdog_guard = watchdog_controller
        .establish(watchdog_seconds)
        .map_err(|failure| map_watchdog_failure(failure).bind_request_context(&request, None))?;

    let limits = ProbeChildLimits {
        wall_clock_seconds: watchdog_seconds,
        child_cpu_seconds: CONTROL_LIMITS.child_cpu_seconds,
        resident_memory_bytes: CONTROL_LIMITS.resident_memory_bytes,
        active_process_limit: 1,
    };
    let job_guard = job_controller
        .create(&limits)
        .map_err(|failure| map_containment_failure(failure).bind_request_context(&request, None))?;

    let mut child = process_factory
        .spawn(&request.executable_path, &expected_argv)
        .map_err(|error| ProbeRunFailure {
            code: ProbeRunFailureCode::ChildLaunchFailed,
            message: format!("spawn failed: {error}"),
            structured_error: None,
        }
        .bind_request_context(&request, None))?;

    if let Err(error) = job_controller.assign_child(&job_guard, &child) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error.bind_request_context(&request, None));
    }

    let stdout_capture = child
        .stdout
        .take()
        .ok_or_else(|| ProbeRunFailure {
            code: ProbeRunFailureCode::OutputCaptureFailed,
            message: "child did not expose stdout".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None))
        .and_then(|stream| {
            spawn_output_reader(stream, PROBE_STDOUT_BYTES_LIMIT).map_err(|error| ProbeRunFailure {
                code: ProbeRunFailureCode::OutputCaptureFailed,
                message: format!("stdout capture failed: {error}"),
                structured_error: None,
            }
            .bind_request_context(&request, None))
        })?;

    let stderr_capture = child
        .stderr
        .take()
        .ok_or_else(|| ProbeRunFailure {
            code: ProbeRunFailureCode::OutputCaptureFailed,
            message: "child did not expose stderr".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None))
        .and_then(|stream| {
            spawn_output_reader(stream, PROBE_STDERR_BYTES_LIMIT).map_err(|error| ProbeRunFailure {
                code: ProbeRunFailureCode::OutputCaptureFailed,
                message: format!("stderr capture failed: {error}"),
                structured_error: None,
            }
            .bind_request_context(&request, None))
        })?;

    let child_status = wait_with_watchdog(&mut child, watchdog_guard.timeout_seconds(), is_cancelled).map_err(|error| {
        let _ = child.kill();
        let _ = child.wait();
        error.bind_request_context(&request, None)
    })?;

    let stdout = stdout_capture.join().map_err(|_| {
        ProbeRunFailure {
            code: ProbeRunFailureCode::OutputCaptureFailed,
            message: "stdout reader thread panicked".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None)
    })?;
    let stderr = stderr_capture.join().map_err(|_| {
        ProbeRunFailure {
            code: ProbeRunFailureCode::OutputCaptureFailed,
            message: "stderr reader thread panicked".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None)
    })?;
    let stdout = stdout.map_err(|error| {
        ProbeRunFailure {
            code: ProbeRunFailureCode::OutputCaptureFailed,
            message: format!("stdout capture failed: {error}"),
            structured_error: None,
        }
        .bind_request_context(&request, None)
    })?;
    let stderr = stderr.map_err(|error| {
        ProbeRunFailure {
            code: ProbeRunFailureCode::OutputCaptureFailed,
            message: format!("stderr capture failed: {error}"),
            structured_error: None,
        }
        .bind_request_context(&request, None)
    })?;

    let stdout_text = String::from_utf8_lossy(&stdout.bytes).into_owned();
    let stderr_text = String::from_utf8_lossy(&stderr.bytes).into_owned();

    if stdout.truncated {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::OutputStdoutOversized,
            message: "ffprobe stdout exceeded 1 MiB".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None));
    }

    if stderr.truncated {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::OutputStderrOversized,
            message: "ffprobe stderr exceeded 1 MiB".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None));
    }

    if !child_status.success() {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::ChildCrashed,
            message: format!("ffprobe exited non-zero: {child_status}"),
            structured_error: None,
        }
        .bind_request_context(&request, None));
    }

    let post_launch = live_fingerprint(&request.staged_input_path).map_err(|failure| {
        match failure {
            FingerprintFailure::IdentityMismatch => ProbeRunFailure {
                code: ProbeRunFailureCode::SourceMutationDetected,
                message: "source identity changed during execution".to_string(),
                structured_error: None,
            },
            FingerprintFailure::LengthMismatch => ProbeRunFailure {
                code: ProbeRunFailureCode::SourceMutationDetected,
                message: "source length changed during execution".to_string(),
                structured_error: None,
            },
            FingerprintFailure::Mutation | FingerprintFailure::AccessFailed => ProbeRunFailure {
                code: ProbeRunFailureCode::SourcePostlaunchMismatch,
                message: "source post-fingerprint check failed".to_string(),
                structured_error: None,
            },
        }
        .bind_request_context(&request, None)
    })?;
    if pre_launch != post_launch {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::SourcePostlaunchMismatch,
            message: "source fingerprint changed across launch window".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None));
    }

    let normalized = parse_probe_observation(&stdout_text).map_err(|failure| {
        failure.bind_request_context(&request, None)
    })?;
    if normalized.streams > PROBE_MAX_STREAMS {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::JsonStreamLimitExceeded,
            message: "stream count exceeds stream ceiling".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None));
    }

    validate_probe_output_completeness("complete").map_err(|failure| {
        ProbeRunFailure {
            code: ProbeRunFailureCode::OutputSerializationFailed,
            message: format!("output completeness rejected: {failure:?}"),
            structured_error: None,
        }
        .bind_request_context(&request, None)
    })?;

    let normalized_json = serde_json::to_string_pretty(&normalized).map_err(|error| {
        ProbeRunFailure {
            code: ProbeRunFailureCode::OutputSerializationFailed,
            message: format!("cannot serialize normalized observation: {error}"),
            structured_error: None,
        }
        .bind_request_context(&request, None)
    })?;
    if normalized_json.len() as u64 > CONTROL_ENVELOPE_BYTES {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::OutputStdoutOversized,
            message: "normalized observation exceeds output envelope".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None));
    }

    let normalized_hash = sha256_hex(normalized_json.as_bytes());
    let output_contract = build_output_contract(
        &request,
        &context.schema_digests,
        &publication_id,
        &publication_fence,
        &normalized_hash,
        normalized_json.len() as u64,
        &output_artifact_id,
        context.output_staging_path,
        &output_port,
        &pre_launch.0,
        &expected_argv,
    )?;

    if output_contract.publication_id != publication_id {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::OutputLineageInvalid,
            message: "output publication id mismatch".to_string(),
            structured_error: None,
        }
        .bind_request_context(&request, None));
    }

    let _input_lock = source_lock;

    Ok(ProbeRuntimeResult {
        normalized_observation: ProbeObservation {
            observation_id: format!("obs-{}", unique_token("vid-impl-002b2")),
            schema_id: "VID-IMPL-P00-002B1-NORMALIZED-OBSERVATION".to_string(),
            schema_version: "1.0.0-p00".to_string(),
            probe_observation: normalized,
            completeness: "complete".to_string(),
            artifact_hash: normalized_hash,
            artifact_size: normalized_json.len() as u64,
            lineage_ref: publication_id,
        },
        output_contract,
        containment: RuntimeContainmentEvidence {
            network_denial_rule_in: network_guard.rule_in.clone(),
            network_denial_rule_out: network_guard.rule_out.clone(),
            network_rule_scope: format!("program={}", network_guard.executable),
            job_object_established: !job_guard.handle().is_invalid(),
            watchdog_seconds: watchdog_guard.timeout_seconds(),
        },
        captured_stdout: sanitize_diagnostics(&stdout_text),
        captured_stderr: sanitize_diagnostics(&stderr_text),
        pre_launch_hash: pre_launch.0,
        post_launch_hash: post_launch.0,
    })
}

fn build_output_contract(
    request: &ProbePrelaunchRequest,
    schema_digests: &HashMap<String, String>,
    publication_id: &str,
    publication_fence: &str,
    integrity_hash: &str,
    integrity_size: u64,
    output_artifact_id: &str,
    output_staging_path: Option<String>,
    output_port: &str,
    staged_copy_sha256: &str,
    argv: &[String],
) -> Result<ProbeOutputContract, ProbeRunFailure> {
    let arg_signature = sha256_hex(argv.join(" ").as_bytes());
    let limits_signature = sha256_hex(
        format!(
            "wall_clock_seconds:{} child_cpu_seconds:{} resident_memory_bytes:{} control_envelope_bytes:{} stream_count:{}",
            CONTROL_LIMITS.wall_clock_seconds,
            CONTROL_LIMITS.child_cpu_seconds,
            CONTROL_LIMITS.resident_memory_bytes,
            CONTROL_LIMITS.control_envelope_bytes,
            CONTROL_LIMITS.stream_count
        )
        .as_bytes(),
    );

    let policy_digest = schema_digests
        .get("VID-IMPL-P00-002B1-POLICY")
        .cloned()
        .unwrap_or_else(|| format!("{:0<64}", "0"));
    let resource_schema_digest = schema_digests
        .get("VID-IMPL-P00-002B1-RESOURCE")
        .cloned()
        .unwrap_or_else(|| format!("{:0<64}", "0"));
    let control_protocol_digest = VID_002B1_CONTROL_PROTOCOL_SHA256.to_string();
    let component_inventory = vec!["probe-adapter".to_string(), "VID-IMPL-P00-002B1".to_string()];

    Ok(ProbeOutputContract {
        artifact_id: output_artifact_id.to_string(),
        artifact_version: "1.0.0-p00".to_string(),
        artifact_type: "probe_observation".to_string(),
        output_schema_id: "VID-IMPL-P00-002B1-NORMALIZED-OBSERVATION".to_string(),
        completeness: "complete".to_string(),
        integrity: ProbeOutputIntegrity {
            algorithm: "sha-256".to_string(),
            value: integrity_hash.to_string(),
        },
        size_bytes: integrity_size,
        component_inventory,
        producing: ProbeProducing {
            job_ref: request.job_id.clone(),
            attempt_ref: request.attempt_id.clone(),
            dispatch_ref: request.dispatch_id.clone(),
            worker_ref: request.worker_instance_id.clone(),
            operation_id: OPERATION_ID.to_string(),
            operation_version: OPERATION_VERSION.to_string(),
            implementation_id: "VID-IMPL-P00-002B1".to_string(),
            implementation_profile_id: "VID-PROBE-P00-001".to_string(),
        },
        input_digests: ProbeResourceInput {
            staged_copy_hash: request.lease.staged_copy_sha256.clone(),
            manifest_digest: request.lease.manifest_digest.clone(),
            logical_source_ref: request.lease.logical_source_ref.clone(),
        },
        parameter_digests: ProbeParameterDigests {
            argv: arg_signature,
            limits: limits_signature,
        },
        resource_digests: ProbeResourceDigests {
            resource: VID_002B1_RESOURCE_PROFILE_SHA256.to_string(),
            schema: resource_schema_digest,
        },
        policy_digests: ProbePolicyDigests {
            policy: policy_fingerprint(),
            schema: policy_digest,
        },
        cache_digests: ProbeCacheDigests {
            schema_bundle: VID_002B1_SCHEMA_BUNDLE_SHA256.to_string(),
            control_protocol: control_protocol_digest,
        },
        output_port: output_port.to_string(),
        staging: ProbeStaging {
            state: "staged-private".to_string(),
            path: output_staging_path.unwrap_or_else(|| request.lease.access_scope.clone()),
            delete_on_stale: true,
            quarantine_on_failure: true,
        },
        lineage: ProbeLineage {
            job_ref: request.job_id.clone(),
            attempt_ref: request.attempt_id.clone(),
            dispatch_ref: request.dispatch_id.clone(),
            worker_ref: request.worker_instance_id.clone(),
            publication_ref: publication_id.to_string(),
        },
        publication_id: publication_id.to_string(),
        fencing: ProbeFencing {
            required: true,
            active_token: publication_fence.to_string(),
        },
    })
}

pub fn parse_probe_observation(raw: &str) -> Result<ProbeNormalizedObservation, ProbeRunFailure> {
    let payload: Value = serde_json::from_str(raw).map_err(|error| ProbeRunFailure {
        code: ProbeRunFailureCode::JsonMalformed,
        message: format!("ffprobe output is not valid JSON: {error}"),
        structured_error: None,
    })?;
    let format = payload.get("format").and_then(Value::as_object).ok_or_else(|| ProbeRunFailure {
        code: ProbeRunFailureCode::JsonMalformed,
        message: "missing format object".to_string(),
        structured_error: None,
    })?;

    let format_name = format
        .get("format_name")
        .and_then(Value::as_str)
        .ok_or_else(|| ProbeRunFailure {
            code: ProbeRunFailureCode::JsonMalformed,
            message: "missing format_name".to_string(),
            structured_error: None,
        })?
        .to_string();
    if format_name.is_empty() {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::JsonMalformed,
            message: "format_name is empty".to_string(),
            structured_error: None,
        });
    }

    let duration_seconds = format
        .get("duration")
        .and_then(parse_f64)
        .ok_or_else(|| ProbeRunFailure {
            code: ProbeRunFailureCode::JsonMalformed,
            message: "missing or invalid duration".to_string(),
            structured_error: None,
        })?;

    let streams = payload
        .get("streams")
        .and_then(Value::as_array)
        .ok_or_else(|| ProbeRunFailure {
            code: ProbeRunFailureCode::JsonMalformed,
            message: "missing streams array".to_string(),
            structured_error: None,
        })?;

    let declared_streams = format
        .get("nb_streams")
        .and_then(parse_u64)
        .or_else(|| Some(streams.len() as u64))
        .unwrap_or(0);
    if declared_streams > PROBE_MAX_STREAMS {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::JsonStreamLimitExceeded,
            message: "declared stream count exceeds stream ceiling".to_string(),
            structured_error: None,
        });
    }

    if declared_streams != streams.len() as u64 {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::JsonMalformed,
            message: "declared stream count does not match stream list".to_string(),
            structured_error: None,
        });
    }

    let has_subtitles = streams.iter().any(|stream| {
        stream
            .get("codec_type")
            .and_then(Value::as_str)
            .is_some_and(|value| value.eq_ignore_ascii_case("subtitle"))
    });

    Ok(ProbeNormalizedObservation {
        format: format_name,
        duration_seconds,
        streams: streams.len() as u64,
        stream_count: declared_streams,
        has_subtitles,
    })
}

pub fn wait_with_watchdog<F: FnMut(u64) -> bool>(
    child: &mut Child,
    timeout_seconds: u64,
    mut is_cancelled: F,
) -> Result<ExitStatus, ProbeRunFailure> {
    let timeout = Duration::from_secs(timeout_seconds.max(1));
    let started = Instant::now();
    let poll_interval = Duration::from_millis(25);

    loop {
        if is_cancelled(started.elapsed().as_secs()) {
            let _ = child.kill();
            let status = child
                .wait()
                .map_err(|error| ProbeRunFailure {
                    code: ProbeRunFailureCode::ChildCrashed,
                    message: format!("wait after cancellation failed: {error}"),
                    structured_error: None,
                })?;
            return Err(ProbeRunFailure {
                code: ProbeRunFailureCode::ProcessCancellation,
                message: format!(
                    "execution cancelled after {} seconds (status: {status})",
                    started.elapsed().as_secs()
                ),
                structured_error: None,
            });
        }

        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {}
            Err(error) => {
                return Err(ProbeRunFailure {
                    code: ProbeRunFailureCode::ChildCrashed,
                    message: format!("child status query failed: {error}"),
                    structured_error: None,
                });
            }
        }

        if started.elapsed() >= timeout {
            let _ = child.kill();
            let status = child
                .wait()
                .map_err(|error| ProbeRunFailure {
                    code: ProbeRunFailureCode::ChildCrashed,
                    message: format!("child wait after timeout failed: {error}"),
                    structured_error: None,
                })?;
            return Err(ProbeRunFailure {
                code: ProbeRunFailureCode::WatchdogTimeout,
                message: format!("watchdog timeout reached, status {status}"),
                structured_error: None,
            });
        }

        thread::sleep(poll_interval);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe_adapter_contract::{
        CONTROL_PROTOCOL_VERSION, EXPECTED_SCHEMA_REFERENCE_IDS, ProbeContainmentEvidence, ProbeLaunchPolicy,
        ProbeLeaseManifest,
    };
    use jsonschema;
    use std::collections::HashMap;
    use std::fs::{self, OpenOptions};
    use std::path::{Path, PathBuf};
    use std::process::Stdio;
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    const TEST_HEX_DIGEST: &str = "11";

    #[derive(Clone, Default)]
    struct CommandProcessFactory {
        command: String,
        args: Vec<String>,
    }

    impl ProbeProcessFactory for CommandProcessFactory {
        fn spawn(&self, _executable: &str, _argv: &[String]) -> io::Result<Child> {
            Command::new(&self.command)
                .args(&self.args)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        }
    }

    #[derive(Clone, Default)]
    struct MockNetworkController {
        deny: bool,
        unavailable: bool,
        verify_failed: bool,
    }

    impl NetworkDenialController for MockNetworkController {
        fn establish(&self, executable: &str) -> Result<NetworkDenyGuard, ProbeRunFailure> {
        if self.unavailable {
            return Err(ProbeRunFailure {
                code: ProbeRunFailureCode::NetworkDenialUnavailable,
                message: "mock unavailable".to_string(),
                structured_error: None,
            });
        }

        if self.deny {
            Err(ProbeRunFailure {
                code: ProbeRunFailureCode::NetworkDenialSetupFailed,
                message: "mock setup failed".to_string(),
                structured_error: None,
            })
        } else {
            if self.verify_failed {
                return Err(ProbeRunFailure {
                    code: ProbeRunFailureCode::NetworkDenialVerifyFailed,
                    message: "mock verify failed".to_string(),
                    structured_error: None,
                });
            }

            Ok(NetworkDenyGuard {
                managed: false,
                rule_in: "mock-in".to_string(),
                rule_out: "mock-out".to_string(),
                executable: executable.to_string(),
                })
            }
        }
    }

    #[derive(Clone, Default)]
    struct MockJobController {
        unavailable: bool,
    }

    impl JobObjectController for MockJobController {
        fn create(&self, _limits: &ProbeChildLimits) -> Result<JobObjectGuard, ProbeRunFailure> {
            if self.unavailable {
                return Err(ProbeRunFailure {
                    code: ProbeRunFailureCode::JobObjectUnavailable,
                    message: "mock unavailable".to_string(),
                    structured_error: None,
                });
            }
            Ok(JobObjectGuard::inert())
        }

        fn assign_child(&self, _job: &JobObjectGuard, _child: &Child) -> Result<(), ProbeRunFailure> {
            if self.unavailable {
                return Err(ProbeRunFailure {
                    code: ProbeRunFailureCode::JobObjectUnavailable,
                    message: "mock unavailable".to_string(),
                    structured_error: None,
                });
            }
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct CapturingJobController {
        unavailable: bool,
        captured: Arc<Mutex<Option<ProbeChildLimits>>>,
    }

    impl CapturingJobController {
        fn captured(&self) -> Option<ProbeChildLimits> {
            self.captured.lock().unwrap_or_else(|error| error.into_inner()).clone()
        }
    }

    impl JobObjectController for CapturingJobController {
        fn create(&self, limits: &ProbeChildLimits) -> Result<JobObjectGuard, ProbeRunFailure> {
            if self.unavailable {
                return Err(ProbeRunFailure {
                    code: ProbeRunFailureCode::JobObjectUnavailable,
                    message: "mock unavailable".to_string(),
                    structured_error: None,
                });
            }
            *self
                .captured
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = Some(limits.clone());
            Ok(JobObjectGuard::inert())
        }

        fn assign_child(&self, _job: &JobObjectGuard, _child: &Child) -> Result<(), ProbeRunFailure> {
            if self.unavailable {
                return Err(ProbeRunFailure {
                    code: ProbeRunFailureCode::JobObjectUnavailable,
                    message: "mock unavailable".to_string(),
                    structured_error: None,
                });
            }
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct MockWatchdogController {
        unavailable: bool,
        timeout: u64,
    }

    impl WatchdogController for MockWatchdogController {
        fn establish(&self, timeout_seconds: u64) -> Result<WatchdogGuard, ProbeRunFailure> {
            if self.unavailable {
                return Err(ProbeRunFailure {
                    code: ProbeRunFailureCode::WatchdogUnavailable,
                    message: "mock unavailable".to_string(),
                    structured_error: None,
                });
            }

            if timeout_seconds == 0 {
                return Err(ProbeRunFailure {
                    code: ProbeRunFailureCode::WatchdogUnavailable,
                    message: "invalid timeout".to_string(),
                    structured_error: None,
                });
            }

            Ok(WatchdogGuard {
                timeout_seconds: self.timeout.max(timeout_seconds),
            })
        }
    }

    fn fixed_schema_digests() -> HashMap<String, String> {
        let mut digests = HashMap::new();
        let digest = format!("{:0<64}", TEST_HEX_DIGEST);
        EXPECTED_SCHEMA_REFERENCE_IDS
            .iter()
            .for_each(|schema| {
                digests.insert((*schema).to_string(), digest.clone());
            });
        digests
    }

    fn request_schema_identity(path: &Path, schema_digests: &HashMap<String, String>) -> Result<ProbePrelaunchRequest, String> {
        let observation = fingerprint_local_file_with_hook(path, None::<fn(u64)>)
            .map_err(|error| format!("fingerprint failed: {error:?}"))?;

        let scope = path
            .parent()
            .and_then(|value| value.to_str())
            .map(ToString::to_string)
            .ok_or_else(|| "path has no parent".to_string())?;

        Ok(ProbePrelaunchRequest {
            operation_id: OPERATION_ID.to_string(),
            operation_version: OPERATION_VERSION.to_string(),
            implementation_id: "VID-IMPL-P00-002B1".to_string(),
            implementation_profile_id: "VID-PROBE-P00-001".to_string(),
            executable_path: REGISTERED_EXECUTABLE_PATH.to_string(),
            executable_sha256: REGISTERED_EXECUTABLE_SHA256.to_string(),
            staged_input_path: path.to_string_lossy().to_string(),
            launch_argv: fixed_probe_argv(path.to_string_lossy().as_ref()),
            launch_policy: ProbeLaunchPolicy {
                use_shell: false,
                use_path_lookup: false,
                user_supplied_options: false,
            },
            schema_digests: schema_digests.clone(),
            capabilities: vec!["cancellation".to_string(), "deadline".to_string()],
            transport: "local-staged-file".to_string(),
            evidence: ProbeContainmentEvidence {
                network_denied: true,
                watchdog_enabled: true,
                job_object_enabled: true,
            },
            output_envelope_bytes: CONTROL_ENVELOPE_BYTES,
            worker_instance_id: "worker-002b2".to_string(),
            dispatch_id: "dispatch-002b2".to_string(),
            control_protocol_version: CONTROL_PROTOCOL_VERSION.to_string(),
            job_id: "job-002b2".to_string(),
            attempt_id: "attempt-002b2".to_string(),
            trace_id: "trace-002b2".to_string(),
            correlation_id: "corr-002b2".to_string(),
            error_instance_id: "err-002b2".to_string(),
            lease: ProbeLeaseManifest {
                lease_id: "lease-002b2".to_string(),
                lease_epoch: 10,
                minimum_acceptable_lease_epoch: 1,
                immutable: true,
                source_sha256: observation.sha256_hex.clone(),
                source_length: observation.byte_length,
                staged_copy_identity: path.to_string_lossy().to_string(),
                staged_copy_sha256: observation.sha256_hex.clone(),
                staged_copy_length: observation.byte_length,
                observed_staged_copy_identity: path.to_string_lossy().to_string(),
                observed_staged_copy_sha256: observation.sha256_hex.clone(),
                observed_staged_copy_length: observation.byte_length,
                access_scope: scope,
                expires_at: "2099-01-01T00:00:00Z".to_string(),
                manifest_digest: format!("{:0<64}", TEST_HEX_DIGEST),
                logical_source_ref: "src-002b2".to_string(),
                observed_fence_token: "pub-fence-002b2".to_string(),
                expected_fence_token: "pub-fence-002b2".to_string(),
            },
        })
    }

    fn runtime_output_port() -> &'static str {
        "probe-observation-output-port"
    }

fn runtime_output_artifact_id(request: &ProbePrelaunchRequest) -> String {
    format!(
            "artifact:job:{}:attempt:{}:dispatch:{}:worker:{}:port:{}",
            request.job_id,
            request.attempt_id,
            request.dispatch_id,
            request.worker_instance_id,
            runtime_output_port()
        )
    }

    fn runtime_context_with_lineage(request: ProbePrelaunchRequest) -> ProbeRuntimeContext {
        let output_port = runtime_output_port().to_string();
        let artifact_id = runtime_output_artifact_id(&request);
        ProbeRuntimeContext::new()
            .with_request(request.clone())
            .with_schema_digests(fixed_schema_digests())
            .with_output_port(output_port)
            .with_output_artifact_id(artifact_id)
            .with_publication(
                format!("pub-{}", unique_token("runtime")),
                request.lease.expected_fence_token.clone(),
            )
    }

    fn assert_runtime_failure_has_structured_error(
        failure: &ProbeRunFailure,
        expected_code: ProbeRunFailureCode,
        request: &ProbePrelaunchRequest,
        expected_publication: Option<&str>,
    ) {
        assert_eq!(failure.code, expected_code);
        let structured = failure
            .structured_error
            .as_ref()
            .expect("runtime failures must carry structured_error");
        assert!(!structured.error_ref.error_instance.trim().is_empty());
        assert_eq!(structured.applies_to.job_ref, request.job_id);
        assert_eq!(structured.applies_to.attempt_ref, request.attempt_id);
        assert_eq!(structured.applies_to.dispatch_ref, request.dispatch_id);
        assert_eq!(structured.applies_to.worker_ref, request.worker_instance_id);
        if let Some(expected) = expected_publication {
            assert_eq!(structured.applies_to.publication_ref.as_deref(), Some(expected));
        } else {
            assert!(
                structured.applies_to.publication_ref.is_none(),
                "pre-publication errors must keep publication_ref absent"
            );
        }
    }

    fn hex_like_64(value: &str) -> bool {
        value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit())
    }

    fn assert_output_contract_matches_output_schema(
        request: &ProbePrelaunchRequest,
        contract: &ProbeOutputContract,
        schema_digests: &HashMap<String, String>,
    ) {
        let contract_value = serde_json::to_value(contract).expect("output contract JSON value");
        let (schema, schema_digest) = output_contract_schema();
        let manifest_output_digest = schema_digests.get("VID-IMPL-P00-002B1-OUTPUT-CONTRACT");
        assert_eq!(
            manifest_output_digest.map(String::as_str),
            Some(schema_digest.as_str()),
            "output schema digest must match schema-bundle manifest"
        );
        assert_output_contract_value_schema_validation(&contract_value, &schema);
        assert!(hex_like_64(schema_digest.as_str()));

        assert_eq!(contract.artifact_type, "probe_observation");
        assert_eq!(contract.completeness, "complete");
        assert_eq!(contract.output_schema_id, "VID-IMPL-P00-002B1-NORMALIZED-OBSERVATION");
        assert!(hex_like_64(&contract.integrity.value));
        assert_eq!(contract.integrity.algorithm, "sha-256");
        assert!(!contract.component_inventory.is_empty());
        assert!(!contract.producing.job_ref.is_empty());
        assert!(!contract.producing.attempt_ref.is_empty());
        assert!(!contract.producing.dispatch_ref.is_empty());
        assert!(!contract.producing.worker_ref.is_empty());
        assert_eq!(contract.producing.operation_id, OPERATION_ID);
        assert_eq!(contract.producing.operation_version, OPERATION_VERSION);
        assert_eq!(contract.producing.implementation_id, "VID-IMPL-P00-002B1");
        assert_eq!(contract.producing.implementation_profile_id, "VID-PROBE-P00-001");

        assert_eq!(contract.input_digests.staged_copy_hash, request.lease.staged_copy_sha256);
        assert_eq!(contract.input_digests.manifest_digest, request.lease.manifest_digest);
        assert_eq!(contract.input_digests.logical_source_ref, request.lease.logical_source_ref);

        assert!(hex_like_64(&contract.parameter_digests.argv));
        assert!(hex_like_64(&contract.parameter_digests.limits));
        assert!(hex_like_64(&contract.resource_digests.resource));
        assert!(hex_like_64(&contract.resource_digests.schema));
        assert!(hex_like_64(&contract.policy_digests.policy));
        assert!(hex_like_64(&contract.policy_digests.schema));
        assert!(hex_like_64(&contract.cache_digests.schema_bundle));
        assert!(hex_like_64(&contract.cache_digests.control_protocol));

        assert_eq!(contract.cache_digests.schema_bundle, VID_002B1_SCHEMA_BUNDLE_SHA256);
        assert_eq!(contract.cache_digests.control_protocol, VID_002B1_CONTROL_PROTOCOL_SHA256);
        assert_eq!(contract.resource_digests.resource, VID_002B1_RESOURCE_PROFILE_SHA256);
        assert_eq!(
            contract.resource_digests.schema,
            *schema_digests
                .get("VID-IMPL-P00-002B1-RESOURCE")
                .unwrap_or(&String::new())
        );
        assert_eq!(
            contract.policy_digests.schema,
            *schema_digests
                .get("VID-IMPL-P00-002B1-POLICY")
                .unwrap_or(&String::new())
        );

        assert!(!contract.lineage.publication_ref.is_empty());
        assert_eq!(contract.publication_id, contract.lineage.publication_ref);
        assert!(!contract.publication_id.is_empty());
        assert!(!contract.fencing.active_token.is_empty());
        assert!(contract.fencing.required);
    }

    fn output_contract_schema() -> (Value, String) {
        let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("Video Localization directory must exist");
        let schema_path = manifest_path.join(
            "contracts/VID-IMPL-P00-002B1/schemas/VID-IMPL-P00-002B1-OUTPUT-CONTRACT.schema.json",
        );
        let schema_text =
            fs::read_to_string(schema_path).expect("output contract schema should exist");
        let schema_digest = sha256_hex(schema_text.as_bytes());
        (serde_json::from_str(&schema_text).expect("output contract schema should parse"), schema_digest)
    }

    fn assert_output_contract_value_schema_validation(contract: &Value, schema: &Value) {
        let validator = jsonschema::draft202012::new(schema).expect("output contract schema should compile");
        let errors: Vec<_> = validator.iter_errors(contract).collect();
        assert!(
            errors.is_empty(),
            "output contract must satisfy schema: {}",
            errors
                .into_iter()
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        );
    }

    fn assert_output_contract_validation_rejects(contract: &mut Value, schema: &Value, failure: &str) {
        let validator = jsonschema::draft202012::new(schema).expect("output contract schema should compile");
        let errors: Vec<_> = validator.iter_errors(contract).collect();
        assert!(
            !errors.is_empty(),
            "output contract mutation should fail: {failure}"
        );
        assert!(
            errors.iter().all(|error| !error.to_string().trim().is_empty()),
            "validator should emit diagnostics for {failure}"
        );
    }

    fn build_success_output_contract_for_schema_validation() -> (ProbeOutputContract, HashMap<String, String>) {
        let staged = temp_path("schema-validation-output", "bin");
        let output_json_path = temp_path("schema-validation-output", "json");
        let output_payload =
            r#"{"format":{"format_name":"matroska","duration":"1.0","nb_streams":"3"},"streams":[{"codec_type":"video"},{"codec_type":"audio"},{"codec_type":"subtitle"}]}"#;
        write_temp_binary(&staged, b"source");
        write_temp_text(&output_json_path, output_payload);

        let schema_digests = schema_digest_map_from_manifest();
        let mut request = request_schema_identity(&staged, &schema_digests).expect("request");
        request.launch_argv = fixed_probe_argv(&staged.to_string_lossy());
        let artifact_id = runtime_output_artifact_id(&request);
        let publication_id = "pub-002b2";
        let context = ProbeRuntimeContext::new()
            .with_request(request)
            .with_schema_digests(schema_digests.clone())
            .with_watchdog_seconds(10)
            .with_output_artifact_id(artifact_id.clone())
            .with_output_port(runtime_output_port().to_string())
            .with_publication(publication_id, "pub-fence-002b2");

        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), format!("type {}", output_json_path.to_string_lossy())],
        };

        let result = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect("runtime should succeed");

        let _ = fs::remove_file(staged);
        let _ = fs::remove_file(output_json_path);
        (result.output_contract, schema_digests)
    }

    fn mutate_and_assert_schema_failure_for_root_field(
        contract: &Value,
        schema: &Value,
        path: &str,
        replacement: Value,
        failure: &str,
    ) {
        let mut contract = contract.clone();
        let mut cursor = &mut contract;
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return;
        }
        for (index, part) in parts[..parts.len() - 1].iter().enumerate() {
            let obj = cursor
                .as_object_mut()
                .expect(&format!("mutation path {path} expects object at segment {index}"));
            cursor = obj
                .entry((*part).to_string())
                .or_insert_with(|| panic!("mutation path {path} missing segment {part}"));
        }
        let parent = cursor
            .as_object_mut()
            .expect(&format!("mutation path {path} expects final parent as object"));
        parent.insert(parts[parts.len() - 1].to_string(), replacement);
        assert_output_contract_validation_rejects(&mut contract, schema, failure);
    }

    fn output_contract_mutation_rejects_path_removal(
        contract: &Value,
        schema: &Value,
        path: &str,
        failure: &str,
    ) {
        let mut contract = contract.clone();
        let mut cursor = &mut contract;
        let parts: Vec<&str> = path.split('.').collect();
        assert!(!parts.is_empty(), "mutation path must be non-empty");

        if parts.len() == 1 {
            let removed = cursor
                .as_object_mut()
                .expect("root object required for contract mutation")
                .remove(parts[0]);
            assert!(removed.is_some(), "path target exists");
            assert_output_contract_validation_rejects(&mut contract, schema, failure);
            return;
        }

        for (index, part) in parts[..parts.len() - 1].iter().enumerate() {
            let obj = cursor
                .as_object_mut()
                .expect(&format!("mutation path {path} expects object at segment {index}"));
            cursor = obj
                .entry((*part).to_string())
                .or_insert_with(|| panic!("mutation path {path} missing segment {part}"));
        }

        let parent = cursor
            .as_object_mut()
            .expect(&format!("mutation path {path} expects final parent as object"));
        let removed = parent.remove(parts[parts.len() - 1]);
        assert!(removed.is_some(), "path target exists");
        assert_output_contract_validation_rejects(&mut contract, schema, failure);
    }

    fn mutation_target_paths_with_additional_properties_false(schema: &Value) -> Vec<String> {
        fn walk(path: &str, value: &Value, out: &mut Vec<String>) {
            let obj = match value {
                Value::Object(map) => map,
                _ => return,
            };
            let is_object = matches!(obj.get("type"), Some(Value::String(value_type)) if value_type == "object");
            let has_additional_false = matches!(obj.get("additionalProperties"), Some(Value::Bool(false)));
            if is_object && has_additional_false {
                if !path.is_empty() {
                    out.push(path.trim_end_matches('.').to_string());
                }
            }

            if let Some(properties) = obj.get("properties").and_then(Value::as_object) {
                for (key, child) in properties {
                    let child_path = if path.is_empty() {
                        key.to_string()
                    } else {
                        format!("{path}.{key}")
                    };
                    walk(&child_path, child, out);
                }
            }
        }

        let mut paths = Vec::new();
        walk("", schema, &mut paths);
        paths
    }

    fn schema_enforces_additional_properties_false(schema: &Value, path: &str) -> bool {
        let mut cursor = schema;
        if path.is_empty() {
            return cursor
                .get("additionalProperties")
                .and_then(Value::as_bool)
                .unwrap_or(false);
        }
        for segment in path.split('.') {
            cursor = cursor
                .get("properties")
                .and_then(Value::as_object)
                .and_then(|properties| properties.get(segment))
                .unwrap_or_else(|| panic!("schema path {path} missing {segment}"));
        }
        cursor
            .get("additionalProperties")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    fn with_closed_nested_unknown_property_checks(
        contract: &ProbeOutputContract,
        schema: &Value,
        closed_paths: &[String],
    ) {
        let contract_value = serde_json::to_value(contract).expect("output contract JSON value");
        for object_path in closed_paths {
            assert!(schema_enforces_additional_properties_false(schema, object_path));
            let mut mutated = contract_value.clone();
            let mut cursor = mutated.as_object_mut().expect("root contract object");
            let segments: Vec<&str> = object_path.split('.').collect();
            for (index, segment) in segments.iter().enumerate() {
                cursor = cursor
                    .get_mut(*segment)
                    .and_then(Value::as_object_mut)
                    .unwrap_or_else(|| {
                        panic!("contract object missing segment {segment} at {index} for {object_path}")
                    });
            }
            cursor.insert("unknown_field".to_string(), Value::String("not-allowed".to_string()));
            assert_output_contract_validation_rejects(&mut mutated, schema, &format!(
                "unknown nested property should fail under additionalProperties:false at {object_path}"
            ));
        }
    }

    fn make_output_contract_schema_validation_base() -> (
        ProbeOutputContract,
        Value,
        Value,
        HashMap<String, String>,
    ) {
        let (contract, schema_digests) = build_success_output_contract_for_schema_validation();
        let contract_value = serde_json::to_value(&contract).expect("output contract JSON value");
        let (schema, schema_digest) = output_contract_schema();
        let manifest_output_digest = schema_digests.get("VID-IMPL-P00-002B1-OUTPUT-CONTRACT");
        assert_eq!(
            manifest_output_digest.map(String::as_str),
            Some(schema_digest.as_str()),
            "output schema digest must match schema-bundle manifest"
        );
        (contract, contract_value, schema, schema_digests)
    }

    fn schema_digest_map_from_manifest() -> HashMap<String, String> {
        let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("Video Localization directory must exist")
            .to_string_lossy()
            .into_owned();
        let manifest_text = fs::read_to_string(
            Path::new(&manifest_path).join("contracts/VID-IMPL-P00-002B1/schema-bundle.json"),
        )
        .expect("schema bundle manifest should exist");
        let manifest_json: serde_json::Value = serde_json::from_str(&manifest_text).expect("schema bundle should parse");
        let mut digests = HashMap::new();
        if let Some(schema_records) = manifest_json.get("schema_records").and_then(|value| value.as_array()) {
            for schema in schema_records {
                let schema_id = schema
                    .get("schema_id")
                    .and_then(Value::as_str)
                    .expect("schema id present");
                let digest = schema
                    .get("sha256")
                    .and_then(Value::as_str)
                    .expect("schema digest present");
                digests.insert(schema_id.to_string(), digest.to_ascii_lowercase());
            }
        }
        digests
    }

    fn temp_path(prefix: &str, ext: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("monotonic clock")
            .as_nanos();
        path.push(format!("vid-impl-002b2-{prefix}-{nanos}.{ext}"));
        path
    }

    fn write_temp_text(path: &Path, content: &str) {
        fs::write(path, content).expect("write temp file");
    }

    fn write_temp_binary(path: &Path, content: &[u8]) {
        fs::write(path, content).expect("write temp binary");
    }

    #[test]
    fn bounded_parse_rejects_malformed_probe_json() {
        let malformed = r#"{"format": "bad""#;
        assert!(parse_probe_observation(malformed).is_err());
    }

    #[test]
    fn bounded_parse_rejects_stream_limit_overflow() {
        let too_many = r#"{"format":{"format_name":"matroska","duration":"1.0","nb_streams":"257"},"streams":[]}"#;
        let error = parse_probe_observation(too_many).expect_err("should reject stream-count ceiling");
        assert_eq!(error.code, ProbeRunFailureCode::JsonStreamLimitExceeded);
    }

    #[test]
    fn source_lock_prevents_mutation_and_relink_races() {
        let path = temp_path("lock", "bin");
        write_temp_binary(&path, b"vid-impl-002b2-source-lock");
        let guard = hold_source_input_readonly(
            path.to_str().expect("path string"),
        )
        .expect("open read lock");

        assert!(OpenOptions::new()
            .write(true)
            .open(&path)
            .is_err());

        let moved = temp_path("lock-target", "bin");
        assert!(fs::rename(&path, &moved).is_err());
        assert!(fs::remove_file(&path).is_err());

        drop(guard);

        let mut rename_ok = false;
        for _ in 0..10 {
            if fs::rename(&path, &moved).is_ok() {
                rename_ok = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        assert!(rename_ok, "rename after source-lock drop should eventually succeed");
        fs::remove_file(&moved).expect("cleanup");
    }

    #[test]
    fn execute_probe_passes_active_process_and_cpu_time_limits_to_job_object() {
        let staged = temp_path("job-object-limits", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&staged, &schema_digests).expect("request");
        let publication_id = "pub-002b2";
        let capture = CapturingJobController::default();
        let context = runtime_context_with_lineage(request.clone())
            .with_publication(publication_id, request.lease.expected_fence_token.clone())
            .with_watchdog_seconds(10);
        let expected_json = r#"{"format":{"format_name":"matroska","duration":"1.0","nb_streams":"1"},"streams":[{"codec_type":"video"}]}"#;
        let output_json_path = temp_path("job-object-limits-output", "json");
        write_temp_text(&output_json_path, expected_json);
        let command = output_json_path.to_string_lossy().to_string();
        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), format!("type {command}")],
        };

        let result = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &capture,
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect("runtime should succeed with captured job limits");

        let captured_limits = capture
            .captured()
            .expect("job limits captured before spawn");
        assert_eq!(captured_limits.active_process_limit, 1);
        assert_eq!(captured_limits.child_cpu_seconds, CONTROL_LIMITS.child_cpu_seconds);

        assert_eq!(result.output_contract.publication_id, publication_id);
        let _ = fs::remove_file(staged);
        let _ = fs::remove_file(output_json_path);
    }

    #[test]
    fn execute_probe_success_with_mocked_containers_and_valid_json() {
        let staged = temp_path("fixture-success", "json");
        let output_json_path = temp_path("fixture-output", "json");
        let output_payload = r#"{"format":{"format_name":"matroska","duration":"1.0","nb_streams":"3"},"streams":[{"codec_type":"video"},{"codec_type":"audio"},{"codec_type":"subtitle"}]}"#;
        write_temp_text(&staged, "probe-source-placeholder");
        write_temp_text(&output_json_path, output_payload);

        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&staged, &schema_digests).expect("request");
        let artifact_id = runtime_output_artifact_id(&request);
        let publication_id = "pub-002b2";
        let context = runtime_context_with_lineage(request.clone())
            .with_publication(publication_id, request.lease.expected_fence_token.clone())
            .with_output_artifact_id(artifact_id.clone())
            .with_watchdog_seconds(10);

        let command = output_json_path.to_string_lossy().to_string();
        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), format!("type {command}")],
        };

        let result = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect("mocked runtime should succeed");

        assert_eq!(result.normalized_observation.probe_observation.format, "matroska");
        assert_eq!(result.normalized_observation.probe_observation.stream_count, 3);
        assert_eq!(result.normalized_observation.probe_observation.streams, 3);
        assert!(result.normalized_observation.probe_observation.has_subtitles);
        assert_eq!(result.output_contract.publication_id, "pub-002b2");
        assert_eq!(result.output_contract.artifact_id, artifact_id);
        assert!(result.captured_stdout.contains("format"));
        assert_output_contract_matches_output_schema(
            &request,
            &result.output_contract,
            &schema_digests,
        );

        let _ = fs::remove_file(staged);
        let _ = fs::remove_file(output_json_path);
    }

    #[test]
    fn execute_probe_fails_without_publication_id() {
        let staged = temp_path("missing-publication", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&staged, &schema_digests).expect("request");
        let context = ProbeRuntimeContext::new()
            .with_request(request.clone())
            .with_schema_digests(schema_digests)
            .with_output_staging(temp_path("missing-publication", "tmp").to_string_lossy().to_string())
            .with_output_artifact_id(runtime_output_artifact_id(&request))
            .with_output_port(runtime_output_port().to_string());

        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), "exit 0".to_string()],
        };

        let error = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("missing publication_id should fail");

        assert_runtime_failure_has_structured_error(
            &error,
            ProbeRunFailureCode::PrelaunchValidationFailed,
            &request,
            None,
        );
        assert_eq!(error.message, "missing publication_id");
        let _ = fs::remove_file(staged);
    }

    #[test]
    fn execute_probe_fails_without_publication_fence_token() {
        let staged = temp_path("missing-fence", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&staged, &schema_digests).expect("request");
        let context = ProbeRuntimeContext::new()
            .with_request(request.clone())
            .with_schema_digests(schema_digests)
            .with_publication("pub-002b2", "")
            .with_output_staging(temp_path("missing-fence", "tmp").to_string_lossy().to_string())
            .with_output_artifact_id(runtime_output_artifact_id(&request))
            .with_output_port(runtime_output_port().to_string());

        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), "exit 0".to_string()],
        };

        let error = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("missing publication_fence_token should fail");

        assert_runtime_failure_has_structured_error(
            &error,
            ProbeRunFailureCode::PublicationFenceMismatch,
            &request,
            None,
        );
        assert_eq!(error.message, "missing publication_fence_token");
        let _ = fs::remove_file(staged);
    }

    #[test]
    fn execute_probe_output_contract_matches_002b1_schema_digests() {
        let staged = temp_path("fixture-contract", "bin");
        let output_json_path = temp_path("fixture-output", "json");
        let output_payload = r#"{"format":{"format_name":"matroska","duration":"1.0","nb_streams":"3"},"streams":[{"codec_type":"video"},{"codec_type":"audio"},{"codec_type":"subtitle"}]}"#;
        write_temp_binary(&staged, b"source");
        write_temp_text(&output_json_path, output_payload);

        let schema_digests = schema_digest_map_from_manifest();
        let mut request = request_schema_identity(&staged, &schema_digests).expect("request");
        request.launch_argv = fixed_probe_argv(&staged.to_string_lossy());
        let artifact_id = runtime_output_artifact_id(&request);
        let publication_id = "pub-002b2";
        let request_for_error_assertion = request.clone();
        let context = ProbeRuntimeContext::new()
            .with_request(request)
            .with_schema_digests(schema_digests.clone())
            .with_watchdog_seconds(10)
            .with_output_artifact_id(artifact_id.clone())
            .with_output_port(runtime_output_port().to_string())
            .with_publication(publication_id, request_for_error_assertion.lease.expected_fence_token.clone());

        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), format!("type {}", output_json_path.to_string_lossy())],
        };

        let result = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect("runtime should succeed");

        let serialized = serde_json::to_string_pretty(&result.output_contract)
            .expect("output contract should serialize");
        let _value: serde_json::Value =
            serde_json::from_str(&serialized).expect("output contract is valid JSON");
        assert_output_contract_matches_output_schema(
            &request_for_error_assertion,
            &result.output_contract,
            &schema_digests,
        );

        let _ = fs::remove_file(staged);
        let _ = fs::remove_file(output_json_path);
    }

    #[test]
    fn execute_probe_output_contract_matches_output_schema_via_draft2020_12_validator() {
        let (contract, contract_value, schema, _schema_digests) = make_output_contract_schema_validation_base();
        assert_output_contract_value_schema_validation(&contract_value, &schema);
        assert_eq!(contract.artifact_type, "probe_observation");
        assert_eq!(contract.completeness, "complete");
    }

    #[test]
    fn output_contract_schema_rejects_required_field_mutations() {
        let (contract, _contract_value, schema, _schema_digests) =
            make_output_contract_schema_validation_base();
        let base = serde_json::to_value(&contract).expect("output contract should serialize into JSON value");
        let root_required_fields = [
            "artifact_id",
            "artifact_version",
            "artifact_type",
            "output_schema_id",
            "completeness",
            "integrity",
            "size_bytes",
            "component_inventory",
            "producing",
            "input_digests",
            "parameter_digests",
            "resource_digests",
            "policy_digests",
            "cache_digests",
            "output_port",
            "staging",
            "lineage",
            "publication_id",
            "fencing",
        ];
        for path in root_required_fields {
            output_contract_mutation_rejects_path_removal(
                &base,
                &schema,
                path,
                &format!("missing root field {path}"),
            );
        }

        let nested_required_paths = [
            "integrity.algorithm",
            "integrity.value",
            "producing.job_ref",
            "producing.attempt_ref",
            "producing.dispatch_ref",
            "producing.worker_ref",
            "producing.operation_id",
            "producing.operation_version",
            "producing.implementation_id",
            "producing.implementation_profile_id",
            "input_digests.staged_copy_hash",
            "input_digests.manifest_digest",
            "input_digests.logical_source_ref",
            "parameter_digests.argv",
            "parameter_digests.limits",
            "resource_digests.resource",
            "resource_digests.schema",
            "policy_digests.policy",
            "policy_digests.schema",
            "cache_digests.schema_bundle",
            "cache_digests.control_protocol",
            "staging.state",
            "staging.path",
            "staging.delete_on_stale",
            "staging.quarantine_on_failure",
            "lineage.job_ref",
            "lineage.attempt_ref",
            "lineage.dispatch_ref",
            "lineage.worker_ref",
            "lineage.publication_ref",
            "fencing.required",
            "fencing.active_token",
        ];
        for path in nested_required_paths {
            output_contract_mutation_rejects_path_removal(
                &base,
                &schema,
                path,
                &format!("missing nested field {path}"),
            );
        }
    }

    #[test]
    fn output_contract_schema_rejects_unknown_properties() {
        let (contract, _contract_value, schema, _schema_digests) = make_output_contract_schema_validation_base();
        let mut contract_value = serde_json::to_value(&contract)
            .expect("output contract should serialize into JSON value");
        contract_value
            .as_object_mut()
            .expect("output contract is object")
            .insert("unexpected_root_property".to_string(), Value::String("closed".to_string()));
        assert_output_contract_validation_rejects(
            &mut contract_value,
            &schema,
            "unknown root property should fail under additionalProperties:false",
        );

        let closed_paths = mutation_target_paths_with_additional_properties_false(&schema);
        with_closed_nested_unknown_property_checks(&contract, &schema, &closed_paths);
    }

    #[test]
    fn output_contract_schema_rejects_malformed_digest_and_violation_classes() {
        let (contract, _contract_value, schema, _schema_digests) = make_output_contract_schema_validation_base();
        let base = serde_json::to_value(&contract).expect("output contract should serialize into JSON value");

        let digest_mutations = [
            ("integrity.value", Value::String("short".to_string())),
            ("integrity.value", Value::String("z".repeat(64))),
            ("input_digests.staged_copy_hash", Value::String("not-a-hex-string".to_string())),
            ("input_digests.manifest_digest", Value::String("123456".to_string())),
            ("parameter_digests.argv", Value::String("not_hex_".to_string())),
            ("parameter_digests.limits", Value::String("".to_string())),
            ("resource_digests.resource", Value::String("G".repeat(64))),
            ("resource_digests.schema", Value::String("".to_string())),
            ("policy_digests.policy", Value::String("".to_string())),
            ("policy_digests.schema", Value::String("z".repeat(128))),
            ("cache_digests.schema_bundle", Value::String("00ZZ".to_string())),
            ("cache_digests.control_protocol", Value::String("".to_string())),
        ];
        for (path, value) in digest_mutations {
            mutate_and_assert_schema_failure_for_root_field(
                &base,
                &schema,
                path,
                value,
                &format!("malformed digest at {path}"),
            );
        }

        let enum_and_type_mutations = [
            ("artifact_type", Value::String("audio_packet".to_string())),
            ("completeness", Value::String("partial".to_string())),
            ("integrity.algorithm", Value::String("md5".to_string())),
            ("size_bytes", Value::String("not-an-integer".to_string())),
            ("size_bytes", Value::from(-1)),
            ("component_inventory", Value::String("unexpected".to_string())),
            ("staging", Value::Object(serde_json::Map::from_iter(vec![(
                "state".to_string(),
                Value::Bool(true),
            )]))),
            ("lineage.publication_ref", Value::Bool(false)),
            ("publication_id", Value::Object(serde_json::Map::new())),
        ];
        for (path, value) in enum_and_type_mutations {
            mutate_and_assert_schema_failure_for_root_field(
                &base,
                &schema,
                path,
                value,
                &format!("schema violation at {path}"),
            );
        }

        // Keep coverage for minItems on component_inventory.
        let mut contract_value = base;
        contract_value["component_inventory"] = Value::Array(vec![]);
        assert_output_contract_validation_rejects(
            &mut contract_value,
            &schema,
            "component_inventory minItems:1",
        );
    }

    #[test]
    fn execute_probe_fails_when_output_artifact_id_is_missing() {
        let staged = temp_path("missing-artifact", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&staged, &schema_digests).expect("request");
        let publication_id = "pub-002b2";
        let context = runtime_context_with_lineage(request.clone())
            .with_publication(publication_id, request.lease.expected_fence_token.clone())
            .with_output_artifact_id("")
            .with_watchdog_seconds(10);

        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), "echo {}".to_string()],
        };

        let error = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("missing output artifact id should fail");

        assert_runtime_failure_has_structured_error(
            &error,
            ProbeRunFailureCode::PrelaunchValidationFailed,
            &request,
            None,
        );
        assert_eq!(error.message, "missing output_artifact_id");
        let _ = fs::remove_file(staged);
    }

    #[test]
    fn execute_probe_fails_when_output_artifact_id_is_not_bound_to_lineage() {
        let staged = temp_path("unbound-artifact", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&staged, &schema_digests).expect("request");
        let publication_id = "pub-002b2";
        let context = runtime_context_with_lineage(request.clone())
            .with_publication(publication_id, request.lease.expected_fence_token.clone())
            .with_output_artifact_id("artifact-not-bound")
            .with_output_port("probe-observation-output-port")
            .with_watchdog_seconds(10);

        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), "echo {}".to_string()],
        };

        let error = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("unbound artifact id should fail");

        assert_runtime_failure_has_structured_error(
            &error,
            ProbeRunFailureCode::OutputLineageInvalid,
            &request,
            None,
        );
        let _ = fs::remove_file(staged);
    }

    #[test]
    fn execute_probe_fails_when_output_port_is_missing() {
        let staged = temp_path("missing-output-port", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&staged, &schema_digests).expect("request");
        let publication_id = "pub-002b2";
        let context = ProbeRuntimeContext::new()
            .with_request(request.clone())
            .with_schema_digests(schema_digests)
            .with_publication(publication_id, request.lease.expected_fence_token.clone())
            .with_output_artifact_id(runtime_output_artifact_id(&request))
            .with_watchdog_seconds(10);

        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), "echo {}".to_string()],
        };

        let error = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("missing output port should fail");

        assert_runtime_failure_has_structured_error(
            &error,
            ProbeRunFailureCode::PrelaunchValidationFailed,
            &request,
            None,
        );
        assert_eq!(error.message, "missing output_port");
        let _ = fs::remove_file(staged);
    }

    #[test]
    fn execute_probe_fails_on_executable_identity_mismatch() {
        let staged = temp_path("mismatch", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let mut request = request_schema_identity(&staged, &schema_digests).expect("request");
        request.executable_path = "C:\\ProgramData\\chocolatey\\lib\\ffmpeg-full\\tools\\ffmpeg\\bin\\ffmpeg.exe".to_string();
        let publication_id = "pub-002b2";
        let context = runtime_context_with_lineage(request.clone())
            .with_publication(publication_id, request.lease.expected_fence_token.clone());

        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), "exit 0".to_string()],
        };

        let error = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("identity mismatch should fail");

        assert_runtime_failure_has_structured_error(
            &error,
            ProbeRunFailureCode::ExecutableIdentityMismatch,
            &request,
            None,
        );
        let _ = fs::remove_file(staged);
    }

    #[test]
    fn execute_probe_fails_when_launch_argv_is_not_fixed() {
        let staged = temp_path("launch-argv", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let mut request = request_schema_identity(&staged, &schema_digests).expect("request");
        request.launch_argv = vec!["-show_entries".to_string(), "format".to_string()];

        let publication_id = "pub-002b2";
        let context = runtime_context_with_lineage(request.clone())
            .with_publication(publication_id, request.lease.expected_fence_token.clone());

        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), "echo {} | type con".to_string()],
        };

        let error = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("non-fixed argv should fail");

        assert_runtime_failure_has_structured_error(
            &error,
            ProbeRunFailureCode::ArgumentMismatch,
            &request,
            None,
        );
        let _ = fs::remove_file(staged);
    }

    #[test]
    fn execute_probe_fails_on_stale_lease() {
        let staged = temp_path("stale", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let mut request = request_schema_identity(&staged, &schema_digests).expect("request");
        request.lease.lease_epoch = 0;
        request.lease.minimum_acceptable_lease_epoch = 1;

        let publication_id = "pub-002b2";
        let context = runtime_context_with_lineage(request.clone())
            .with_publication(publication_id, request.lease.expected_fence_token.clone());

        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), "exit 0".to_string()],
        };

        let error = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("stale lease should fail");

        assert_runtime_failure_has_structured_error(
            &error,
            ProbeRunFailureCode::PublicationFenceMismatch,
            &request,
            None,
        );
        let _ = fs::remove_file(staged);
    }

    #[test]
    fn execute_probe_fails_when_publication_fence_changes() {
        let staged = temp_path("fence", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let mut request = request_schema_identity(&staged, &schema_digests).expect("request");
        request.lease.expected_fence_token = "expected-token".to_string();
        request.lease.observed_fence_token = "observed-token".to_string();

        let publication_id = "pub-002b2";
        let context = runtime_context_with_lineage(request.clone()).with_publication(publication_id, "mismatch-token");

        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), "exit 0".to_string()],
        };

        let error = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("fence mismatch should fail");

        assert_runtime_failure_has_structured_error(
            &error,
            ProbeRunFailureCode::PublicationFenceMismatch,
            &request,
            None,
        );
        let _ = fs::remove_file(staged);
    }

    #[test]
    fn execute_probe_fails_when_source_changes_before_spawn() {
        let staged = temp_path("source-prelaunch-mutation", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&staged, &schema_digests).expect("request");

        write_temp_binary(&staged, b"mutated before launch");
        let request = request;

        let publication_id = "pub-002b2";
        let context = runtime_context_with_lineage(request.clone())
            .with_publication(publication_id, request.lease.expected_fence_token.clone());
        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), "exit 0".to_string()],
        };

        let error = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("source mutation before launch should fail");

        assert_runtime_failure_has_structured_error(
            &error,
            ProbeRunFailureCode::SourceMutationDetected,
            &request,
            None,
        );
        let _ = fs::remove_file(staged);
    }

    #[test]
    fn execute_probe_fails_when_network_containment_unavailable() {
        let staged = temp_path("network", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&staged, &schema_digests).expect("request");
        let publication_id = "pub-002b2";
        let context = runtime_context_with_lineage(request.clone())
            .with_publication(publication_id, request.lease.expected_fence_token.clone());
        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), "exit 0".to_string()],
        };
        let error = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController {
                unavailable: true,
                ..Default::default()
            },
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("network unavailability should fail");
        assert_runtime_failure_has_structured_error(
            &error,
            ProbeRunFailureCode::NetworkDenialUnavailable,
            &request,
            None,
        );
        let _ = fs::remove_file(staged);
    }

    #[test]
    fn execute_probe_fails_when_network_verification_fails() {
        let staged = temp_path("network-verify", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&staged, &schema_digests).expect("request");
        let publication_id = "pub-002b2";
        let context = runtime_context_with_lineage(request.clone())
            .with_publication(publication_id, request.lease.expected_fence_token.clone());
        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), "exit 0".to_string()],
        };

        let error = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController {
                verify_failed: true,
                ..Default::default()
            },
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("network verification failure should fail distinctly from setup");
        assert_runtime_failure_has_structured_error(
            &error,
            ProbeRunFailureCode::NetworkDenialVerifyFailed,
            &request,
            None,
        );
        let _ = fs::remove_file(staged);
    }

    #[test]
    fn prelaunch_and_runtime_codes_keep_expired_lease_and_network_verify_distinct() {
        assert_ne!(
            map_prelaunch_failure(&ProbePrelaunchFailure::ExpiredLease),
            ProbeRunFailureCode::NetworkDenialVerifyFailed
        );
        assert_ne!(
            map_prelaunch_failure(&ProbePrelaunchFailure::ExpiredLease),
            ProbeRunFailureCode::NetworkDenialSetupFailed
        );
    }

    #[test]
    fn execute_probe_fails_when_job_object_unavailable() {
        let staged = temp_path("job", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&staged, &schema_digests).expect("request");
        let publication_id = "pub-002b2";
        let context = runtime_context_with_lineage(request.clone())
            .with_publication(publication_id, request.lease.expected_fence_token.clone());
        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), "exit 0".to_string()],
        };

        let error = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController {
                unavailable: true,
                ..Default::default()
            },
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("job unavailable should fail");
        assert_runtime_failure_has_structured_error(
            &error,
            ProbeRunFailureCode::JobObjectUnavailable,
            &request,
            None,
        );
        let _ = fs::remove_file(staged);
    }

    #[test]
    fn execute_probe_fails_when_child_launch_fails() {
        let staged = temp_path("launch-fail", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&staged, &schema_digests).expect("request");
        let publication_id = "pub-002b2";
        let context = runtime_context_with_lineage(request.clone())
            .with_publication(publication_id, request.lease.expected_fence_token.clone());

        let factory = CommandProcessFactory {
            command: "does-not-exist.exe".to_string(),
            args: vec!["/C".to_string()],
        };

        let error = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("missing executable should fail launch");

        assert_runtime_failure_has_structured_error(
            &error,
            ProbeRunFailureCode::ChildLaunchFailed,
            &request,
            None,
        );
        assert!(error.message.starts_with("spawn failed"));
        let _ = fs::remove_file(staged);
    }

    #[test]
    fn execute_probe_fails_when_watchdog_unavailable() {
        let staged = temp_path("watchdog", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&staged, &schema_digests).expect("request");
        let publication_id = "pub-002b2";
        let context = runtime_context_with_lineage(request.clone())
            .with_publication(publication_id, request.lease.expected_fence_token.clone());
        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), "exit 0".to_string()],
        };
        let error = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                unavailable: true,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("watchdog unavailable should fail");
        assert_runtime_failure_has_structured_error(
            &error,
            ProbeRunFailureCode::WatchdogUnavailable,
            &request,
            None,
        );
        let _ = fs::remove_file(staged);
    }

    #[test]
    fn execute_probe_fails_when_process_crashes() {
        let staged = temp_path("crash", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&staged, &schema_digests).expect("request");
        let publication_id = "pub-002b2";
        let context = runtime_context_with_lineage(request.clone())
            .with_publication(publication_id, request.lease.expected_fence_token.clone());
        let factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), "exit 1".to_string()],
        };
        let error = execute_bound_probe_with_context(
            context,
            &factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("non-zero child should fail");
        assert_runtime_failure_has_structured_error(
            &error,
            ProbeRunFailureCode::ChildCrashed,
            &request,
            None,
        );
        let _ = fs::remove_file(staged);
    }

    #[test]
    fn execute_probe_detects_malformed_and_misaligned_child_output() {
        let staged = temp_path("malformed", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&staged, &schema_digests).expect("request");
        let request_for_error_assertion = request.clone();
        let publication_id = "pub-002b2";
        let context = runtime_context_with_lineage(request)
            .with_publication(publication_id, "pub-fence-002b2".to_string());
        let malformed_factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), "echo not-json".to_string()],
        };
        let malformed_error = execute_bound_probe_with_context(
            context.clone(),
            &malformed_factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("malformed should fail");
        assert_runtime_failure_has_structured_error(
            &malformed_error,
            ProbeRunFailureCode::JsonMalformed,
            &request_for_error_assertion,
            None,
        );

        let oversized_payload = "x".repeat((CONTROL_ENVELOPE_BYTES + 1024) as usize);
        let oversized_path = temp_path("oversized", "txt");
        write_temp_text(&oversized_path, &oversized_payload);
        let oversized_factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), format!("type {}", oversized_path.to_string_lossy())],
        };

        let context = runtime_context_with_lineage(
            request_schema_identity(&staged, &fixed_schema_digests()).expect("request"),
        )
        .with_publication(publication_id, "pub-fence-002b2".to_string());
        let oversized_error = execute_bound_probe_with_context(
            context,
            &oversized_factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 10,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("oversized should fail");
        assert_runtime_failure_has_structured_error(
            &oversized_error,
            ProbeRunFailureCode::OutputStdoutOversized,
            &request_for_error_assertion,
            None,
        );
        let _ = fs::remove_file(staged);
        let _ = fs::remove_file(oversized_path);
    }

    #[test]
    fn execute_probe_times_out_and_can_be_cancelled() {
        let output_json_path = temp_path("timeout-output", "json");
        let output_payload = r#"{"format":{"format_name":"matroska","duration":"1.0","nb_streams":"0"},"streams":[]}"#;
        write_temp_text(&output_json_path, output_payload);

        let staged = temp_path("timeout", "bin");
        write_temp_binary(&staged, b"source");
        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&staged, &schema_digests).expect("request");
        let request_for_error_assertion = request.clone();
        let publication_id = "pub-002b2";
        let timeout_context = runtime_context_with_lineage(request.clone())
            .with_publication(publication_id, request.lease.expected_fence_token.clone())
            .with_watchdog_seconds(1);
        let cancel_context = runtime_context_with_lineage(request)
            .with_publication(publication_id, request_for_error_assertion.lease.expected_fence_token.clone())
            .with_watchdog_seconds(30);

        let timeout_factory_command = format!(
            "type {} & ping 127.0.0.1 -n 30 >nul",
            output_json_path.to_string_lossy()
        );
        let sleep_factory = CommandProcessFactory {
            command: "cmd".to_string(),
            args: vec!["/C".to_string(), timeout_factory_command],
        };
        let timeout_error = execute_bound_probe_with_context(
            timeout_context,
            &sleep_factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 1,
                ..Default::default()
            },
            |_| false,
        )
        .expect_err("timeout expected");
        assert_runtime_failure_has_structured_error(
            &timeout_error,
            ProbeRunFailureCode::WatchdogTimeout,
            &request_for_error_assertion,
            None,
        );

        let cancel_error = execute_bound_probe_with_context(
            cancel_context,
            &sleep_factory,
            &MockNetworkController::default(),
            &MockJobController::default(),
            &MockWatchdogController {
                timeout: 30,
                ..Default::default()
            },
            |elapsed| elapsed > 1,
        )
        .expect_err("cancellation expected");
        assert_runtime_failure_has_structured_error(
            &cancel_error,
            ProbeRunFailureCode::ProcessCancellation,
            &request_for_error_assertion,
            None,
        );

        let _ = fs::remove_file(staged);
        let _ = fs::remove_file(output_json_path);
    }

    #[test]
    #[ignore = "live evidence only: generates synthetic fixture and executes real ffprobe under live controllers"]
    fn execute_live_synthetic_matroska_with_real_containment_controls() {
        let ffmpeg_path = Path::new(PROBE_EXECUTABLE)
            .with_file_name("ffmpeg.exe")
            .to_string_lossy()
            .to_string();

        assert!(
            Path::new(&ffmpeg_path).exists(),
            "ffmpeg executable expected beside registered ffprobe at {ffmpeg_path}"
        );

        let fixture_dir = temp_path("live", "mkv");
        fs::create_dir_all(&fixture_dir).expect("fixture staging dir");
        let source_path = fixture_dir.join("synthetic-real.mkv");
        let subtitle_path = fixture_dir.join("synthetic.srt");

        fs::write(
            &subtitle_path,
            "1\n00:00:00,000 --> 00:00:01,000\nSynthetic subtitle for containment proof\n",
        )
        .expect("write synthetic subtitle");

        let encode_status = Command::new(&ffmpeg_path)
            .args([
                "-y",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc=size=64x64:rate=24:d=1",
                "-c:v",
                "ffv1",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=1",
                "-i",
                subtitle_path.to_string_lossy().as_ref(),
                "-c:v",
                "ffv1",
                "-map",
                "0:v",
                "-map",
                "1:a",
                "-map",
                "2:s:0",
                "-c:a",
                "pcm_s16le",
                "-ar",
                "48000",
                "-ac",
                "2",
                "-c:s",
                "subrip",
                "-shortest",
                source_path.to_string_lossy().as_ref(),
            ])
            .status()
            .expect("ffmpeg synthetic fixture command");
        assert!(encode_status.success(), "ffmpeg synthetic fixture command failed");

        let schema_digests = fixed_schema_digests();
        let request = request_schema_identity(&source_path, &schema_digests).expect("request");
        let artifact_id = runtime_output_artifact_id(&request);
        let context = ProbeRuntimeContext::new()
            .with_request(request)
            .with_schema_digests(schema_digests)
            .with_watchdog_seconds(CONTROL_LIMITS.wall_clock_seconds)
            .with_publication("pub-002b2-live", "pub-fence-002b2")
            .with_output_staging(fixture_dir.to_string_lossy().to_string())
            .with_output_port(runtime_output_port().to_string())
            .with_output_artifact_id(artifact_id.clone());

        let result = execute_bound_probe_with_context(
            context,
            &DefaultProbeProcessFactory,
            &DefaultNetworkDenialController,
            &DefaultJobObjectController,
            &DefaultWatchdogController,
            |_| false,
        )
        .expect("live real probe run must succeed under live controls");

        assert_eq!(result.normalized_observation.probe_observation.format, "matroska,webm");
        assert!(
            result
                .normalized_observation
                .probe_observation
                .format
                .split(',')
                .any(|value| value.eq_ignore_ascii_case("webm")),
            "format_name must be treated as raw metadata and not promoted outside matroska",
        );
        assert_eq!(result.normalized_observation.probe_observation.stream_count, 3);
        assert!(result.normalized_observation.probe_observation.has_subtitles);
        assert_eq!(result.output_contract.publication_id, "pub-002b2-live");
        assert_eq!(result.output_contract.artifact_id, artifact_id);

        fs::remove_file(&source_path).ok();
        fs::remove_file(&subtitle_path).ok();
    }

    #[test]
    fn parse_probe_output_observation_requires_expected_fields() {
        let missing_fields = r#"{"streams":[]}"#;
        let error = parse_probe_observation(missing_fields).expect_err("missing fields rejected");
        assert_eq!(error.code, ProbeRunFailureCode::JsonMalformed);
    }

    #[test]
    fn verify_registered_ffprobe_metadata_matches_contract_constants() {
        let output = Command::new(PROBE_EXECUTABLE).arg("-version").output().expect("ffprobe -version should run");
        assert!(
            output.status.success(),
            "ffprobe -version failed; code should be 0 to establish live executable identity"
        );

        let text = String::from_utf8_lossy(&output.stdout);
        let mut lines = text.lines();
        let version_line = lines.next().unwrap_or_default();
        assert!(
            version_line.starts_with(PROBE_EXECUTABLE_VERSION),
            "ffprobe version mismatch in launch profile"
        );
        let config_line = lines
            .find_map(|line| line.strip_prefix("configuration: ").map(|value| format!("configuration: {value}")))
            .expect("ffprobe -version should expose configuration");
        assert_eq!(config_line, PROBE_EXECUTABLE_CONFIGURATION);
    }
}

fn map_containment_failure(error: ProbeRunFailure) -> ProbeRunFailure {
    error
}

fn to_structured_runtime_error(
    code: &ProbeRunFailureCode,
    message: &str,
    request: &ProbePrelaunchRequest,
    publication_ref: Option<&str>,
) -> StructuredError {
    let mapping = runtime_disposition_mapping(code);
    let mut safe_details = Vec::new();
    let mut safe_causes = Vec::new();

    safe_details.push(message.to_string());
    if !request.trace_id.is_empty() {
        safe_details.push(request.trace_id.clone());
    }
    if !request.correlation_id.is_empty() {
        safe_details.push(request.correlation_id.clone());
    }

    safe_causes.push(request.lease.lease_id.clone());
    if !request.control_protocol_version.is_empty() {
        safe_causes.push(request.control_protocol_version.clone());
    }
    if !request.executable_sha256.is_empty() {
        safe_causes.push(request.executable_sha256.clone());
    }

    let request_error_instance = if request.error_instance_id.trim().is_empty() {
        unique_token("runtime")
    } else {
        request.error_instance_id.clone()
    };
    let error_instance = format!(
        "{request_error_instance}-{:?}-{}",
        code,
        unique_token("eid")
    );
    let identity_refs = mapping
        .identity_refs
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();

    StructuredError {
        schema_id: STRUCTURED_ERROR_SCHEMA_ID,
        schema_version: STRUCTURED_ERROR_SCHEMA_VERSION,
        error_ref: ErrorRef {
            namespace: ERROR_REF_NAMESPACE,
            code: format!("{:?}", code),
            version: ERROR_REF_VERSION,
            error_instance,
        },
        trace_id: request.trace_id.clone(),
        correlation_id: request.correlation_id.clone(),
        subject: "probe runtime execution".to_string(),
        video_disposition: mapping.video_disposition.to_string(),
        shared_state: mapping.shared_terminal_state.to_string(),
        error_category: mapping.error_category.to_string(),
        retry_class: mapping.retry_class.to_string(),
        operation_stage: mapping.operation_stage.to_string(),
        safe_recovery: mapping.safe_recovery.to_string(),
        identity_refs,
        details: safe_details.into_iter().take(3).collect(),
        causes: safe_causes.into_iter().take(3).collect(),
        applies_to: StructuredErrorAppliesTo {
            job_ref: request.job_id.clone(),
            attempt_ref: request.attempt_id.clone(),
            dispatch_ref: request.dispatch_id.clone(),
            worker_ref: request.worker_instance_id.clone(),
            publication_ref: publication_ref.map(str::to_string),
        },
    }
}

fn to_structured_runtime_error_with_synthetic_identity(
    code: &ProbeRunFailureCode,
    message: &str,
    error_instance: &str,
    publication_ref: Option<&str>,
) -> StructuredError {
    let mapping = runtime_disposition_mapping(code);
    let error_instance = if error_instance.trim().is_empty() {
        unique_token("runtime")
    } else {
        format!("{error_instance}-{}", unique_token("eid"))
    };

    StructuredError {
        schema_id: STRUCTURED_ERROR_SCHEMA_ID,
        schema_version: STRUCTURED_ERROR_SCHEMA_VERSION,
        error_ref: ErrorRef {
            namespace: ERROR_REF_NAMESPACE,
            code: format!("{:?}", code),
            version: ERROR_REF_VERSION,
            error_instance,
        },
        trace_id: "synthetic".to_string(),
        correlation_id: "synthetic".to_string(),
        subject: "probe runtime execution".to_string(),
        video_disposition: mapping.video_disposition.to_string(),
        shared_state: mapping.shared_terminal_state.to_string(),
        error_category: mapping.error_category.to_string(),
        retry_class: mapping.retry_class.to_string(),
        operation_stage: mapping.operation_stage.to_string(),
        safe_recovery: mapping.safe_recovery.to_string(),
        identity_refs: mapping
            .identity_refs
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        details: vec!["request context unavailable".to_string(), message.to_string()],
        causes: vec!["missing prelaunch request".to_string()],
        applies_to: StructuredErrorAppliesTo {
            job_ref: "missing".to_string(),
            attempt_ref: "missing".to_string(),
            dispatch_ref: "missing".to_string(),
            worker_ref: "missing".to_string(),
            publication_ref: publication_ref.map(str::to_string),
        },
    }
}

fn runtime_disposition_mapping(code: &ProbeRunFailureCode) -> &'static DispositionMapping {
    let disposition = match code {
        ProbeRunFailureCode::ExecutableIdentityMismatch
        | ProbeRunFailureCode::ExecutableVersionMismatch
        | ProbeRunFailureCode::ExecutableConfigurationMismatch
        | ProbeRunFailureCode::ExecutableAccessFailed
        | ProbeRunFailureCode::SourcePrelaunchFailed
        | ProbeRunFailureCode::SourcePostlaunchMismatch
        | ProbeRunFailureCode::SourceMutationDetected
        | ProbeRunFailureCode::SourceIdentityMismatch
        | ProbeRunFailureCode::SourceLengthMismatch => "VID-INGEST-IDENTITY-MISMATCH",

        ProbeRunFailureCode::NetworkDenialUnavailable
        | ProbeRunFailureCode::NetworkDenialSetupFailed
        | ProbeRunFailureCode::NetworkDenialVerifyFailed
        | ProbeRunFailureCode::JobObjectUnavailable
        | ProbeRunFailureCode::JobObjectSetupFailed
        | ProbeRunFailureCode::JobObjectVerifyFailed => "VID-INGEST-PROTECTED",

        ProbeRunFailureCode::WatchdogUnavailable
        | ProbeRunFailureCode::WatchdogTimeout
        | ProbeRunFailureCode::OutputStdoutOversized
        | ProbeRunFailureCode::OutputStderrOversized => "VID-INGEST-LIMIT",

        ProbeRunFailureCode::WatchdogCancelled | ProbeRunFailureCode::ProcessCancellation => {
            "VID-INGEST-CANCELLED"
        }
        ProbeRunFailureCode::PublicationLeaseExpired
        | ProbeRunFailureCode::PublicationFenceMismatch
        | ProbeRunFailureCode::OutputLineageInvalid => "VID-INGEST-MALFORMED",

        ProbeRunFailureCode::JsonMalformed
        | ProbeRunFailureCode::JsonStreamLimitExceeded
        | ProbeRunFailureCode::OutputSerializationFailed
        | ProbeRunFailureCode::PrelaunchValidationFailed
        | ProbeRunFailureCode::ArgumentMismatch => "VID-INGEST-MALFORMED",

        ProbeRunFailureCode::ChildLaunchFailed
        | ProbeRunFailureCode::ChildCrashed
        | ProbeRunFailureCode::OutputCaptureFailed => "VID-INGEST-WORKER-FAILED",
    };

    DISPOSITION_MAP
        .iter()
        .find(|entry| entry.video_disposition == disposition)
        .expect("all runtime dispositions must exist in DISPOSITION_MAP")
}

fn validate_output_port_and_artifact_lineage(
    output_port: &str,
    output_artifact_id: &str,
    request: &ProbePrelaunchRequest,
) -> Result<(), ProbeRunFailure> {
    let markers = [
        ("job", request.job_id.as_str()),
        ("attempt", request.attempt_id.as_str()),
        ("dispatch", request.dispatch_id.as_str()),
        ("worker", request.worker_instance_id.as_str()),
        ("port", output_port),
    ];
    if output_port.trim().is_empty() {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::PrelaunchValidationFailed,
            message: "output_port is empty".to_string(),
            structured_error: None,
        });
    }
    if output_artifact_id.trim().is_empty() {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::PrelaunchValidationFailed,
            message: "output_artifact_id is empty".to_string(),
            structured_error: None,
        });
    }
    if markers.iter().any(|(_, marker)| marker.is_empty()) {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::SourcePrelaunchFailed,
            message: "request lineage incomplete for output binding".to_string(),
            structured_error: None,
        });
    }
    if !markers
        .iter()
        .all(|(label, marker)| output_artifact_id.contains(&format!("{label}:{marker}")))
    {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::OutputLineageInvalid,
            message: "output artifact id is not bound to output lineage".to_string(),
            structured_error: None,
        });
    }
    Ok(())
}

fn map_watchdog_failure(error: ProbeRunFailure) -> ProbeRunFailure {
    match error.code {
        ProbeRunFailureCode::WatchdogUnavailable => error,
        _ => ProbeRunFailure {
            code: ProbeRunFailureCode::WatchdogUnavailable,
            message: error.message,
            structured_error: None,
        },
    }
}

fn map_prelaunch_failure(failure: &ProbePrelaunchFailure) -> ProbeRunFailureCode {
    match failure {
        ProbePrelaunchFailure::ArgumentMismatch => ProbeRunFailureCode::ArgumentMismatch,
        ProbePrelaunchFailure::ComponentMismatch | ProbePrelaunchFailure::OutOfPolicyIdentity => {
            ProbeRunFailureCode::ExecutableIdentityMismatch
        }
        ProbePrelaunchFailure::MutableLease => ProbeRunFailureCode::SourceMutationDetected,
        ProbePrelaunchFailure::SourceIdentityMismatch => ProbeRunFailureCode::SourceIdentityMismatch,
        ProbePrelaunchFailure::SourceHashMismatch => ProbeRunFailureCode::SourceIdentityMismatch,
        ProbePrelaunchFailure::SourceLengthMismatch => ProbeRunFailureCode::SourceLengthMismatch,
        ProbePrelaunchFailure::SchemaDigestMismatch(_) | ProbePrelaunchFailure::SchemaReferenceMismatch => {
            ProbeRunFailureCode::PrelaunchValidationFailed
        }
        ProbePrelaunchFailure::ExpiredLease => ProbeRunFailureCode::PublicationLeaseExpired,
        ProbePrelaunchFailure::StaleLeaseOrFence => ProbeRunFailureCode::PublicationFenceMismatch,
        ProbePrelaunchFailure::PathMissing
        | ProbePrelaunchFailure::PathInvalid
        | ProbePrelaunchFailure::ScopeMismatch
        | ProbePrelaunchFailure::TransportUnsupported
        | ProbePrelaunchFailure::InvalidLaunchPolicy
        | ProbePrelaunchFailure::OperationIdentityMismatch
        | ProbePrelaunchFailure::OperationVersionMismatch
        | ProbePrelaunchFailure::ImplementationMismatch
        | ProbePrelaunchFailure::ImplementationProfileMismatch
        | ProbePrelaunchFailure::WorkerInstanceMismatch
        | ProbePrelaunchFailure::DispatchMismatch
        | ProbePrelaunchFailure::ControlProtocolMismatch
        | ProbePrelaunchFailure::EnvelopeExceeded
        | ProbePrelaunchFailure::PartialCompletenessRejected
        | ProbePrelaunchFailure::MissingContainmentEvidence(_)
        | ProbePrelaunchFailure::UnsupportedCapability(_)
        | ProbePrelaunchFailure::MissingRequiredCapability(_) => ProbeRunFailureCode::PrelaunchValidationFailed,
    }
}

fn verify_registered_executable_identity(executable_path: &str, executable_sha256: &str) -> Result<(), ProbeRunFailure> {
    if !hex_equal_ignore_case(executable_path, PROBE_EXECUTABLE) {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::ExecutableIdentityMismatch,
            message: format!("path mismatch: {executable_path}"),
            structured_error: None,
        });
    }

    if !hex_equal_ignore_case(executable_sha256, REGISTERED_EXECUTABLE_SHA256) {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::ExecutableIdentityMismatch,
            message: "sha mismatch at contract input".to_string(),
            structured_error: None,
        });
    }

    let observed = compute_file_sha256_hex(executable_path).map_err(|error| ProbeRunFailure {
        code: ProbeRunFailureCode::ExecutableAccessFailed,
        message: format!("cannot read registered executable: {error}"),
        structured_error: None,
    })?;
    if !hex_equal_ignore_case(&observed, REGISTERED_EXECUTABLE_SHA256) {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::ExecutableIdentityMismatch,
            message: "executable hash mismatch at runtime".to_string(),
            structured_error: None,
        });
    }

    Ok(())
}

fn verify_registered_executable_metadata(path: &str) -> Result<(), ProbeRunFailure> {
    let output = Command::new(path)
        .arg("-version")
        .output()
        .map_err(|error| ProbeRunFailure {
            code: ProbeRunFailureCode::ExecutableAccessFailed,
            message: format!("cannot execute ffprobe -version: {error}"),
            structured_error: None,
        })?;
    if !output.status.success() {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::ExecutableVersionMismatch,
            message: "ffprobe -version failed".to_string(),
            structured_error: None,
        });
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut lines = text.lines();
    let version_line = lines.next().unwrap_or_default();
    if !version_line.starts_with(PROBE_EXECUTABLE_VERSION) {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::ExecutableVersionMismatch,
            message: format!("unexpected ffprobe version line: {version_line}"),
            structured_error: None,
        });
    }

    let config_line = lines
        .find_map(|line| line.strip_prefix("configuration: ").map(|value| format!("configuration: {value}")))
        .ok_or_else(|| ProbeRunFailure {
            code: ProbeRunFailureCode::ExecutableConfigurationMismatch,
            message: "missing ffprobe configuration line".to_string(),
            structured_error: None,
        })?;
    if config_line != PROBE_EXECUTABLE_CONFIGURATION {
        return Err(ProbeRunFailure {
            code: ProbeRunFailureCode::ExecutableConfigurationMismatch,
            message: "configuration does not match registered executable".to_string(),
            structured_error: None,
        });
    }

    Ok(())
}

fn spawn_output_reader<R: Read + Send + 'static>(stream: R, byte_limit: u64) -> io::Result<JoinHandle<io::Result<CapturedOutput>>> {
    let max = byte_limit.try_into().unwrap_or(usize::MAX);
    Ok(thread::spawn(move || read_with_limit(stream, max)))
}

fn read_with_limit<R: Read>(mut stream: R, byte_limit: usize) -> io::Result<CapturedOutput> {
    let mut bytes = Vec::new();
    let mut truncated = false;
    let mut total = 0usize;
    let mut buffer = [0u8; 4096];

    loop {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        if !truncated {
            let remain = byte_limit.saturating_sub(bytes.len());
            let take = read.min(remain);
            bytes.extend_from_slice(&buffer[..take]);
            if read > remain {
                truncated = true;
            }
        } else {
            truncated = true;
        }
        total += read;
    }

    if total > byte_limit {
        truncated = true;
    }

    Ok(CapturedOutput { bytes, truncated })
}

fn hold_source_input_readonly(path: &str) -> io::Result<ProbeInputEvidenceGuard> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        options.share_mode(1);
    }
    let handle = options.open(path)?;
    Ok(ProbeInputEvidenceGuard {
        _handle: Some(handle),
    })
}

fn validate_probe_output_completeness(completeness: &str) -> Result<(), String> {
    crate::probe_adapter_contract::validate_probe_output_completeness(completeness)
        .map_err(|failure| format!("{failure:?}"))
}

fn sha256_hex(d: impl AsRef<[u8]>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(d.as_ref());
    digest_to_hex(hasher.finalize())
}

fn compute_file_sha256_hex(path: &str) -> io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(digest_to_hex(hasher.finalize()))
}

fn digest_to_hex(digest: impl AsRef<[u8]>) -> String {
    digest.as_ref().iter().map(|value| format!("{value:02x}")).collect()
}

fn live_fingerprint(path: &str) -> Result<(String, u64), FingerprintFailure> {
    let result = fingerprint_local_file_with_hook(path, None::<fn(u64)>).map_err(|error| match error {
        crate::fingerprint::FingerprintDisposition::PreflightFailure(_) => FingerprintFailure::IdentityMismatch,
        crate::fingerprint::FingerprintDisposition::IoFailure => FingerprintFailure::AccessFailed,
        crate::fingerprint::FingerprintDisposition::CandidateMutated => FingerprintFailure::Mutation,
        crate::fingerprint::FingerprintDisposition::ExceedsSourceCeiling => {
            FingerprintFailure::LengthMismatch
        }
    })?;
    Ok((result.sha256_hex, result.byte_length))
}

fn policy_fingerprint() -> String {
    sha256_hex("VID-IMPL-P00-002B1-POLICY")
}

fn sanitize_diagnostics(raw: &str) -> String {
    raw.chars()
        .filter(|ch| ch.is_ascii_graphic() || ch.is_ascii_whitespace())
        .take(4096)
        .collect()
}

fn parse_u64(value: &Value) -> Option<u64> {
    value.as_u64().or_else(|| value.as_str().and_then(|value| value.parse::<u64>().ok()))
}

fn parse_f64(value: &Value) -> Option<f64> {
    value.as_f64().or_else(|| value.as_str().and_then(|value| value.parse::<f64>().ok()))
}

fn hex_equal_ignore_case(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn unique_token(prefix: &str) -> String {
    let token = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{prefix}-{token}")
}

fn wall_clock_ticks_to_100ns(seconds: u64) -> i64 {
    seconds.saturating_mul(10_000_000) as i64
}

#[cfg(windows)]
fn add_network_rule(name: &str, executable: &str, inbound: bool) -> Result<(), String> {
    let direction = if inbound { "in" } else { "out" };
    let status = Command::new("netsh")
        .args([
            "advfirewall",
            "firewall",
            "add",
            "rule",
            &format!("name={name}"),
            &format!("dir={direction}"),
            "action=block",
            &format!("program={executable}"),
            "enable=yes",
            "profile=any",
        ])
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("netsh add rule failed: {status}"))
    }
}

#[cfg(windows)]
fn verify_network_rule(name: &str) -> Result<bool, String> {
    let output = Command::new("netsh")
        .args(["advfirewall", "firewall", "show", "rule", &format!("name={name}")])
        .output()
        .map_err(|error| error.to_string())?;
    Ok(output.status.success())
}

#[cfg(windows)]
fn remove_network_rule(name: &str) -> Result<(), String> {
    let _ = Command::new("netsh")
        .args(["advfirewall", "firewall", "delete", "rule", &format!("name={name}")])
        .status()
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[derive(Debug)]
enum FingerprintFailure {
    IdentityMismatch,
    LengthMismatch,
    Mutation,
    AccessFailed,
}
