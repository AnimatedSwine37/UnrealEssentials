use std::fs::{File, Metadata};
use std::io::{BufReader, Read, Seek, SeekFrom};
use anyhow::anyhow;
use byteorder::ReadBytesExt;
use retoc::container_header::{EIoContainerHeaderVersion, StoreEntry};
use retoc::{lower_utf16_cityhash, FPackageId};
use retoc::ser::ReadExt;
use retoc::zen::{ExternalPackageDependency, FExportBundleEntry, FExportBundleHeader, FExportMapEntry, FInternalDependencyArc, FZenPackageImportedPackageNamesContainer, FZenPackageSummary, FZenPackageVersioningInfo};
use crate::assets::AssetEntry;
use crate::GenericResult;
use crate::metadata::UtocMetadata;

pub trait MetadataProvider {
    fn check_v2_import(&self, package_id: FPackageId) -> Option<StoreEntry>;
    fn get_imports_ue4<T: Read + Seek>(
        &self,
        store_entry: &mut StoreEntry,
        reader: &mut T,
        package_id: FPackageId,
        package_header: &FZenPackageSummary,
        package_dependencies: &[ExternalPackageDependency],
    );
}

pub fn size_of_export_bundle_header_ue4(header_version: EIoContainerHeaderVersion) -> u32 {
    match header_version {
        EIoContainerHeaderVersion::Initial => 8,
        _ => unreachable!()
    }
}

pub trait StoreEntryBuilder {
    fn rebuild_store_entry<T: MetadataProvider>(
        asset_entry: &AssetEntry,
        package_id: FPackageId,
        header_version: EIoContainerHeaderVersion,
        metadata_provider: &T
    ) -> GenericResult<StoreEntry>;
}

pub struct StoreEntryBuilderOld;

impl StoreEntryBuilder for StoreEntryBuilderOld {
    fn rebuild_store_entry<T: MetadataProvider>(
        asset_entry: &AssetEntry,
        package_id: FPackageId,
        header_version: EIoContainerHeaderVersion,
        metadata_provider: &T
    ) -> GenericResult<StoreEntry> {
        if let Some(store) = metadata_provider.check_v2_import(package_id) {
            return Ok(store);
        }
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
        let actual_entries = (export_bundle_bytes - predicted_export_bundles.len() as u32 * size_of_export_bundle_header_ue4(header_version))
            / size_of_export_bundle_header_ue4(header_version);
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
        metadata_provider.get_imports_ue4(
            &mut store_entry,
            &mut reader,
            package_id,
            &package_header,
            package_dependencies.as_slice()
        );
        store_entry.export_bundles_size = asset_entry.size;
        Ok(store_entry)
    }
}

pub struct StoreEntryBuilderNew;

