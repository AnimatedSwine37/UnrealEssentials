use std::io::{Read, Seek, SeekFrom};
use anyhow::anyhow;
use retoc::container_header::{EIoContainerHeaderVersion, StoreEntry};
use retoc::FPackageId;
use retoc::ser::ReadExt;
use retoc::zen::{ExternalPackageDependency, FExportBundleEntry, FExportBundleHeader, FExportMapEntry, FInternalDependencyArc, FZenPackageImportedPackageNamesContainer, FZenPackageSummary, FZenPackageVersioningInfo};
use crate::GenericResult;

pub(crate) fn size_of_export_bundle_header(header_version: EIoContainerHeaderVersion) -> u32 {
    match header_version {
        EIoContainerHeaderVersion::Initial => 8,
        _ => 16
    }
}

/// UE4 ONLY! By default, Unreal Essentials 2.0 will resolve asset dependencies using the same logic
/// as 1.x for UE4 games for backwards compatibility.
pub fn get_asset_exports_old<R: Read + Seek>(
    reader: &mut R,
    package_header: &FZenPackageSummary,
    store_entry: &mut StoreEntry,
    header_version: EIoContainerHeaderVersion
) -> GenericResult<()> {
    store_entry.export_count = (package_header.export_bundle_entries_offset
        - package_header.export_map_offset) / size_of::<FExportMapEntry>() as i32;
    // Go through each export bundle to look for the highest index
    reader.seek(SeekFrom::Start(package_header.export_bundle_entries_offset as u64))?;
    let mut predicted_export_bundles: Vec<FExportBundleHeader> = vec![];
    loop {
        let new_entry = FExportBundleHeader::deserialize(reader, header_version)?;
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
    Ok(())
}

/// If no asset metadata is available, use the dependency resolving logic that retoc uses.
/// Asset metadata is required for versions below UE 5.3
pub fn get_asset_exports_new<R: Read + Seek>(
    reader: &mut R,
    summary: &FZenPackageSummary,
    store_entry: &mut StoreEntry,
    header_version: EIoContainerHeaderVersion
) -> GenericResult<()> {
    /// From retoc:
    /// https://github.com/trumank/retoc/blob/master/retoc/src/zen.rs#L871
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
    Ok(())
}