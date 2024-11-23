use std::path::{Path, PathBuf};

pub(crate) fn compress_path(path: &Path) -> Vec<u8> {
    let mut compressed = Vec::new();
    let components: Vec<_> = path.components().map(|c| c.as_os_str().to_string_lossy()).collect();
    
    for component in components {
        let bytes = component.as_bytes();
        compressed.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
        compressed.extend_from_slice(bytes);
    }
    
    compressed
}

pub(crate) fn decompress_path(data: &[u8]) -> Option<PathBuf> {
    let mut path = PathBuf::new();
    let mut pos = 0;

    while pos + 2 <= data.len() {
        let len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        if pos + len > data.len() {
            return None;
        }

        if let Ok(component) = String::from_utf8(data[pos..pos + len].to_vec()) {
            path.push(component);
        } else {
            return None;
        }

        pos += len;
    }

    Some(path)
}
