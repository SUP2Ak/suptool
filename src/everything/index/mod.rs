use std::sync::Arc;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::io::{self, Read, Write, Seek, SeekFrom};
use windows::Win32::Storage::FileSystem::{CreateFileW, FILE_GENERIC_READ, FILE_SHARE_READ, 
    OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, FILE_ATTRIBUTE_NORMAL, FILE_READ_ATTRIBUTES};
use windows::Win32::Foundation::{HANDLE, CloseHandle};
use windows::core::{PCWSTR, HSTRING};
use std::cmp::Ordering;

use std::fs::File;

use memmap2::{Mmap, MmapOptions};
// use std::time::SystemTime;
// use rayon::prelude::*;

pub mod header;
pub mod entry;
pub mod block;
pub mod bitmap;
// use crate::pages::everysup::MAX_DISPLAY_RESULTS;
use crate::everything::path::PathTable;
// use crate::everything::search::SearchResult;
// use crate::everything::search::SearchEngine;
use crate::everything::index::header::IndexHeader;
use crate::everything::index::entry::FileEntry;
use crate::everything::index::block::SearchBlock;

// const BLOCK_SIZE: usize = 64 * 1024;    // 64KB par bloc
// const MAX_PATH_LENGTH: usize = 260;     // MAX_PATH Windows
// const INDEX_VERSION: u32 = 1;
// const CACHE_FLUSH_THRESHOLD: usize = 10_000;

pub struct EverythingIndex {
    pub header: IndexHeader,
    pub blocks: Vec<SearchBlock>,
    pub path_table: Arc<PathTable>,
    pub mmap: Option<Mmap>,
}

impl EverythingIndex {
    pub fn new() -> Self {
        Self {
            header: IndexHeader::new(),
            blocks: Vec::new(),
            path_table: Arc::new(PathTable::new()),
            mmap: None,
        }
    }

    pub fn add_file(&mut self, path: &Path) -> io::Result<()> {
        let start_time = std::time::Instant::now();
        let mut total_files = 0;
        let batch_size = 100_000;
        let mut entries = Vec::with_capacity(batch_size);
        
        if path.is_dir() {
            let mut stack = vec![path.to_path_buf()];
            
            while let Some(current_path) = stack.pop() {
                if let Ok(read_dir) = std::fs::read_dir(&current_path) {
                    for entry in read_dir.filter_map(Result::ok) {
                        let path = entry.path();
                        if let Ok(metadata) = entry.metadata() {
                            let file_entry = FileEntry::from_metadata(&metadata, &path);
                            entries.push((file_entry, path.clone()));
                            
                            if entries.len() >= batch_size {
                                self.process_batch(&entries, &mut total_files, start_time)?;
                                entries.clear();
                            }
                            
                            if path.is_dir() {
                                stack.push(path);
                            }
                        }
                    }
                }
            }
        }
        
        // Traiter le dernier lot
        if !entries.is_empty() {
            self.process_batch(&entries, &mut total_files, start_time)?;
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        println!("Total indexé : {} fichiers en {:.1}s ({:.0} fichiers/sec)", 
            total_files, elapsed, total_files as f64 / elapsed);

        Ok(())
    }

    fn process_batch(&mut self, entries: &[(FileEntry, PathBuf)], total_files: &mut usize, start_time: std::time::Instant) -> io::Result<()> {
        *total_files += entries.len();
        let elapsed = start_time.elapsed().as_secs_f64();
        println!("Indexation en cours : {} fichiers ({:.0} fichiers/sec)", 
            total_files, *total_files as f64 / elapsed);

        let mut current_block = SearchBlock::new();
        
        for (entry, path) in entries {
            let name = path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
                
            let path_id = self.path_table.add_path(path);
            let mut entry = entry.clone();
            entry.set_path_id(path_id);
            
            if !current_block.add_entry(entry, &path) {
                self.blocks.push(current_block);
                current_block = SearchBlock::new();
                current_block.add_entry(entry, &path);
            }
        }
        
        if !current_block.entries.is_empty() {
            self.blocks.push(current_block);
        }
        
        Ok(())
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut file = File::create(path)?;
        
        // Écrire l'en-tête
        self.header.write_to(&mut file)?;

        // Écrire les blocs
        let block_offset = file.stream_position()?;
        for block in &self.blocks {
            block.write_to(&mut file)?;
        }

        // Écrire la table des chemins
        let path_offset = file.stream_position()?;
        self.path_table.write_to(&mut file)?;

        // Mettre à jour les offsets
        file.seek(SeekFrom::Start(16))?;
        file.write_all(&path_offset.to_le_bytes())?;
        file.write_all(&block_offset.to_le_bytes())?;

        Ok(())
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let mut cursor = std::io::Cursor::new(&mmap[..]);

        let header = IndexHeader::read_from(&mut cursor)?;
        
        if &header.magic != b"EFU\0" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid index file"));
        }

        let mut blocks_cursor = std::io::Cursor::new(&mmap[header.block_table_offset as usize..]);
        let blocks = SearchBlock::read_multiple(&mut blocks_cursor, header.block_count)?;

        let mut path_table_cursor = std::io::Cursor::new(&mmap[header.path_table_offset as usize..]);
        let path_table = PathTable::read_from(&mut path_table_cursor)?;

        Ok(Self {
            header,
            blocks,
            path_table: Arc::new(path_table),
            mmap: Some(mmap),
        })
    }

    pub fn add_entry(&mut self, mut entry: FileEntry, path: &Path) -> io::Result<()> {
        let path_str = path.to_string_lossy().into_owned();
        let name = path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        // Ajouter le chemin à la table des chemins
        let path_id = self.path_table.add_path(path);
        entry.set_path_id(path_id);

        // Ajouter au cache d'écriture

        Ok(())
    }
}
