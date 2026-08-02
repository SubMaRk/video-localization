use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const PHASE00_SOURCE_LIMIT_BYTES: u64 = 2 * 1024 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreflightDisposition {
    MissingPath,
    NotAFile,
    PathTraversal,
    AlternateDataStream,
    DevicePath,
    NonLocalPath,
    ExceedsSourceCeiling,
    InvalidPath,
    PathUnavailable,
}

#[derive(Debug, Clone)]
pub struct FileMetadataSnapshot {
    pub byte_length: u64,
    pub readonly: bool,
    pub created_unix_millis: Option<i128>,
    pub modified_unix_millis: Option<i128>,
}

#[derive(Debug, Clone)]
pub struct PreflightResult {
    pub attempted_path: PathBuf,
    pub canonical_path: PathBuf,
    pub metadata_before_hash: FileMetadataSnapshot,
}

pub fn source_ceiling_bytes() -> u64 {
    PHASE00_SOURCE_LIMIT_BYTES
}

pub fn is_within_source_ceiling(byte_length: u64) -> bool {
    byte_length <= PHASE00_SOURCE_LIMIT_BYTES
}

pub fn preflight_local_file(path: impl AsRef<Path>) -> Result<PreflightResult, PreflightDisposition> {
    let raw_path = path.as_ref();
    let attempted_path = raw_path.to_path_buf();

    validate_path_safety(raw_path)?;

    let metadata = fs::metadata(raw_path).map_err(|err| match err.kind() {
        std::io::ErrorKind::NotFound => PreflightDisposition::MissingPath,
        _ => PreflightDisposition::PathUnavailable,
    })?;

    if !metadata.is_file() {
        return Err(PreflightDisposition::NotAFile);
    }

    let prehash = snapshot_metadata(&metadata);

    if !is_within_source_ceiling(prehash.byte_length) {
        return Err(PreflightDisposition::ExceedsSourceCeiling);
    }

    let canonical_path = raw_path
        .canonicalize()
        .map_err(|_| PreflightDisposition::PathUnavailable)?;

    validate_path_safety(&canonical_path)?;

    Ok(PreflightResult {
        attempted_path,
        canonical_path,
        metadata_before_hash: prehash,
    })

}

fn validate_path_safety(path: &Path) -> Result<(), PreflightDisposition> {
    if let Some(disposition) = classify_path_safety(path) {
        return Err(disposition);
    }

    if has_parent_traversal(path) {
        return Err(PreflightDisposition::PathTraversal);
    }

    if has_alternate_data_stream(path) {
        return Err(PreflightDisposition::AlternateDataStream);
    }

    Ok(())
}

fn has_parent_traversal(path: &Path) -> bool {
    path.components().any(|component| component == Component::ParentDir)
}

fn has_alternate_data_stream(path: &Path) -> bool {
    let path_text = path.to_string_lossy();

    if let Some(verbatim_path) = path_text.strip_prefix("\\\\?\\") {
        if has_drive_prefix(verbatim_path) {
            return false;
        }
    }

    let has_drive_prefix = has_drive_prefix(path_text.as_ref());

    let colon_count = path_text.chars().filter(|ch| *ch == ':').count();

    if has_drive_prefix {
        colon_count > 1
    } else {
        colon_count > 0
    }
}

fn classify_path_safety(path: &Path) -> Option<PreflightDisposition> {
    let text = path.to_string_lossy();

    if text.trim().is_empty() {
        return Some(PreflightDisposition::InvalidPath);
    }

    if text.starts_with("\\\\?\\") {
        return classify_verbatim_namespace(&text);
    }

    if text.starts_with("\\\\.\\" ) {
        return Some(PreflightDisposition::DevicePath);
    }

    if text.starts_with("\\\\") {
        return Some(PreflightDisposition::NonLocalPath);
    }

    if text.contains("://") {
        return Some(PreflightDisposition::NonLocalPath);
    }

    None
}

fn classify_verbatim_namespace(path_text: &str) -> Option<PreflightDisposition> {
    const VERBATIM_PREFIX_LEN: usize = 4;

    if path_text.len() < VERBATIM_PREFIX_LEN {
        return Some(PreflightDisposition::InvalidPath);
    }

    let verbatim_path = &path_text[VERBATIM_PREFIX_LEN..];
    if has_drive_prefix(verbatim_path) {
        return None;
    }

    let normalized = verbatim_path.to_ascii_uppercase();
    if normalized.starts_with("UNC\\")
        || normalized.starts_with("UNC/")
    {
        return Some(PreflightDisposition::NonLocalPath);
    }

    if normalized.starts_with(".\\")
        || normalized.starts_with("./")
        || normalized.starts_with("GLOBALROOT\\")
        || normalized.starts_with("GLOBALROOT/")
    {
        return Some(PreflightDisposition::DevicePath);
    }

    Some(PreflightDisposition::InvalidPath)
}

fn has_drive_prefix(path_text: &str) -> bool {
    path_text.len() >= 3
        && path_text.as_bytes()[1] == b':'
        && path_text.as_bytes()[0].is_ascii_alphabetic()
        && (path_text.as_bytes()[2] == b'\\' || path_text.as_bytes()[2] == b'/')
}

pub(crate) fn snapshot_metadata(metadata: &fs::Metadata) -> FileMetadataSnapshot {
    FileMetadataSnapshot {
        byte_length: metadata.len(),
        readonly: metadata.permissions().readonly(),
        created_unix_millis: to_unix_millis(metadata.created().ok()),
        modified_unix_millis: to_unix_millis(metadata.modified().ok()),
    }
}

fn to_unix_millis(timestamp: Option<SystemTime>) -> Option<i128> {
    timestamp
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| (duration.as_secs() as i128) * 1000 + (duration.subsec_nanos() as i128) / 1_000_000)
}
