use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    collections::HashMap,
    io::{Cursor, Seek, SeekFrom},
    mem,
    sync::Mutex
};

bitflags! {
    #[derive(Debug, PartialEq)]
    struct UtocMetadataFlags : u8 {
        const None = 0;
        const UseCompressionZlib = 1 << 0;
        const UseCompressionOodle = 1 << 1;
        const UseCompressionLZ4 = 1 << 2;
        const UseCompressionGzip = 1 << 3;
    }
}

pub static UTOC_METADATA: Mutex<Option<UtocMetadata>> = Mutex::new(None);

// .utocmeta structure:
// count: u64
// chunk_hashes: [u64; length]
// flags: [u8; length]
#[derive(Debug, PartialEq)]
pub struct UtocMetadata {
    chunk_flags: HashMap<u64, UtocMetadataFlags>
}

impl UtocMetadata {
    pub fn new() -> Self {
        Self { chunk_flags: HashMap::new() }
    }
    pub fn add_entries(&mut self, data: Vec<u8>) {
        let mut metadata_reader = Cursor::new(data);
        let val_count = metadata_reader.read_u64::<byteorder::LittleEndian>().unwrap();
        let flags_offset = mem::size_of::<u64>() as u64 + mem::size_of::<u64>() as u64 * val_count;
        for i in 0..val_count {
            metadata_reader.seek(SeekFrom::Start(mem::size_of::<u64>() as u64 + 0xc * i));
            let curr_hash = metadata_reader.read_u64::<byteorder::LittleEndian>().unwrap();
            metadata_reader.seek(SeekFrom::Start(flags_offset + i));
            let chunk_flag = UtocMetadataFlags::from_bits_truncate(metadata_reader.read_u8().unwrap());
            self.chunk_flags.insert(curr_hash, chunk_flag);
        }
    }
}