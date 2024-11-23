use super::entry::FileEntry;
use super::bitmap::CharBitmap;
use std::io::{self, Read, Write};
use std::mem;
use std::path::PathBuf;
use std::path::Path;

const BLOCK_SIZE: usize = 64 * 1024;  // 64KB

#[repr(C)]
pub(crate) struct SearchBlock {
    pub char_bitmap: CharBitmap,      // Bitmap de tous les caractères
    pub name_starts: CharBitmap,      // Bitmap des premiers caractères
    pub entries: Vec<FileEntry>,      // Entrées du bloc
    pub paths: Vec<PathBuf>,          // Chemins associés aux entrées
}

impl SearchBlock {
    pub fn new() -> Self {
        Self {
            char_bitmap: CharBitmap::new(),
            name_starts: CharBitmap::new(),
            entries: Vec::with_capacity(BLOCK_SIZE / mem::size_of::<FileEntry>()),
            paths: Vec::with_capacity(BLOCK_SIZE / mem::size_of::<FileEntry>()),
        }
    }

    pub fn add_entry(&mut self, entry: FileEntry, path: &Path) -> bool {
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        
        let name_lower = name.to_lowercase();
        
        for c in name_lower.bytes() {
            self.char_bitmap.set(c);
        }
        
        if let Some(first_char) = name_lower.bytes().next() {
            self.name_starts.set(first_char);
        }

        self.entries.push(entry);
        self.paths.push(path.to_path_buf());
        true
    }

    pub fn matches(&self, query: &str) -> Result<bool, &'static str> {
        let query = query.to_lowercase();
        
        if query.is_empty() {
            return Ok(false);
        }

        // Vérifier le premier caractère
        if let Some(first_char) = query.bytes().next() {
            //println!("DEBUG: Vérification du caractère '{}' dans le bloc", first_char as char);
            //println!("DEBUG: Bitmap des premiers caractères: {:?}", self.name_starts);
            if !self.name_starts.get(first_char) {
                return Ok(false);
            }
        }

        // Vérifier les chemins
        for path in &self.paths {
            let name = path.file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            
            if name.contains(&query) {
                println!("DEBUG: Trouvé '{}' dans '{}'", query, name);
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        // Écrire les bitmaps
        self.char_bitmap.write_to(writer)?;
        self.name_starts.write_to(writer)?;

        // Écrire le nombre d'entrées
        writer.write_all(&(self.entries.len() as u32).to_le_bytes())?;

        // Écrire les entrées
        for entry in &self.entries {
            writer.write_all(unsafe {
                std::slice::from_raw_parts(
                    entry as *const _ as *const u8,
                    mem::size_of::<FileEntry>()
                )
            })?;
        }

        Ok(())
    }

    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut block = Self::new();

        // Lire les bitmaps
        block.char_bitmap = CharBitmap::read_from(reader)?;
        block.name_starts = CharBitmap::read_from(reader)?;

        // Lire le nombre d'entrées
        let mut size_bytes = [0u8; 4];
        reader.read_exact(&mut size_bytes)?;
        let entry_count = u32::from_le_bytes(size_bytes) as usize;

        // Lire les entrées
        for _ in 0..entry_count {
            let mut entry_bytes = [0u8; mem::size_of::<FileEntry>()];
            reader.read_exact(&mut entry_bytes)?;
            let entry = unsafe {
                std::ptr::read(entry_bytes.as_ptr() as *const FileEntry)
            };
            block.entries.push(entry);
        }

        Ok(block)
    }

    fn size_in_bytes(&self) -> usize {
        mem::size_of::<CharBitmap>() * 2 + 
        self.entries.len() * mem::size_of::<FileEntry>()
    }

    pub fn read_multiple(reader: &mut impl Read, count: u32) -> io::Result<Vec<Self>> {
        let mut blocks = Vec::with_capacity(count as usize);
        for _ in 0..count {
            blocks.push(Self::read_from(reader)?);
        }
        Ok(blocks)
    }
}