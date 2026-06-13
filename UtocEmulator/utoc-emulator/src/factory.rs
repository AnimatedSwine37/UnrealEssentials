use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Seek, SeekFrom};
use anyhow::Context;
use byteorder::ReadBytesExt;
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use retoc::{lower_utf16_cityhash, EIoChunkType, EIoStoreTocVersion, FIoChunkHash, FIoChunkId, FIoContainerId, FIoOffsetAndLength, FIoStoreTocCompressedBlockEntry, FIoStoreTocEntryMeta, FIoStoreTocEntryMetaFlags, FPackageId, Toc, UEPath, UEPathBuf};
use retoc::container_header::{EIoContainerHeaderVersion, FIoContainerHeader, StoreEntry};
use retoc::ser::{ReadExt, WriteExt};
use retoc::version::EngineVersion;
use retoc::zen::{ExternalPackageDependency, FExportBundleHeader, FExportMapEntry, FInternalDependencyArc, FZenPackageSummary};
use crate::ffi::{Array, PartitionBlock};
use crate::{log, GenericResult};
use crate::assets::{AssetCollection, AssetEntry, MOUNT_POINT, UASSET_EXTENSION, UBULK_EXTENSION, UMAP_EXTENSION, UPTNL_EXTENSION};
use crate::metadata::{UtocMetaImportType, UtocMetadata};

/// UE4 ONLY, for compatibility with Unreal Essentials 1.x.
/// UE5 uses the same logic for resolving package dependencies that retoc uses
#[derive(Debug)]
pub(crate) struct LegacyImportIdResolver;

fn from_buffer_text<R: Read + Seek>(reader: &mut R) -> GenericResult<String> {
    let len_raw: u16 = reader.read_u16::<byteorder::BigEndian>()?;
    let len = len_raw & 0x7fff;
    let b_is_wide = if (len_raw & 0x8000) != 0 { true } else { false };
    if b_is_wide && reader.stream_position()? % 2 != 0 {
        let _: u8 = reader.de()?; // align to nearest 0x2 if wide
    }
    let mut buf: Vec<u8> = vec![0; if b_is_wide { 2 * len } else { len } as usize];
    reader.read_exact(&mut buf)?;
    Ok(unsafe {
        if b_is_wide {
            // Safety: length of buf was multiplied by 2 to read n u16s, dividing by 2 is just undoing that
            String::from_utf16_lossy(std::slice::from_raw_parts(buf.as_ptr() as *const u16, buf.len() / 2))
        } else {
            String::from_utf8_unchecked(buf)
        }
    })
}

impl LegacyImportIdResolver {
    /// Set all graph package imports as container summary imports. This behavior will likely be kept to
    /// maintain compatibility with older P3RE mods since validation does break some other assets
    fn from_graph_packages_unvalidated(dependencies: &[ExternalPackageDependency]) -> Vec<FPackageId> {
        dependencies.iter().map(|d| d.from_package_id).collect()
    }

    /// From graph package imports, but with validating the name entries for matching file names
    /// Not all files actually have the same graph package imports as container header imports (Unreal shenanigans)
    /// However, there's an edge case with files ending in an underscore + number, where Unreal won't serialize that
    /// last portion of the string. I'd need an external tool to fix this
    fn from_graph_packages_validated<T: Read + Seek>(reader: &mut T, summary: &FZenPackageSummary,
                                                     dependencies: &[ExternalPackageDependency]) -> Vec<FPackageId> {
        reader.seek(SeekFrom::Start(summary.name_map_names_offset as u64)).unwrap();
        let name_count = (summary.name_map_names_size as usize / size_of::<u64>()) - 1;
        let names: Vec<String> = (0..name_count).filter_map(|_| from_buffer_text(reader).ok()).collect();
        let mut path_name_hashes = vec![];
        loop { // we only want to hash file paths, which Unreal always serializes at the beginning
            if path_name_hashes.len() == names.len() || !names[path_name_hashes.len()].starts_with("/") {
                break;
            }
            path_name_hashes.push(lower_utf16_cityhash(&names[path_name_hashes.len()]));
        }
        dependencies.iter()
            .filter_map(|d| {
                match path_name_hashes.contains(&d.from_package_id.0) {
                    true => Some(d.from_package_id),
                    false => None
                }
            })
            .collect()
    }

