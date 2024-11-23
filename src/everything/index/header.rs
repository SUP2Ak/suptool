use std::io::{self, Read, Write};

#[repr(C, packed)]
pub(crate) struct IndexHeader {
    pub magic: [u8; 4],           // "EFU\0"
    pub version: u32,             // Version du format
    pub block_count: u32,         // Nombre de blocs
    pub entry_count: u64,         // Nombre total d'entrées
    pub path_table_offset: u64,   // Offset de la table des chemins
    pub block_table_offset: u64,  // Offset de la table des blocs
}

impl IndexHeader {
    pub fn new() -> Self {
        Self {
            magic: *b"EFU\0",
            version: 1,
            block_count: 0,
            entry_count: 0,
            path_table_offset: 0,
            block_table_offset: 0,
        }
    }

    pub fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&self.magic)?;
        writer.write_all(&self.version.to_le_bytes())?;
        writer.write_all(&self.block_count.to_le_bytes())?;
        writer.write_all(&self.entry_count.to_le_bytes())?;
        writer.write_all(&self.path_table_offset.to_le_bytes())?;
        writer.write_all(&self.block_table_offset.to_le_bytes())?;
        Ok(())
    }

    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        
        let mut bytes = [0u8; 4];
        reader.read_exact(&mut bytes)?;
        let version = u32::from_le_bytes(bytes);
        
        reader.read_exact(&mut bytes)?;
        let block_count = u32::from_le_bytes(bytes);
        
        let mut bytes8 = [0u8; 8];
        reader.read_exact(&mut bytes8)?;
        let entry_count = u64::from_le_bytes(bytes8);
        
        reader.read_exact(&mut bytes8)?;
        let path_table_offset = u64::from_le_bytes(bytes8);
        
        reader.read_exact(&mut bytes8)?;
        let block_table_offset = u64::from_le_bytes(bytes8);

        if &magic != b"EFU\0" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic number"));
        }

        Ok(Self {
            magic,
            version,
            block_count,
            entry_count,
            path_table_offset,
            block_table_offset,
        })
    }
}