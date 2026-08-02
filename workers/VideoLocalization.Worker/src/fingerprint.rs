use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::preflight::{
    preflight_local_file,
    source_ceiling_bytes,
    FileMetadataSnapshot,
    PreflightDisposition,
    snapshot_metadata,
};

pub const SHA2_CRATE_VERSION: &str = "0.11.0";

#[derive(Debug, Clone)]
pub struct FingerprintAlgorithm {
    pub name: &'static str,
    pub version: &'static str,
}

#[derive(Debug, Clone)]
pub struct FingerprintObservation {
    /// Candidate file-path and hash metadata only; caller assigns stable Video asset IDs.
    pub attempted_path: String,
    /// Candidate file-path observation only; not an authoritative identity for Video assets.
    pub canonical_path: String,
    pub byte_length: u64,
    pub sha256_hex: String,
    pub algorithm: FingerprintAlgorithm,
    pub metadata_before_hash: FileMetadataSnapshot,
    pub metadata_after_hash: FileMetadataSnapshot,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FingerprintDisposition {
    PreflightFailure(PreflightDisposition),
    IoFailure,
    CandidateMutated,
    ExceedsSourceCeiling,
}

pub const DEFAULT_HASH_CHUNK_BYTES: usize = 64 * 1024;

pub fn fingerprint_local_file(path: impl AsRef<Path>) -> Result<FingerprintObservation, FingerprintDisposition> {
    fingerprint_local_file_internal::<fn(u64)>(path, None)
}

pub fn fingerprint_local_file_with_hook<F>(
    path: impl AsRef<Path>,
    on_chunk_hook: Option<F>,
) -> Result<FingerprintObservation, FingerprintDisposition>
where
    F: FnMut(u64),
{
    fingerprint_local_file_internal(path, on_chunk_hook)
}

fn fingerprint_local_file_internal<F>(
    path: impl AsRef<Path>,
    on_chunk_hook: Option<F>,
) -> Result<FingerprintObservation, FingerprintDisposition>
where
    F: FnMut(u64),
{
    fingerprint_local_file_internal_with_ceiling(path, on_chunk_hook, source_ceiling_bytes())
}

fn fingerprint_local_file_internal_with_ceiling<F>(
    path: impl AsRef<Path>,
    mut on_chunk_hook: Option<F>,
    ceiling_bytes: u64,
) -> Result<FingerprintObservation, FingerprintDisposition>
where
    F: FnMut(u64),
{
    let preflight = preflight_local_file(path).map_err(FingerprintDisposition::PreflightFailure)?;
    let mut file = open_read_handle_for_fingerprint(&preflight.canonical_path)?;
    let pre_meta = snapshot_metadata(&file.metadata().map_err(|_| FingerprintDisposition::IoFailure)?);

    file.seek(SeekFrom::Start(0)).map_err(|_| FingerprintDisposition::IoFailure)?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; DEFAULT_HASH_CHUNK_BYTES];
    let mut total_read = 0u64;

    loop {
        let read_bytes = file.read(&mut buffer).map_err(|_| FingerprintDisposition::IoFailure)?;
        if read_bytes == 0 {
            break;
        }

        hasher.update(&buffer[..read_bytes]);
        total_read += read_bytes as u64;

        if total_read > ceiling_bytes {
            return Err(FingerprintDisposition::ExceedsSourceCeiling);
        }

        if let Some(hook) = on_chunk_hook.as_mut() {
            hook(total_read);
        }
    }

    let metadata_after_hash = snapshot_metadata(&file.metadata().map_err(|_| FingerprintDisposition::IoFailure)?);

    if pre_meta.byte_length != total_read
        || pre_meta.byte_length != metadata_after_hash.byte_length
        || pre_meta.readonly != metadata_after_hash.readonly
        || pre_meta.created_unix_millis != metadata_after_hash.created_unix_millis
        || pre_meta.modified_unix_millis != metadata_after_hash.modified_unix_millis
    {
        return Err(FingerprintDisposition::CandidateMutated);
    }

    let mut hex = String::with_capacity(64);
    for byte in hasher.finalize().iter() {
        use std::fmt::Write;
        let _ = write!(hex, "{:02x}", byte);
    }

    Ok(FingerprintObservation {
        attempted_path: preflight.attempted_path.to_string_lossy().to_string(),
        canonical_path: preflight.canonical_path.to_string_lossy().to_string(),
        byte_length: total_read,
        sha256_hex: hex,
        algorithm: FingerprintAlgorithm {
            name: "SHA-256",
            version: SHA2_CRATE_VERSION,
        },
        metadata_before_hash: pre_meta,
        metadata_after_hash,
    })
}

#[cfg(windows)]
fn open_read_handle_for_fingerprint(path: &Path) -> Result<File, FingerprintDisposition> {
    use std::os::windows::fs::OpenOptionsExt;

    OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(path)
        .map_err(|_| FingerprintDisposition::IoFailure)
}

#[cfg(not(windows))]
fn open_read_handle_for_fingerprint(path: &Path) -> Result<File, FingerprintDisposition> {
    File::open(path).map_err(|_| FingerprintDisposition::IoFailure)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file_path(prefix: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("{}-{}.bin", prefix, nanos));
        path
    }

    fn write_temp_file(prefix: &str, content: &[u8]) -> std::path::PathBuf {
        let path = temp_file_path(prefix);
        std::fs::write(&path, content).expect("temp file write");
        path
    }

    #[test]
    fn fingerprint_loop_enforces_ceiling_before_completion() {
        let content = b"loop-ceiling-probe";
        let path = write_temp_file("vid-impl-002a-loop-ceiling", content);

        let result =
            super::fingerprint_local_file_internal_with_ceiling(&path, None::<fn(u64)>, (content.len() / 2) as u64);

        std::fs::remove_file(&path).ok();
        assert!(matches!(
            result,
            Err(FingerprintDisposition::ExceedsSourceCeiling)
        ));
    }
}
