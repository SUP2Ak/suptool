use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::io::{self, Read, Write};
use super::compress::{compress_path, decompress_path};

pub(crate) struct PathTable {
    paths: RwLock<Vec<Arc<PathBuf>>>,
    lookup: RwLock<HashMap<Arc<PathBuf>, u32>>,
}

impl PathTable {
    pub fn new() -> Self {
        Self {
            paths: RwLock::new(Vec::new()),
            lookup: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_path(&self, path: &Path) -> u32 {
        let path_buf = PathBuf::from(path);
        let path_arc = Arc::new(path_buf);
        
        // Vérifier si le chemin existe déjà
        if let Some(&id) = self.lookup.read().get(&path_arc) {
            return id;
        }

        // Ajouter le nouveau chemin
        let mut paths = self.paths.write();
        let mut lookup = self.lookup.write();
        
        let id = paths.len() as u32;
        paths.push(path_arc.clone());
        lookup.insert(path_arc, id);
        
        id
    }

    pub fn get_path(&self, id: u32) -> Option<Arc<PathBuf>> {
        self.paths.read().get(id as usize).cloned()
    }

    pub fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        let paths = self.paths.read();
        
        // Écrire le nombre de chemins
        writer.write_all(&(paths.len() as u32).to_le_bytes())?;

        // Écrire chaque chemin compressé
        for path in paths.iter() {
            let compressed = compress_path(path);
            writer.write_all(&(compressed.len() as u32).to_le_bytes())?;
            writer.write_all(&compressed)?;
        }

        Ok(())
    }

    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut table = Self::new();
        
        // Lire le nombre de chemins
        let mut count_bytes = [0u8; 4];
        reader.read_exact(&mut count_bytes)?;
        let count = u32::from_le_bytes(count_bytes);

        // Lire chaque chemin
        for _ in 0..count {
            let mut len_bytes = [0u8; 4];
            reader.read_exact(&mut len_bytes)?;
            let len = u32::from_le_bytes(len_bytes) as usize;

            let mut compressed = vec![0u8; len];
            reader.read_exact(&mut compressed)?;

            if let Some(path) = decompress_path(&compressed) {
                table.add_path(&path);
            }
        }

        Ok(table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_path_table() {
        let table = PathTable::new();
        
        let path1 = PathBuf::from("/home/user/test.txt");
        let path2 = PathBuf::from("/home/user/docs/file.pdf");
        
        let id1 = table.add_path(&path1);
        let id2 = table.add_path(&path2);
        
        assert_ne!(id1, id2);
        assert_eq!(table.get_path(id1).unwrap().as_path(), &path1);
        assert_eq!(table.get_path(id2).unwrap().as_path(), &path2);
    }
}
