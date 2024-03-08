use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    collections::{HashMap, HashSet},
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
// version: u32 @ 0x0
// alt_auto_import_count: u32 @ 0x4
// manual_import_count: u32 @ 0x8
// compressed_package_count: u32 @ 0xc
// alt_import_assets: [u64; length] @ alt_auto_imports_offset
// manual_import_assets @ manual_imports_offset
//      asset_hash: u64,
//      count: u64,
//      imports: [u64; count]
// compressed_assets: [u64; length] @ compressed_package_offset
// compressed_asset_flags: [u8; length] @ compressed_package_offset + compressed_assets
#[derive(Debug, PartialEq)]
pub struct UtocMetadata {
    // version: u32,
    //alt_auto_imports_offset: u32,
    //manual_imports_offset: u32,
    //compressed_package_offset: u32,
    alt_import_assets: HashSet<u64>,
    manual_import_assets: HashMap<u64, Vec<u64>>,
    compressed_assets: HashMap<u64, UtocMetadataFlags>
}

pub enum UtocMetaImportType {
    GraphPackageUnvalidated,
    GraphPackageValidated,
    Manual
}

impl UtocMetadata {
    pub fn new() -> Self {
        Self { 
            alt_import_assets: HashSet::new(),
            manual_import_assets: HashMap::new(),
            compressed_assets: HashMap::new()
        }
    }
    pub fn add_entries<TByteOrder: byteorder::ByteOrder>(&mut self, data: Vec<u8>) {
        let mut metadata_reader = Cursor::new(data);
        // read header
        let version = metadata_reader.read_u32::<TByteOrder>().unwrap();
        let alt_auto_import_count = metadata_reader.read_u32::<TByteOrder>().unwrap();
        let manual_import_count = metadata_reader.read_u32::<TByteOrder>().unwrap();
        let compressed_package_count = metadata_reader.read_u32::<TByteOrder>().unwrap();
        println!("{}, {}, {}, {}", version, alt_auto_import_count, manual_import_count, compressed_package_count);
        // read alt auto imports
        for i in 0..alt_auto_import_count {
            let curr_import = metadata_reader.read_u64::<TByteOrder>().unwrap();
            //println!("adding import {:X}", &curr_import);
            self.alt_import_assets.insert(curr_import);
        }
        // read manual imports
        for i in 0..manual_import_count {
            let mut manual_imports = vec![];
            let manual_asset = metadata_reader.read_u64::<TByteOrder>().unwrap();
            let import_count = metadata_reader.read_u64::<TByteOrder>().unwrap();
            for j in 0..import_count {
                manual_imports.push(metadata_reader.read_u64::<TByteOrder>().unwrap());
            }
            self.manual_import_assets.insert(manual_asset, manual_imports);
        }
        // read compressed packages
        let mut compressed_assets = Vec::with_capacity(compressed_package_count as usize);
        for i in 0..compressed_package_count {
            compressed_assets.push(metadata_reader.read_u64::<TByteOrder>().unwrap());
        }
        for i in 0..compressed_package_count {
            self.compressed_assets.insert(compressed_assets[i as usize], UtocMetadataFlags::from_bits(metadata_reader.read_u8().unwrap()).unwrap());
        }
    }
    pub fn get_import_type(&self, asset: u64) -> UtocMetaImportType {
        if self.alt_import_assets.contains(&asset) {
            return UtocMetaImportType::GraphPackageValidated;
        } else if self.manual_import_assets.contains_key(&asset) {
            return UtocMetaImportType::Manual;
        }
        return UtocMetaImportType::GraphPackageUnvalidated;
    }
    pub fn get_manual_import(&self, hash: u64) -> Option<&Vec<u64>> {
        self.manual_import_assets.get(&hash)
    }
}