    /// If required, import ids can be manually specified from the metadata file. Trying to generate a
    /// UCAS file with no external metadata was always going to be a challenge
    fn from_metadata_v1(meta: &UtocMetadata, asset: FPackageId) -> Vec<FPackageId> {
        meta.get_manual_v1_import(asset).map_or(
            vec![], |n| n.iter().map(|v| *v).collect())
    }

    /// Using the new metadata schema (just retoc::container_header::StoreEntry)
    fn from_metadata_v2(meta: &UtocMetadata, asset: FPackageId) -> Vec<FPackageId> {
        meta.get_manual_v2_import(asset).map_or(
            vec![], |n| n.imported_packages)
    }
}

pub(crate) fn size_of_export_bundle_header(header_version: EIoContainerHeaderVersion) -> u32 {
    match header_version {
        EIoContainerHeaderVersion::Initial => 8,
        _ => 16
    }
}

fn align_usize(value: usize, alignment: usize) -> usize {
    (value + alignment - 1) & !(alignment - 1)
}

#[derive(Debug)]
pub enum ChunkData<'a> {
    Asset(&'a AssetEntry),
    Header(&'a [u8])
}

impl<'a> ChunkData<'a> {
    pub fn size(&self) -> u64 {
        match self {
            Self::Asset(a) => a.size,
            Self::Header(h) => h.len() as u64
        }
    }
}

pub struct IoStoreWriter {
    toc_stream: BufWriter<Vec<u8>>,
    partitions: Vec<PartitionBlock>,
    cas_pointer: u64,
    toc: Toc,
    container_header: Option<FIoContainerHeader>,
}

impl IoStoreWriter {
    pub fn new(toc_version: EIoStoreTocVersion,
        container_header_version: Option<EIoContainerHeaderVersion>, mount_point: UEPathBuf)
        -> GenericResult<Self> {
        let toc_stream = BufWriter::new(vec![]);

        let mut toc = Toc::new();
        toc.compression_block_size = 0x10000;
        toc.version = toc_version;
        toc.container_id = FIoContainerId::from_name("Game");
        toc.directory_index.mount_point = mount_point;
        toc.partition_size = u64::MAX;

        let container_header = container_header_version.map(|v| FIoContainerHeader::new(v, toc.container_id));
        Ok(Self {
            toc_stream,
            partitions: vec![],
            cas_pointer: 0,
            toc,
            container_header
        })
    }

    pub fn write_chunk(&mut self, chunk_id: FIoChunkId, path: Option<&UEPath>, data: ChunkData) -> GenericResult<(u64)> {
        if let Some(path) = path {
            let index = &mut self.toc.directory_index;
            let relative_path = path.strip_prefix(&index.mount_point)
                .with_context(|| format!("mount point {} does not contain path {path}", index.mount_point))?;
            index.add_file(relative_path, self.toc.chunks.len() as u32);
        }

        let start_block = self.toc.compression_blocks.len();
        let cas_start = self.cas_pointer;
        for i in 0..(data.size() / self.toc.compression_block_size as u64) + 1 {
            let block_len = (data.size() - (i * self.toc.compression_block_size as u64)).min(self.toc.compression_block_size as u64);
            let (compressed_size, uncompressed_size) = (block_len as u32, block_len as u32);
            let compression_method_index = 0; // "None"
            self.toc.compression_blocks.push(FIoStoreTocCompressedBlockEntry::new(self.cas_pointer, compressed_size, uncompressed_size, compression_method_index));
            self.cas_pointer += compressed_size as u64;
        }

        let offset_and_length = FIoOffsetAndLength::new(start_block as u64 * self.toc.compression_block_size as u64, data.size());
        self.toc.chunks.push(chunk_id.with_version(self.toc.version));
        self.toc.chunk_offset_lengths.push(offset_and_length);
        self.toc.chunk_metas.push(FIoStoreTocEntryMeta {
            chunk_hash: FIoChunkHash([0; 32]),
            flags: FIoStoreTocEntryMetaFlags::empty(),
        });
        Ok(cas_start)
    }

    pub fn write_file_chunk(&mut self, chunk_id: FIoChunkId, path: &UEPath, data: &AssetEntry) -> GenericResult<()> {
        let start = self.write_chunk(chunk_id, Some(path), ChunkData::Asset(data))?;
        self.partitions.push(PartitionBlock::new(data.os_path.to_str().unwrap(), start, data.size));
        Ok(())
    }

    pub fn write_file_container(&mut self, data: &[u8]) -> GenericResult<()> {
        if let Some(container_header) = &self.container_header {
            let chunk_id = FIoChunkId::create(container_header.container_id.0, 0, EIoChunkType::ContainerHeader);
            self.write_chunk(chunk_id, None, ChunkData::Header(data))?;
        }
        Ok(())
    }

    pub fn write_package_chunk(&mut self, chunk_id: FIoChunkId, path: &UEPath, data: &AssetEntry, store: &StoreEntry) -> GenericResult<()> {
        let container_header = self.container_header.as_mut()
            .expect("FIoContainerHeader is required to write package chunks");
        container_header.add_package(FPackageId(chunk_id.get_chunk_id()), store.clone());
        self.write_file_chunk(chunk_id, path, data)
    }

    pub fn finalize(mut self, toc: &mut Array<u8>, blocks: &mut Array<PartitionBlock>, header: &mut Array<u8>) -> GenericResult<()> {
        if let Some(container_header) = &self.container_header {
            let mut chunk_buffer = vec![];
            container_header.serialize(&mut Cursor::new(&mut chunk_buffer))?;
            // container header is always aligned for AES for some reason
            chunk_buffer.resize(align_usize(chunk_buffer.len(), 16), 0);
            self.write_file_container(&chunk_buffer)?;
            *header = chunk_buffer.into();
        }
        self.toc_stream.ser(&self.toc)?;
        *toc = self.toc_stream.into_inner()?.into();
        *blocks = self.partitions.into();
        Ok(())
    }
}

pub struct IoStoreFactory;
impl IoStoreFactory {
    /// UE4 ONLY! By default, Unreal Essentials 2.0 will resolve asset dependencies using the same logic
    /// as 1.x for UE4 games for backwards compatibility.
    fn rebuild_store_entry(
        asset_entry: &AssetEntry,
        package_id: FPackageId,
        header_version: EIoContainerHeaderVersion
    ) -> GenericResult<StoreEntry> {
        let mut reader = BufReader::with_capacity(
            0x2000, File::open(asset_entry.os_path.as_path())?);
        let mut store_entry = StoreEntry::default();
        let package_header = FZenPackageSummary::deserialize(
            &mut reader, header_version)?;
        store_entry.export_count = (package_header.export_bundle_entries_offset
            - package_header.export_map_offset) / size_of::<FExportMapEntry>() as i32;
        // Go through each export bundle to look for the highest index
        reader.seek(SeekFrom::Start(package_header.export_bundle_entries_offset as u64))?;
        let mut predicted_export_bundles: Vec<FExportBundleHeader> = vec![];
        loop {
            let new_entry = FExportBundleHeader::deserialize(&mut reader, header_version)?;
            if new_entry.entry_count == 0 {
                break;
            }
            if let Some(last) = predicted_export_bundles.last()
                && new_entry.first_entry_index != last.entry_count {
                break;
            }
            predicted_export_bundles.push(new_entry);
        }
        let export_bundle_bytes = (package_header.graph_data_offset - package_header.export_bundle_entries_offset) as u32;
        let actual_entries = (export_bundle_bytes - predicted_export_bundles.len() as u32 * size_of_export_bundle_header(header_version))
            / size_of_export_bundle_header(header_version);
        let mut export_bundle_count = predicted_export_bundles.len();
        loop {
            if export_bundle_count == 0 || actual_entries == predicted_export_bundles[..export_bundle_count]
                .iter().map(|v| v.entry_count).sum::<u32>() {
                break;
            }
            export_bundle_count -= 1;
        }
        store_entry.export_bundle_count = export_bundle_count.max(1) as i32;

        reader.seek(SeekFrom::Start(package_header.graph_data_offset as u64))?;
        let mut package_dependencies: Vec<ExternalPackageDependency> = vec![];
        let imported_packages_count: i32 = reader.de()?;
        for _ in 0..imported_packages_count {
            let imported_package_id = reader.de()?;
            let legacy_arcs: Vec<FInternalDependencyArc> = reader.de()?;
            package_dependencies.push(ExternalPackageDependency {
                from_package_id: imported_package_id,
                external_dependency_arcs: vec![],
                legacy_dependency_arcs: legacy_arcs
            });
        }
        let metadata = UtocMetadata::instance();
        store_entry.imported_packages = match metadata.as_ref().unwrap().get_import_type(package_id) {
            UtocMetaImportType::GraphPackageUnvalidated => LegacyImportIdResolver::from_graph_packages_unvalidated(&package_dependencies),
            UtocMetaImportType::GraphPackageValidated => LegacyImportIdResolver::from_graph_packages_validated(&mut reader, &package_header, &package_dependencies),
            UtocMetaImportType::ManualV1 => LegacyImportIdResolver::from_metadata_v1(metadata.as_ref().unwrap(), package_id),
            UtocMetaImportType::ManualV2 => LegacyImportIdResolver::from_metadata_v2(metadata.as_ref().unwrap(), package_id),
        };
        drop(metadata);
        store_entry.export_bundles_size = asset_entry.size;
        Ok(store_entry)
    }

    fn insert_uasset(writer: &mut IoStoreWriter, chunk_id: FIoChunkId, asset_path: &str,
        asset_entry: &AssetEntry, header_version: EIoContainerHeaderVersion) -> GenericResult<()> {

        let store_entry = match header_version {
            EIoContainerHeaderVersion::Initial => Self::rebuild_store_entry(asset_entry, chunk_id.get_package_id(), header_version)?,
            _ => StoreEntry::default()
        };
        writer.write_package_chunk(chunk_id, UEPath::new(asset_path), asset_entry, &store_entry)?;
        Ok(())
    }

    fn insert_bulk(writer: &mut IoStoreWriter, chunk_id: FIoChunkId, asset_path: &str, asset_entry: &AssetEntry) -> GenericResult<()> {
        _ = writer.write_file_chunk(chunk_id, UEPath::new(asset_path), asset_entry);
        Ok(())
    }

    pub(crate) fn build(
        version: EngineVersion,
        toc: &mut Array<u8>,
        blocks: &mut Array<PartitionBlock>,
        header: &mut Array<u8>
    ) -> GenericResult<()> {
        let mut writer = IoStoreWriter::new(
            version.toc_version(),
            Some(version.container_header_version()),
            UEPath::new(MOUNT_POINT).into()
        )?;

        let bar = ProgressBar::new(AssetCollection::instance().as_ref().unwrap().len() as u64);
        let color_fmt = match Term::stdout().features().true_colors_supported() {
            true => "#DA70D6/#9932CC", false => "135/90"
        };
        let template_fmt = format!("[{{elapsed_precise}}] {{bar:40.{}}} {{pos:>7}}/{{len:7}} ({{percent_precise}}%) {{msg}}", color_fmt);
        let bar_style = ProgressStyle::with_template(&template_fmt)?
            .progress_chars("##-");
        bar.set_style(bar_style);
        bar.tick();

        for (asset_name, asset_entry) in AssetCollection::instance().as_ref().unwrap() {
            // log!(Debug, "IoStoreFactory::build: {}", asset_name);
            let chunk_type = match asset_entry.os_path.extension().map(|v| v.to_str().unwrap())
                .and_then(|ext| match ext {
                    UASSET_EXTENSION | UMAP_EXTENSION => Some(EIoChunkType::ExportBundleData),
                    UBULK_EXTENSION => Some(EIoChunkType::BulkData),
                    UPTNL_EXTENSION => Some(EIoChunkType::OptionalBulkData),
                    _ => None
                }) {
                Some(v) => v,
                None => continue
            };
            let asset_name_tr = asset_name[MOUNT_POINT.len() - 1..].rsplit_once('.').unwrap().0;
            let chunk_id = FIoChunkId::create(lower_utf16_cityhash(asset_name_tr), 0, chunk_type.clone());
            match chunk_type {
                EIoChunkType::ExportBundleData => Self::insert_uasset(
                    &mut writer, chunk_id, asset_name, asset_entry, version.container_header_version())?,
                EIoChunkType::BulkData | EIoChunkType::OptionalBulkData => Self::insert_bulk(
                    &mut writer, chunk_id, asset_name, asset_entry)?,
                _ => ()
            };
            bar.set_message(asset_name.clone());
            bar.set_position(bar.position() + 1);
        }
        writer.finalize(toc, blocks, header)?;
        Ok(())
    }
}