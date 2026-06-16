use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::sync::MutexGuard;
use anyhow::anyhow;
use retoc::container_header::{EIoContainerHeaderVersion, StoreEntries, StoreEntry};
use retoc::FPackageId;
use retoc::ser::{ReadExt, Readable, WriteExt, Writeable};
use retoc::version::EngineVersion;
use crate::GenericResult;

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
#[derive(Debug, PartialEq)]
pub struct UtocMetadata {
    alt_import_assets: HashSet<FPackageId>,
    manual_import_assets: HashMap<FPackageId, Vec<FPackageId>>,
    fast_resolve_assets: StoreEntries
}

pub enum UtocMetaImportType {
    GraphPackageUnvalidated,
    GraphPackageValidated,
    ManualV1,
    ManualV2
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum UtocMetaVersion {
    Initial = 1,
    FastResolver
}

impl Writeable for UtocMetaVersion {
    fn ser<S: Write>(&self, stream: &mut S) -> anyhow::Result<()> {
        stream.ser(&(*self as u32))
    }
}

impl Readable for UtocMetaVersion {
    fn de<S: Read>(stream: &mut S) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let raw: u32 = stream.de()?;
        match raw {
            1 => Ok(UtocMetaVersion::Initial),
            2 => Ok(UtocMetaVersion::FastResolver),
            _ => Err(anyhow!("Unknown utocmeta version {}, expected version 1 or 2", raw))
        }
    }
}

impl Default for UtocMetadata {
    fn default() -> Self {
        Self {
            alt_import_assets: HashSet::new(),
            manual_import_assets: HashMap::new(),
            fast_resolve_assets: StoreEntries::default()
        }
    }
}

impl UtocMetadata {
    pub fn add_from_utocmeta(&mut self, data: &[u8], engine: EngineVersion) -> GenericResult<()> {
        let mut reader = Cursor::new(data);
        // read header
        let version: UtocMetaVersion = reader.de()?;
        let alt_auto_import_count: u32 = reader.de()?;
        let manual_v1_count: u32 = reader.de()?;
        let compressed_package_count: u32 = match version {
            UtocMetaVersion::Initial => reader.de()?,
            UtocMetaVersion::FastResolver => 0
        };
        // read alt auto imports
        for _ in 0..alt_auto_import_count {
            self.alt_import_assets.insert(reader.de()?);
        }
        // read manual imports
        for _ in 0..manual_v1_count {
            let (manual_asset, import_count) = (reader.de()?, reader.de()?);
            let manual_imports: Vec<FPackageId> = (0..import_count).filter_map(|_| reader.de().ok()).collect();
            self.manual_import_assets.insert(manual_asset, manual_imports);
        }
        match version {
            UtocMetaVersion::Initial => {
                // read compressed packages - this is dummied out since this field never did anything
                for _ in 0..compressed_package_count {
                    let _: Vec<FPackageId> = reader.de()?;
                }
            },
            UtocMetaVersion::FastResolver => {
                for (id, entry) in StoreEntries::deserialize(
                    &mut reader, engine.container_header_version())?.into_iter() {
                    self.fast_resolve_assets.insert(id, entry);
                }
            }
        }
        Ok(())
    }

    pub fn add_from_uassetmeta(&mut self, key: FPackageId, path: &Path) -> GenericResult<()> {
        self.fast_resolve_assets.insert(key, Cursor::new(std::fs::read(path)?).de()?);
        Ok(())
    }

    pub fn add_from_store_entry(&mut self, key: FPackageId, store: StoreEntry) -> GenericResult<()> {
        self.fast_resolve_assets.insert(key, store);
        Ok(())
    }


    pub fn get_import_type(&self, asset: FPackageId) -> UtocMetaImportType {
        if self.fast_resolve_assets.contains(asset) {
            return UtocMetaImportType::ManualV2;
        } else if self.alt_import_assets.contains(&asset) {
            return UtocMetaImportType::GraphPackageValidated;
        } else if self.manual_import_assets.contains_key(&asset) {
            return UtocMetaImportType::ManualV1;
        }
        UtocMetaImportType::GraphPackageUnvalidated
    }


    pub fn get_manual_v1_import(&self, asset: FPackageId) -> Option<&[FPackageId]> {
        self.manual_import_assets.get(&asset).map(|v| v.as_slice())
    }

    pub fn get_manual_v2_import(&self, asset: FPackageId) -> Option<StoreEntry> {
        self.fast_resolve_assets.get(asset)
    }

    pub fn serialize<S: Write>(&self, stream: &mut S, version: EIoContainerHeaderVersion) -> GenericResult<()> {
        stream.ser(&UtocMetaVersion::FastResolver)?;
        stream.ser(&(self.alt_import_assets.len() as u32))?;
        stream.ser(&(self.manual_import_assets.len() as u32))?;
        // stream.ser(&(0u32))?;
        self.fast_resolve_assets.serialize(stream, version)?;
        Ok(())
    }
}