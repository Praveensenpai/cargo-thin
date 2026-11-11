use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileKind {
    Release,
    Debug,
}

impl std::fmt::Display for ProfileKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Release => write!(f, "release"),
            Self::Debug => write!(f, "debug"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryArtifact {
    pub path: PathBuf,
    pub filename: String,
    pub profile: ProfileKind,
    pub size_bytes: u64,
    pub is_valid_elf: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TargetBreakdown {
    pub deps_bytes: u64,
    pub incremental_bytes: u64,
    pub build_bytes: u64,
    pub fingerprint_bytes: u64,
    pub other_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub root_path: PathBuf,
    pub package_name: String,
    pub target_path: PathBuf,
    pub total_target_bytes: u64,
    pub binaries: Vec<BinaryArtifact>,
    pub breakdown: TargetBreakdown,
    pub selected: bool,
}

impl ProjectInfo {
    pub fn preserved_binaries_bytes(&self, keep_both: bool) -> u64 {
        if keep_both {
            return self.binaries.iter().map(|b| b.size_bytes).sum();
        }

        let has_release = self
            .binaries
            .iter()
            .any(|b| b.profile == ProfileKind::Release);
        if has_release {
            self.binaries
                .iter()
                .filter(|b| b.profile == ProfileKind::Release)
                .map(|b| b.size_bytes)
                .sum()
        } else {
            self.binaries.iter().map(|b| b.size_bytes).sum()
        }
    }

    pub fn reclaimable_bytes(&self, keep_both: bool) -> u64 {
        let preserved = self.preserved_binaries_bytes(keep_both);
        self.total_target_bytes.saturating_sub(preserved)
    }

    pub fn reclaimable_percent(&self, keep_both: bool) -> f64 {
        if self.total_target_bytes == 0 {
            return 0.0;
        }
        let reclaimable = self.reclaimable_bytes(keep_both) as f64;
        let total = self.total_target_bytes as f64;
        (reclaimable / total * 100.0).clamp(0.0, 100.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    TargetSizeDesc,
    ReclaimableDesc,
    NameAsc,
}

impl SortOrder {
    pub fn next(self) -> Self {
        match self {
            Self::TargetSizeDesc => Self::ReclaimableDesc,
            Self::ReclaimableDesc => Self::NameAsc,
            Self::NameAsc => Self::TargetSizeDesc,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::TargetSizeDesc => "Target Size (Desc)",
            Self::ReclaimableDesc => "Reclaimable (Desc)",
            Self::NameAsc => "Name (A-Z)",
        }
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    const TIB: f64 = GIB * 1024.0;

    let b = bytes as f64;
    if b >= TIB {
        format!("{:.2} TB", b / TIB)
    } else if b >= GIB {
        format!("{:.2} GB", b / GIB)
    } else if b >= MIB {
        format!("{:.2} MB", b / MIB)
    } else if b >= KIB {
        format!("{:.2} KB", b / KIB)
    } else {
        format!("{bytes} B")
    }
}
