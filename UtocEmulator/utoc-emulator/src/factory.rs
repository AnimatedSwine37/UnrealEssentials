use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Seek, SeekFrom};
use anyhow::{anyhow, Context};
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use retoc::{lower_utf16_cityhash, EIoChunkType, EIoStoreTocVersion, FIoChunkHash, FIoChunkId, FIoContainerId, FIoOffsetAndLength, FIoStoreTocCompressedBlockEntry, FIoStoreTocEntryMeta, FIoStoreTocEntryMetaFlags, FPackageId, Toc, UEPath, UEPathBuf};
use retoc::container_header::{EIoContainerHeaderVersion, FIoContainerHeader, StoreEntry};
use retoc::ser::{ReadExt, WriteExt};
use retoc::version::EngineVersion;
use retoc::zen::{ExternalPackageDependency, FExportBundleEntry, FExportBundleHeader, FExportMapEntry, FInternalDependencyArc, FZenPackageImportedPackageNamesContainer, FZenPackageSummary, FZenPackageVersioningInfo};
use utoc_lib::assets::*;
use utoc_lib::assets::AssetEntry;
use utoc_lib::metadata::UtocMetaImportType;
use utoc_lib::store::{StoreEntryBuilder, StoreEntryBuilderNew, StoreEntryBuilderOld};
use crate::ffi::{Array, PartitionBlock};
use crate::GenericResult;
use crate::assets::AssetCollection;
use crate::metadata::{MetadataAdapter, MetadataState};

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
    fn insert_uasset(writer: &mut IoStoreWriter, chunk_id: FIoChunkId, asset_path: &str,
        asset_entry: &AssetEntry, header_version: EIoContainerHeaderVersion) -> GenericResult<()> {
        let adapter = MetadataAdapter;
        let store_entry = match header_version {
            EIoContainerHeaderVersion::Initial =>
                StoreEntryBuilderOld::rebuild_store_entry(asset_entry, chunk_id.get_package_id(), header_version, &adapter)?,
            _ => StoreEntryBuilderNew::rebuild_store_entry(asset_entry, chunk_id.get_package_id(), header_version, &adapter)?,
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
            bar.set_message(asset_name_tr.to_owned());
            bar.set_position(bar.position() + 1);
        }
        writer.finalize(toc, blocks, header)?;
        Ok(())
    }
}