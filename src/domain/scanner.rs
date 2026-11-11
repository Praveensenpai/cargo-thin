use std::fs;
use std::path::Path;
use toml::Value;
use walkdir::WalkDir;

use crate::domain::elf::is_valid_elf_executable;
use crate::domain::project::{BinaryArtifact, ProfileKind, ProjectInfo, TargetBreakdown};

pub fn scan_projects(root: &Path) -> Vec<ProjectInfo> {
    let mut projects = Vec::new();

    let walker = WalkDir::new(root)
        .max_depth(4)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "target" && name != ".git" && name != "node_modules"
        });

    for entry in walker.filter_map(Result::ok) {
        if entry.file_name() != "Cargo.toml" {
            continue;
        }
        let project_dir = entry.path().parent().unwrap_or(root);
        let target_dir = project_dir.join("target");
        if !target_dir.is_dir() {
            continue;
        }
        if let Some(info) = inspect_project(project_dir, &target_dir) {
            projects.push(info);
        }
    }

    projects.sort_by_key(|a| std::cmp::Reverse(a.total_target_bytes));
    projects
}

fn inspect_project(project_dir: &Path, target_dir: &Path) -> Option<ProjectInfo> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let package_name = parse_package_name(&cargo_toml_path).unwrap_or_else(|| {
        project_dir
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    });

    let binaries = discover_binaries(target_dir, &package_name);
    let (total_bytes, breakdown) = analyze_target_directory(target_dir);

    Some(ProjectInfo {
        root_path: project_dir.to_path_buf(),
        package_name,
        target_path: target_dir.to_path_buf(),
        total_target_bytes: total_bytes,
        binaries,
        breakdown,
        selected: true,
    })
}

fn parse_package_name(cargo_toml_path: &Path) -> Option<String> {
    let content = fs::read_to_string(cargo_toml_path).ok()?;
    let parsed: Value = toml::from_str(&content).ok()?;
    let name = parsed.get("package")?.get("name")?.as_str()?;
    Some(name.to_string())
}

fn discover_binaries(target_dir: &Path, _pkg_name: &str) -> Vec<BinaryArtifact> {
    let mut binaries = Vec::new();
    let profiles = [
        (ProfileKind::Release, target_dir.join("release")),
        (ProfileKind::Debug, target_dir.join("debug")),
    ];

    for (kind, profile_dir) in profiles {
        if !profile_dir.is_dir() {
            continue;
        }

        let Ok(entries) = fs::read_dir(&profile_dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && is_valid_elf_executable(&path) {
                let filename = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);

                binaries.push(BinaryArtifact {
                    path,
                    filename,
                    profile: kind,
                    size_bytes,
                    is_valid_elf: true,
                });
            }
        }
    }

    binaries
}

fn analyze_target_directory(target_dir: &Path) -> (u64, TargetBreakdown) {
    let mut total_bytes = 0u64;
    let mut breakdown = TargetBreakdown::default();

    for entry in WalkDir::new(target_dir).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }

        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        total_bytes = total_bytes.saturating_add(size);

        categorize_path(entry.path(), target_dir, size, &mut breakdown);
    }

    (total_bytes, breakdown)
}

fn categorize_path(path: &Path, target_dir: &Path, size: u64, breakdown: &mut TargetBreakdown) {
    let Ok(rel) = path.strip_prefix(target_dir) else {
        breakdown.other_bytes = breakdown.other_bytes.saturating_add(size);
        return;
    };

    let rel_str = rel.to_string_lossy();
    if rel_str.contains("/deps/") || rel_str.starts_with("deps/") {
        breakdown.deps_bytes = breakdown.deps_bytes.saturating_add(size);
    } else if rel_str.contains("/incremental/") || rel_str.starts_with("incremental/") {
        breakdown.incremental_bytes = breakdown.incremental_bytes.saturating_add(size);
    } else if rel_str.contains("/build/") || rel_str.starts_with("build/") {
        breakdown.build_bytes = breakdown.build_bytes.saturating_add(size);
    } else if rel_str.contains("/.fingerprint/") || rel_str.starts_with(".fingerprint/") {
        breakdown.fingerprint_bytes = breakdown.fingerprint_bytes.saturating_add(size);
    } else {
        breakdown.other_bytes = breakdown.other_bytes.saturating_add(size);
    }
}
