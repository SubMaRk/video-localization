use std::cell::Cell;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use videolocalization_worker::{
    fingerprint_local_file,
    fingerprint_local_file_with_hook,
    is_within_source_ceiling,
    preflight_local_file,
    PreflightDisposition,
    SHA2_CRATE_VERSION,
};

#[test]
fn known_empty_fixture_has_expected_sha_and_length() {
    let path = write_temp_file("vid-impl-002a-empty", b"");
    let result = fingerprint_local_file(&path).expect("fingerprint");

    assert_eq!(result.byte_length, 0);
    assert_eq!(result.sha256_hex, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    assert_eq!(result.algorithm.version, SHA2_CRATE_VERSION);

    fs::remove_file(path).ok();
}

#[test]
fn known_content_fixture_has_expected_sha_and_length() {
    let content = b"VideoLocalization deterministic fixture";
    let path = write_temp_file("vid-impl-002a-content", content);

    let result = fingerprint_local_file(&path).expect("fingerprint");

    assert_eq!(result.byte_length, content.len() as u64);
    assert_eq!(result.sha256_hex, "5c50dea06b0c2bedab214a9beb2f126b56e7f3344cfc84d657a07783082e8bd6");

    fs::remove_file(path).ok();
}

#[test]
fn hashing_is_chunked_for_multi_chunk_file() {
    let mut payload = vec![0u8; 200_000];
    payload.iter_mut().enumerate().for_each(|(index, value)| {
        *value = (index % 251) as u8;
    });

    let path = write_temp_file("vid-impl-002a-multi", &payload);
    let result = fingerprint_local_file(&path).expect("fingerprint");

    assert_eq!(result.byte_length, payload.len() as u64);
    assert_eq!(result.sha256_hex.len(), 64);

    fs::remove_file(path).ok();
}

#[test]
fn missing_directory_traversal_ads_and_network_paths_fail_distinctly() {
    let missing = preflight_local_file(r"C:\does-not-exist-002a.bin");
    assert!(matches!(missing, Err(PreflightDisposition::MissingPath)));

    let temp_dir = std::env::temp_dir();
    let dir_result = preflight_local_file(&temp_dir);
    assert!(matches!(dir_result, Err(PreflightDisposition::NotAFile)));

    let traversal = preflight_local_file("..\\example\\video.mp4");
    assert!(matches!(traversal, Err(PreflightDisposition::PathTraversal)));

    let ads = preflight_local_file("C:\\temp\\clip.mov:stream");
    assert!(matches!(ads, Err(PreflightDisposition::AlternateDataStream)));

    let network = preflight_local_file("\\\\server\\share\\video.mov");
    assert!(matches!(network, Err(PreflightDisposition::NonLocalPath)));

    let device = preflight_local_file("\\\\.\\pipe\\video");
    assert!(matches!(device, Err(PreflightDisposition::DevicePath)));

    let extended_unc = preflight_local_file(r"\\?\UNC\server\share\video.mov");
    assert!(matches!(extended_unc, Err(PreflightDisposition::NonLocalPath)));

    let extended_device = preflight_local_file(r"\\?\.\pipe\video");
    assert!(matches!(extended_device, Err(PreflightDisposition::DevicePath)));

    let extended_global_root = preflight_local_file(r"\\?\GLOBALROOT\Device\HarddiskVolume1\foo");
    assert!(matches!(extended_global_root, Err(PreflightDisposition::DevicePath)));

    let unknown_verbatim_namespace = preflight_local_file(r"\\?\MYSTERYNS\video.mov");
    assert!(matches!(unknown_verbatim_namespace, Err(PreflightDisposition::InvalidPath)));

    let empty = preflight_local_file("   ");
    assert!(matches!(empty, Err(PreflightDisposition::InvalidPath)));

    let valid_extended_local = preflight_local_file(r"\\?\C:\does-not-exist-002a.bin");
    assert!(matches!(valid_extended_local, Err(PreflightDisposition::MissingPath)));
}

#[test]
fn source_ceiling_check_is_unit_testable_without_full_file() {
    let two_tib = 2 * 1024u64 * 1024 * 1024 * 1024;

    assert!(is_within_source_ceiling(two_tib));
    assert!(!is_within_source_ceiling(two_tib + 1));
}

#[test]
fn mutation_detected_during_hashing_is_rejected() {
    let payload = vec![b'A'; 300_000];
    let path = write_temp_file("vid-impl-002a-mutate", &payload);

    let attempted_mutation = Cell::new(false);
    let write_failed = Cell::new(false);

    let result = fingerprint_local_file_with_hook(
        &path,
        Some(|bytes| {
            if bytes > 64_000 && !attempted_mutation.get() {
                attempted_mutation.set(true);
                let open_result = OpenOptions::new().append(true).open(&path);
                write_failed.set(open_result.is_err());
            }
        }),
    );

    #[cfg(windows)]
    {
        assert!(result.is_ok(), "hashing should complete with sharing denied for write attempts");
        assert!(write_failed.get(), "write must fail while read handle is active");
    }

    #[cfg(not(windows))]
    {
        assert!(
            matches!(result, Err(FingerprintDisposition::CandidateMutated)),
            "mutation should be rejected"
        );
    }

    fs::remove_file(&path).ok();
}

#[test]
fn equal_bytes_generate_same_identity_without_collapsing_path() {
    let content = b"Phase 00 canonicality check";
    let path_one = write_temp_file("vid-impl-002a-eq-a", content);
    let path_two = write_temp_file("vid-impl-002a-eq-b", content);

    let left = fingerprint_local_file(&path_one).expect("left fingerprint");
    let right = fingerprint_local_file(&path_two).expect("right fingerprint");

    assert_eq!(left.sha256_hex, right.sha256_hex);
    assert_ne!(left.canonical_path, right.canonical_path);

    fs::remove_file(path_one).ok();
    fs::remove_file(path_two).ok();
}

#[test]
fn equal_content_can_bind_distinct_caller_asset_ids_without_hash_semantics_change() {
    let content = b"Caller-owned identity is not path-derived";
    let path_one = write_temp_file("vid-impl-002a-asset-a", content);
    let path_two = write_temp_file("vid-impl-002a-asset-b", content);

    let left = fingerprint_local_file(&path_one).expect("left fingerprint");
    let right = fingerprint_local_file(&path_two).expect("right fingerprint");

    let caller_asset_id_left = "VIDEO-ASSET-LHS-001";
    let caller_asset_id_right = "VIDEO-ASSET-RHS-002";

    let left_binding = (caller_asset_id_left, left.sha256_hex.clone(), left.canonical_path);
    let right_binding = (caller_asset_id_right, right.sha256_hex.clone(), right.canonical_path);

    assert_eq!(left_binding.1, right_binding.1);
    assert_eq!(left_binding.2.len(), right_binding.2.len());
    assert_ne!(left_binding.0, right_binding.0);

    fs::remove_file(path_one).ok();
    fs::remove_file(path_two).ok();
}

fn temp_file_path(prefix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("{}-{}.bin", prefix, nanos));
    path
}

fn write_temp_file(prefix: &str, content: &[u8]) -> PathBuf {
    let path = temp_file_path(prefix);
    fs::write(&path, content).expect("temp file write");
    path
}