impl StoreEntryBuilder for StoreEntryBuilderNew {
    fn rebuild_store_entry<T: MetadataProvider>(
        asset_entry: &AssetEntry,
        package_id: FPackageId,
        header_version: EIoContainerHeaderVersion,
        metadata_provider: &T
    ) -> GenericResult<StoreEntry> {
        if let Some(store) = metadata_provider.check_v2_import(package_id) {
            return Ok(store);
        }
        if header_version < EIoContainerHeaderVersion::NoExportInfo {
            return Err(anyhow!("Asset metadata is required for UE5 versions before 5.3!").into_boxed_dyn_error());
        }
        let mut reader = BufReader::with_capacity(
            0x2000, File::open(asset_entry.os_path.as_path())?);
        let mut store_entry = StoreEntry::default();

        /// From retoc:
        /// https://github.com/trumank/retoc/blob/master/retoc/src/zen.rs#L871
        let summary = FZenPackageSummary::deserialize(
            &mut reader, header_version)?;
        let _: Option<FZenPackageVersioningInfo> = // optional versioning info
            if summary.has_versioning_info != 0 { Some(reader.de()?) } else { None };
        // For UE PackageVersion >= EUnrealEngineObjectUE5Version::VERSE_CELLS is checked here, however at this point we do not know the package file version for the package
        // We do know the container header version though, and VERSE_CELLS is introduced as a part of UE 5.6, which ships with header_version == EIoContainerHeaderVersion::SoftPackageReferencesOffset
        // so we can check for that instead and get the correct result without having to know the engine version at this point
        let cell_import_map_offset = if header_version >= EIoContainerHeaderVersion::SoftPackageReferencesOffset {
            reader.de()? } else { summary.export_bundle_entries_offset };
        store_entry.export_count = ((cell_import_map_offset - summary.export_map_offset) as usize / size_of::<FExportMapEntry>()) as i32;
        let expected_export_bundle_entries_count = store_entry.export_count * 2; // Each export must have Create and Serialize
        reader.seek(SeekFrom::Start(summary.export_bundle_entries_offset as u64))?;
        // New style export bundles entries, UE5.0+. Export bundle entries count is derived from the graph data offset
        let export_bundle_entries_end_offset = if summary.dependency_bundle_headers_offset > 0 {
            summary.dependency_bundle_headers_offset } else { summary.graph_data_offset };
        store_entry.export_bundle_count = ((export_bundle_entries_end_offset
            - summary.export_bundle_entries_offset) as usize
            / size_of::<FExportBundleEntry>()) as i32;
        if store_entry.export_bundle_count != expected_export_bundle_entries_count {
            return Err(anyhow!(
                "Expected to have Create and Serialize commands in export bundle for each export in the package. Got only {} export bundle entries with {} exports",
                store_entry.export_bundle_count,
                store_entry.export_count
            ).into_boxed_dyn_error());
        }
        let mut imported_package_names: FZenPackageImportedPackageNamesContainer = FZenPackageImportedPackageNamesContainer::default();
        if summary.imported_package_names_offset > 0 {
            reader.seek(SeekFrom::Start(summary.imported_package_names_offset as u64))?;
            imported_package_names = reader.de()?;
        }

        // ImportedPackageNames is required for this to work, which is true for UE 5.3+
        store_entry.imported_packages = imported_package_names.imported_package_names.iter()
            .map(|x| FPackageId::from_name(x)).collect();
        store_entry.export_bundles_size = asset_entry.size;
        Ok(store_entry)
    }
}

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

/// UE4 ONLY, for compatibility with Unreal Essentials 1.x.
/// UE5 uses the same logic for resolving package dependencies that retoc uses
#[derive(Debug)]
pub struct LegacyImportIdResolver;

impl LegacyImportIdResolver {
    /// Set all graph package imports as container summary imports. This behavior will likely be kept to
    /// maintain compatibility with older P3RE mods since validation does break some other assets
    pub fn from_graph_packages_unvalidated(dependencies: &[ExternalPackageDependency]) -> Vec<FPackageId> {
        dependencies.iter().map(|d| d.from_package_id).collect()
    }

    /// From graph package imports, but with validating the name entries for matching file names
    /// Not all files actually have the same graph package imports as container header imports (Unreal shenanigans)
    /// However, there's an edge case with files ending in an underscore + number, where Unreal won't serialize that
    /// last portion of the string. I'd need an external tool to fix this
    pub fn from_graph_packages_validated<T: Read + Seek>(reader: &mut T, summary: &FZenPackageSummary,
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
    pub fn from_metadata_v1(meta: &UtocMetadata, asset: FPackageId) -> Vec<FPackageId> {
        meta.get_manual_v1_import(asset).map_or(
            vec![], |n| n.iter().map(|v| *v).collect())
    }
}


#[cfg(target_os = "linux")]
pub fn os_file_size(metadata: &Metadata) -> u64 {
    std::os::linux::fs::MetadataExt::st_size(&meta)
}

#[cfg(target_os = "windows")]
pub fn os_file_size(metadata: &Metadata) -> u64 {
    std::os::windows::fs::MetadataExt::file_size(metadata)
}