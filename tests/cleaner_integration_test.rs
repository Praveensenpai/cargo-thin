use cargo_thin::domain::cleaner::clean_project;
use cargo_thin::domain::elf::is_valid_elf_executable;
use cargo_thin::domain::project::{BinaryArtifact, ProfileKind, ProjectInfo, TargetBreakdown};
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn create_mock_elf_binary(path: &Path, size: usize) {
    let mut file = File::create(path).unwrap();
    let mut data = vec![0u8; size.max(24)];
    data[0..4].copy_from_slice(&[0x7f, b'E', b'L', b'F']); // ELF magic
    data[16..18].copy_from_slice(&3u16.to_le_bytes()); // ET_DYN (PIE)
    data[20..24].copy_from_slice(b"TEST");
    file.write_all(&data).unwrap();

    let mut perms = file.metadata().unwrap().permissions();
    perms.set_mode(0o755);
    file.set_permissions(perms).unwrap();
}

fn setup_mock_project(root: &Path) -> (ProjectInfo, PathBuf, PathBuf) {
    let target_dir = root.join("target");
    let release_dir = target_dir.join("release");
    let debug_dir = target_dir.join("debug");
    fs::create_dir_all(release_dir.join("deps")).unwrap();
    fs::create_dir_all(release_dir.join("incremental")).unwrap();
    fs::create_dir_all(debug_dir.join("deps")).unwrap();

    let release_bin = release_dir.join("my-cool-cli");
    create_mock_elf_binary(&release_bin, 10_000);

    let debug_bin = debug_dir.join("my-cool-cli");
    create_mock_elf_binary(&debug_bin, 50_000);

    fs::write(
        release_dir.join("deps").join("libfoo.rlib"),
        vec![1u8; 100_000],
    )
    .unwrap();
    fs::write(
        debug_dir.join("deps").join("libfoo.rlib"),
        vec![2u8; 200_000],
    )
    .unwrap();

    let project = ProjectInfo {
        root_path: root.to_path_buf(),
        package_name: "my-cool-cli".to_string(),
        target_path: target_dir,
        total_target_bytes: 360_000,
        binaries: vec![
            BinaryArtifact {
                path: release_bin.clone(),
                filename: "my-cool-cli".to_string(),
                profile: ProfileKind::Release,
                size_bytes: 10_000,
                is_valid_elf: true,
            },
            BinaryArtifact {
                path: debug_bin.clone(),
                filename: "my-cool-cli".to_string(),
                profile: ProfileKind::Debug,
                size_bytes: 50_000,
                is_valid_elf: true,
            },
        ],
        breakdown: TargetBreakdown::default(),
        selected: true,
    };

    (project, release_bin, debug_bin)
}

#[test]
fn test_clean_project_preserves_release_binary_and_purges_intermediate_bloat() {
    let temp = tempdir().unwrap();
    let (project, release_bin, debug_bin) = setup_mock_project(temp.path());

    let report = clean_project(&project, false).unwrap();

    assert_eq!(report.package_name, "my-cool-cli");
    assert_eq!(report.binaries_preserved, 1);
    assert!(report.bytes_freed > 250_000);

    // Release binary MUST be preserved
    assert!(release_bin.exists());
    assert!(is_valid_elf_executable(&release_bin));
    let restored_bytes = fs::read(&release_bin).unwrap();
    assert_eq!(&restored_bytes[20..24], b"TEST");

    // Intermediate directories and debug tree MUST be purged
    let target = temp.path().join("target");
    assert!(!target.join("release").join("deps").exists());
    assert!(!debug_bin.exists());
}
