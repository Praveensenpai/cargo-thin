use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::domain::elf::is_valid_elf_executable;
use crate::domain::project::{BinaryArtifact, ProfileKind, ProjectInfo};

#[derive(Debug, Error)]
pub enum CleanError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Binary verification failed after purge: {0}")]
    VerificationFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanReport {
    pub package_name: String,
    pub bytes_before: u64,
    pub bytes_after: u64,
    pub bytes_freed: u64,
    pub binaries_preserved: usize,
}

struct StagedBinary {
    staged_path: PathBuf,
    target_rel_path: PathBuf,
    size_bytes: u64,
}

pub fn clean_project(project: &ProjectInfo, keep_both: bool) -> Result<CleanReport, CleanError> {
    let to_preserve = select_binaries_to_preserve(&project.binaries, keep_both);
    let stage_dir = project.target_path.join(".cargo_thin_stage");

    if stage_dir.exists() {
        let _ = fs::remove_dir_all(&stage_dir);
    }
    fs::create_dir_all(&stage_dir)?;

    let staged = stage_binaries(&project.target_path, &to_preserve, &stage_dir)?;

    purge_intermediate_artifacts(&project.target_path, keep_both)?;

    restore_staged_binaries(&project.target_path, &staged)?;

    if stage_dir.exists() {
        let _ = fs::remove_dir_all(&stage_dir);
    }

    let bytes_after = measure_dir_size(&project.target_path);
    let bytes_freed = project.total_target_bytes.saturating_sub(bytes_after);

    Ok(CleanReport {
        package_name: project.package_name.clone(),
        bytes_before: project.total_target_bytes,
        bytes_after,
        bytes_freed,
        binaries_preserved: staged.len(),
    })
}

fn select_binaries_to_preserve(
    binaries: &[BinaryArtifact],
    keep_both: bool,
) -> Vec<&BinaryArtifact> {
    if keep_both {
        return binaries.iter().collect();
    }

    let has_release = binaries.iter().any(|b| b.profile == ProfileKind::Release);
    if has_release {
        binaries
            .iter()
            .filter(|b| b.profile == ProfileKind::Release)
            .collect()
    } else {
        binaries.iter().collect()
    }
}

fn stage_binaries(
    target_dir: &Path,
    binaries: &[&BinaryArtifact],
    stage_dir: &Path,
) -> Result<Vec<StagedBinary>, CleanError> {
    let mut staged = Vec::new();

    for (idx, bin) in binaries.iter().enumerate() {
        let Ok(rel_path) = bin.path.strip_prefix(target_dir) else {
            continue;
        };

        let stage_file = stage_dir.join(format!("staged_bin_{idx}"));
        fs::copy(&bin.path, &stage_file)?;

        staged.push(StagedBinary {
            staged_path: stage_file,
            target_rel_path: rel_path.to_path_buf(),
            size_bytes: bin.size_bytes,
        });
    }

    Ok(staged)
}

fn purge_intermediate_artifacts(target_dir: &Path, keep_both: bool) -> Result<(), CleanError> {
    let debug_dir = target_dir.join("debug");
    let release_dir = target_dir.join("release");

    if !keep_both && release_dir.is_dir() && debug_dir.is_dir() {
        let _ = fs::remove_dir_all(&debug_dir);
    } else if debug_dir.is_dir() {
        purge_profile_dir(&debug_dir)?;
    }

    if release_dir.is_dir() {
        purge_profile_dir(&release_dir)?;
    }

    let tmp_dir = target_dir.join("tmp");
    if tmp_dir.is_dir() {
        let _ = fs::remove_dir_all(tmp_dir);
    }

    let rustc_info = target_dir.join(".rustc_info.json");
    if rustc_info.is_file() {
        let _ = fs::remove_file(rustc_info);
    }

    Ok(())
}

fn purge_profile_dir(profile_dir: &Path) -> io::Result<()> {
    let subdirs = ["deps", "incremental", "build", ".fingerprint", "examples"];
    for sub in subdirs {
        let p = profile_dir.join(sub);
        if p.is_dir() {
            let _ = fs::remove_dir_all(p);
        }
    }

    let Ok(entries) = fs::read_dir(profile_dir) else {
        return Ok(());
    };

    for entry in entries.flatten() {
        clean_extraneous_file(&entry.path());
    }

    Ok(())
}

fn clean_extraneous_file(path: &Path) {
    if !path.is_file() {
        return;
    }
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    if name.ends_with(".d") || name.ends_with(".rlib") || name.ends_with(".rmeta") {
        let _ = fs::remove_file(path);
    }
}

fn restore_staged_binaries(target_dir: &Path, staged: &[StagedBinary]) -> Result<(), CleanError> {
    for item in staged {
        let dest = target_dir.join(&item.target_rel_path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::copy(&item.staged_path, &dest)?;
        let meta = fs::metadata(&dest)?;
        let mut perms = meta.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest, perms)?;

        if meta.len() != item.size_bytes {
            return Err(CleanError::VerificationFailed(format!(
                "Restored binary size mismatch at {:?}: expected {}, found {}",
                dest,
                item.size_bytes,
                meta.len()
            )));
        }

        if !is_valid_elf_executable(&dest) {
            return Err(CleanError::VerificationFailed(format!(
                "Restored binary at {:?} is corrupted or invalid",
                dest
            )));
        }
    }

    Ok(())
}

fn measure_dir_size(dir: &Path) -> u64 {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}
