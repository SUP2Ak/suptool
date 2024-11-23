use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::Metadata;
use std::path::Path;
use std::hash::{Hash, Hasher};
use std::hash::DefaultHasher;

#[derive(Clone, Copy)]
pub(crate) struct FileEntry {
    pub path_id: u32,     // ID dans la table des chemins (4 bytes)
    pub name_hash: u64,   // Hash du nom (8 bytes)
    pub flags: u8,        // Attributs (1 byte)
    pub size: u32,        // Taille compressée (4 bytes)
    pub date: u32,        // Date compressée (4 bytes)
}

impl FileEntry {
    pub fn new(path_id: u32, name: &str, size: u64, is_dir: bool, modified: u64) -> Self {
        let mut flags = FileFlags::empty();
        if is_dir {
            flags.insert(FileFlags::DIRECTORY);
        }
        
        Self {
            path_id,
            name_hash: fast_hash(name),
            flags: flags.bits(),
            size: compress_size(size),
            date: compress_date(modified),
        }
    }

    pub fn is_dir(&self) -> bool {
        FileFlags::from_bits_truncate(self.flags).contains(FileFlags::DIRECTORY)
    }

    pub fn decompress_size(&self) -> u64 {
        decompress_size(self.size)
    }

    pub fn decompress_date(&self) -> SystemTime {
        decompress_date(self.date)
    }

    pub fn from_metadata(metadata: &Metadata, path: &Path) -> Self {
        let name_hash = fast_hash(path.file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default()
            .as_ref());
            
        Self {
            path_id: 0,  // Sera défini plus tard
            name_hash,
            flags: if metadata.is_dir() { FileFlags::DIRECTORY.bits() } else { 0 },
            size: compress_size(metadata.len()),
            date: compress_date(metadata.modified()
                .unwrap_or_else(|_| SystemTime::now())
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs())
        }
    }

    pub fn set_path_id(&mut self, path_id: u32) {
        self.path_id = path_id;
    }

    pub fn get_name(&self) -> String {
        // Décompresser et retourner le nom stocké dans le hash
        let mut hasher = DefaultHasher::new();
        self.name_hash.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

bitflags::bitflags! {
    struct FileFlags: u8 {
        const DIRECTORY = 0b0000_0001;
        const HIDDEN    = 0b0000_0010;
        const SYSTEM    = 0b0000_0100;
        const READONLY  = 0b0000_1000;
    }
}

#[inline]
fn compress_size(size: u64) -> u32 {
    if size == 0 {
        return 0;
    }
    // Compression logarithmique
    let log = (size as f64).log2() as u32;
    let mantissa = ((size as f64) / (1u64 << log) as f64 * 256.0) as u32;
    (log << 8) | (mantissa & 0xFF)
}

#[inline]
fn decompress_size(compressed: u32) -> u64 {
    if compressed == 0 {
        return 0;
    }
    let log = compressed >> 8;
    let mantissa = (compressed & 0xFF) as f64 / 256.0;
    ((1u64 << log) as f64 * mantissa) as u64
}

#[inline]
fn compress_date(timestamp: u64) -> u32 {
    // Compression relative à 2000-01-01
    const EPOCH_2000: u64 = 946684800;
    if timestamp < EPOCH_2000 {
        return 0;
    }
    ((timestamp - EPOCH_2000) / 60) as u32 // Précision à la minute
}

#[inline]
fn decompress_date(compressed: u32) -> SystemTime {
    const EPOCH_2000: u64 = 946684800;
    UNIX_EPOCH + std::time::Duration::from_secs(
        EPOCH_2000 + (compressed as u64 * 60)
    )
}

#[inline]
pub(crate) fn fast_hash(s: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for b in s.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}