use std::fs::File;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
const ET_EXEC: u16 = 2;
const ET_DYN: u16 = 3;

pub fn is_valid_elf_executable(path: &Path) -> bool {
    let Ok(metadata) = path.metadata() else {
        return false;
    };

    if !metadata.is_file() {
        return false;
    }

    // Must have executable bit set
    if metadata.permissions().mode() & 0o111 == 0 {
        return false;
    }

    // Reject shared libraries and object files
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        if file_name.ends_with(".so")
            || file_name.contains(".so.")
            || file_name.ends_with(".o")
            || file_name.ends_with(".rlib")
            || file_name.ends_with(".a")
        {
            return false;
        }
    }

    let Ok(mut file) = File::open(path) else {
        return false;
    };

    // Read ELF ident and header
    let mut header = [0u8; 18];
    if file.read_exact(&mut header).is_err() {
        return false;
    }

    if header[0..4] != ELF_MAGIC {
        return false;
    }

    // e_type is at byte 16..18 in little endian on x86_64
    let e_type = u16::from_le_bytes([header[16], header[17]]);
    e_type == ET_EXEC || e_type == ET_DYN
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_rejects_empty_file() {
        let temp = NamedTempFile::new().unwrap();
        assert!(!is_valid_elf_executable(temp.path()));
    }

    #[test]
    fn test_valid_elf_pie_executable() {
        let mut temp = NamedTempFile::new().unwrap();
        let mut header = vec![0u8; 18];
        header[0..4].copy_from_slice(&ELF_MAGIC);
        header[16..18].copy_from_slice(&3u16.to_le_bytes()); // ET_DYN (PIE)
        temp.write_all(&header).unwrap();

        let mut perms = temp.as_file().metadata().unwrap().permissions();
        perms.set_mode(0o755);
        temp.as_file().set_permissions(perms).unwrap();

        assert!(is_valid_elf_executable(temp.path()));
    }
}
