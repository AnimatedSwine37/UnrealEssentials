// IO Store Package Header types

type PackageObjectIndex = u64; // TODO: make proper struct for this
type ObjectFlags = u32; // this probably doesn't need to be defined...
type ExportFilterFlags = u8; // and this one too...

// Structure of IO Store Asset:
// Header: FPackageSummary (requires converting PAK Package to IO Package)
// Data: contents of .uexp - 4 magic bytes at end
// Texture Bulk: all of .ubulk

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use crate::{
    pak_package::{FObjectImport, FObjectExport, GameName, NameMap},
    string::{ FMappedName, FStringDeserializerText, FString16, Hasher16 },
    toc_factory::{TocResolverCommon, TocResolverType2}
};
use std::{
    error::Error,
    fs::File,
    fmt,
    io::{BufReader, Cursor, ErrorKind, Read, Seek, SeekFrom, Write}
};
// IoStoreObjectIndex is a 64 bit value consisting of a hash of a target string for the lower 62 bits and an object type for the highest 2
// expect for Empty which represents a null value and Export which contains an index to another item on the export tree
// This struct is used to fully represent an import on an IO Store package, and is the basic structure for several named fields in export
#[derive(Debug, Clone, PartialEq)]
pub enum IoStoreObjectIndex {
    Export(u64),            // type 0 (index, Export -> Export)
    ScriptImport(String),   // type 1 (string hash, represents Import mounted at /Script/...)
    PackageImport(String),  // type 2 (string hash, represents Import mounted at /Game/...)
    Empty                   // type 3 (-1)
}

impl IoStoreObjectIndex {
    pub fn from_buffer<R: Read + Seek, E: byteorder::ByteOrder>(&self, reader: &mut R) -> IoStoreObjectIndex {
        let raw_value = reader.read_u64::<E>().unwrap();
        let obj_type = raw_value & (3 << 62);
        match obj_type {
            0 => IoStoreObjectIndex::Export(0), // can't derive string name from hash, will likely need to separate this off to another type for container header building
            1 => IoStoreObjectIndex::ScriptImport(String::new()),
            2 => IoStoreObjectIndex::PackageImport(String::new()),
            3 => IoStoreObjectIndex::Empty,
            _ => panic!("Invalid obj type {}", obj_type),
        }
    }
    // TOOO: upgrade trait bounds to Write + Seek
    pub fn to_buffer<W: Write, E: byteorder::ByteOrder>(&self, writer: &mut W) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Export(i) => writer.write_u64::<E>(*i as u64)?,
            Self::ScriptImport(v) => writer.write_u64::<E>(IoStoreObjectIndex::generate_hash(v, 1))?,
            Self::PackageImport(v) => writer.write_u64::<E>(IoStoreObjectIndex::generate_hash(v, 2))?,
            Self::Empty => writer.write_u64::<E>(u64::MAX)?,
        }
        Ok(())
    }

    fn generate_hash(import: &str, obj_type: u64) -> u64 {
        //println!("make hash for {}", import);
        let to_hash = String::from(import).to_lowercase();
        // hash chars are sized according to if the platform supports wide characters, which is usually the case
        let to_hash: Vec<u16> = to_hash.encode_utf16().collect();
        // safety: Vec is contiguous, so a Vec<u8> of length `2 * n` will take the same memory as a Vec<u16> of len `n`
        let to_hash = unsafe { std::slice::from_raw_parts(to_hash.as_ptr() as *const u8, to_hash.len() * 2) };
        // verified: the strings are identical (no null terminator) when using FString16
        let mut hash: u64 = cityhasher::hash(to_hash); // cityhash it
        hash &= !(3 << 62); // first 62 bits are our hash
        hash |= obj_type << 62; // stick the type in high 2 bits
        hash
    }
}

pub struct ObjectImport;
impl ObjectImport {
    // Convert FObjectImport into named ObjectImport
    pub fn from_pak_asset<N: NameMap>(import_map: &Vec<FObjectImport>, name_map: &N) -> Vec<IoStoreObjectIndex> {
        let mut resolves = vec![];
        for (i, v) in import_map.into_iter().enumerate() {
            match v.resolve(name_map, import_map) {
                Ok(obj) => resolves.push(obj),
                Err(e) => panic!("Error converting PAK formatted import to IO Store import on ID {} \nValue {:?}\nReason: {}", i, v, e.to_string())
            }
        }
        resolves
    }

    pub fn map_to_buffer<W: Write, E: byteorder::ByteOrder>(map: &Vec<IoStoreObjectIndex>, writer: &mut W) -> Result<(), Box<dyn Error>> {
        for i in map {
            i.to_buffer::<W, E>(writer)?;
        }
        Ok(())
    }
}

// Generic package summary implementation that contains fields appropriate for creating virtual container header
// The fields that are relevant are export_offset, export_bundle_offset and graph_offset
pub struct PackageSummaryExports {
    name_offset: u32,
    name_count: u32,
    import_offset: u32,
    export_offset: u32,
    export_bundle_offset: u32,
    graph_offset: u32
}

impl PackageSummaryExports {
    fn get_export_count(&self) -> u64 {
        (self.export_bundle_offset - self.export_offset) as u64 / IO_PACKAGE_FEXPORTMAP_SERIALIZED_SIZE
    }
    fn get_export_bundle_count(&self) -> u64 {
        0
    }
}

pub trait PackageIoSummaryDeserialize {
    // Create a PackageSummary instance from a given serialized FPackageSummary type. Since IO store packages don't include a file magic, 
    // this assumes that the reader stream is positioned correctly at the beginning of the package's header. An incorrect stream position can
    // lead to weird errors
    fn to_package_summary<R: Read + Seek, E: byteorder::ByteOrder>(reader: &mut R) -> Result<PackageSummaryExports, Box<dyn Error>>;
}

// Io Store Asset Header
#[repr(C)]
pub struct PackageSummary1 { // Unreal Engine 4.25 (untested)
    package_flags: u32,
    name_map_offset: i32,
    import_map_offset: i32,
    export_map_offset: i32,
    export_bundle_offset: i32,
    graph_data_offset: i32,
    graph_data_size: i32,
    bulk_data_start_offset: i32,
    global_import_index: i32,
    padding: i32
}

/* 
impl PackageIoSummaryDeserialize for PackageSummary1 {
    fn to_package_summary<R: Read + Seek, E: byteorder::ByteOrder>(reader: &mut R) -> Result<PackageSummaryExports, Box<dyn Error>> {
        reader.seek(SeekFrom::Current(0x4));
        let name_offset = reader.read_u32::<E>()?; // FPackageSummary->name_map_offset
        let name_count = 0;
        let import_offset = reader.read_u32::<E>()?; // FPackageSummary->import_map_offset
        let export_offset = reader.read_u32::<E>()?; // FPackageSummary->export_map_offset
        let export_bundle_offset = reader.read_u32::<E>()?; // FPackageSummary->export_bundle_export
        let graph_offset = reader.read_u32::<E>()?; // FPackageSummary->graph_offset
        Ok(PackageSummaryExports { name_offset, name_count, import_offset, export_offset, export_bundle_offset, graph_offset })
    }
}
*/

#[repr(C)]
pub struct PackageSummary2 { // Unreal Engine 4.25+, 4.26-4.27 (normal, plus, chaos)
    name: FMappedName,     
    source_name: FMappedName,
    package_flags: u32,
    cooked_header_size: u32,
    name_map_names_offset: i32,
    name_map_names_size: i32,
    name_map_hashes_offset: i32,
    name_map_hashes_size: i32,
    import_map_offset: i32,
    export_map_offset: i32,
    export_bundles_offset: i32,
    graph_data_offset: i32,
    graph_data_size: i32,
    pad: i32
}

impl PackageIoSummaryDeserialize for PackageSummary2 {
    fn to_package_summary<R: Read + Seek, E: byteorder::ByteOrder>(reader: &mut R) -> Result<PackageSummaryExports, Box<dyn Error>> {
        reader.seek(SeekFrom::Current(0x18));
        let name_offset = reader.read_u32::<E>()?; // FPackageSummary->name_map_offset
        reader.seek(SeekFrom::Current(0x8));
        let name_count = (reader.read_u32::<E>()? - 1) / std::mem::size_of::<u64>() as u32; // FPackageSummary->name_map_hashes_size - Algorithm hash
        let import_offset = reader.read_u32::<E>()?; // FPackageSummary->import_map_offset
        let export_offset = reader.read_u32::<E>()?; // FPackageSummary->export_map_offset
        let export_bundle_offset = reader.read_u32::<E>()?; // FPackageSummary->export_bundle_export
        let graph_offset = reader.read_u32::<E>()?; // FPackageSummary->graph_offset
        Ok(PackageSummaryExports { name_offset, name_count, import_offset, export_offset, export_bundle_offset, graph_offset })
    }
}

impl PackageSummary2 {
    pub fn from_buffer<R: Read + Seek, E: byteorder::ByteOrder>(reader: &mut R) -> Self {
        let name = reader.read_u64::<E>().unwrap().into();
        let source_name = reader.read_u64::<E>().unwrap().into();
        let package_flags = reader.read_u32::<E>().unwrap();
        let cooked_header_size = reader.read_u32::<E>().unwrap();
        let name_map_names_offset = reader.read_i32::<E>().unwrap();
        let name_map_names_size = reader.read_i32::<E>().unwrap();
        let name_map_hashes_offset = reader.read_i32::<E>().unwrap();
        let name_map_hashes_size = reader.read_i32::<E>().unwrap();
        let import_map_offset = reader.read_i32::<E>().unwrap();
        let export_map_offset = reader.read_i32::<E>().unwrap();
        let export_bundles_offset = reader.read_i32::<E>().unwrap();
        let graph_data_offset = reader.read_i32::<E>().unwrap();
        let graph_data_size = reader.read_i32::<E>().unwrap();
        Self {
            name,
            source_name,
            package_flags,
            cooked_header_size,
            name_map_names_offset,
            name_map_names_size,
            name_map_hashes_offset,
            name_map_hashes_size,
            import_map_offset,
            export_map_offset,
            export_bundles_offset,
            graph_data_offset,
            graph_data_size,
            pad: 0
        }
    }
}

#[repr(C)]
pub struct ZenPackageSummaryType1 { // Unreal Engine 5.0-5.2 (untested)
    bool_has_version_info: u32,
    header_size: u32,
    name: FMappedName,
    package_flags: u32,
    cooked_header_size: u32,
    imported_public_export_hashes_offset: i32,
    import_map_offset: i32,
    export_map_offset: i32,
    export_bundle_entries_offset: i32,
    graph_data_offset: i32
}

/* 
impl PackageIoSummaryDeserialize for ZenPackageSummaryType1 {
    fn to_package_summary<R: Read + Seek, E: byteorder::ByteOrder>(reader: &mut R) -> Result<PackageSummaryExports, Box<dyn Error>> {
        reader.seek(SeekFrom::Current(0x20));
        let export_offset = reader.read_u32::<E>()?; // FPackageSummary->export_map_offset
        let export_bundle_offset = reader.read_u32::<E>()?; // FPackageSummary->export_bundle_export
        let graph_offset = reader.read_u32::<E>()?; // FPackageSummary->graph_offset
        Ok(PackageSummaryExports { export_offset, export_bundle_offset, graph_offset })
    }
}
*/

#[repr(C)]
pub struct ZenPackageSummaryType2 { // Unreal Engine 5.3 (untested)
    bool_has_version_info: u32,
    header_size: u32,
    name: FMappedName,
    package_flags: u32,
    cooked_header_size: u32,
    imported_public_export_hases_offset: i32,
    import_map_offset: i32,
    export_map_offset: i32,
    export_bundle_entries_offset: i32,
    dependency_bundle_headers_offset: i32,
    dependency_bundle_entries_offset: i32,
    imported_package_names_offset: i32
}
// ZenPackageSummaryType2 looks like it has something different going on with how it's dependency graph works
// This can be worked on later

pub struct FGraphExternalArc {
    from_export_bundle_index: u32,
    to_export_bundle_index: u32
}

impl FGraphExternalArc {
    fn from_buffer<R: Read + Seek, E: byteorder::ByteOrder>(reader: &mut R) -> Self {
        let from_export_bundle_index = reader.read_u32::<E>().unwrap();
        let to_export_bundle_index = reader.read_u32::<E>().unwrap();
        Self { from_export_bundle_index, to_export_bundle_index }
    }
}

pub struct FGraphPackage {
    pub imported_package_id: u64, // hashed
    external_arcs: Vec<FGraphExternalArc>
}

impl FGraphPackage {
    pub fn from_buffer<R: Read + Seek, E: byteorder::ByteOrder>(reader: &mut R) -> Self {
        let imported_package_id = reader.read_u64::<E>().unwrap();
        let external_arc_count = reader.read_u32::<E>().unwrap();
        let mut external_arcs = Vec::with_capacity(external_arc_count as usize);
        for _ in 0..external_arc_count {
            external_arcs.push(FGraphExternalArc::from_buffer::<R, E>(reader));
        }
        Self {
            imported_package_id,
            external_arcs
        }
    }

    pub fn list_from_buffer<R: Read + Seek, E: byteorder::ByteOrder>(reader: &mut R) -> Vec<Self> {
        let imported_packages_count = reader.read_u32::<E>().unwrap();
        let mut values = vec![];
        for _ in 0..imported_packages_count {
            values.push(FGraphPackage::from_buffer::<R, E>(reader));
        }
        values
    }
}

#[repr(u32)]
pub enum ExportBundleCommandType {
    Create = 0,
    Serialize,
    Count, // added in UE 4.25+
}
impl TryFrom<u32> for ExportBundleCommandType {
    type Error = String;
    fn try_from(value: u32) -> Result<ExportBundleCommandType, Self::Error> {
        match value {
            0 => Ok(ExportBundleCommandType::Create),
            1 => Ok(ExportBundleCommandType::Serialize),
            2 => Ok(ExportBundleCommandType::Count),
            _ => Err(format!("An invalid type \"{}\" for ExportBundleCommandType was provided", value))
        }
    }
}
pub struct ExportBundleEntry { // same across all versions of Unreal Engine
    local_export_index: u32,
    command_type: ExportBundleCommandType
}
pub trait ExportBundle {
    // Get the number of export bundles that a package has. This info is used to build it's entry in 
    fn from_buffer<R: Read + Seek, E: byteorder::ByteOrder>(export_bundle_offset: u32, graph_offset: u32, reader: &mut R) -> u32;
}

#[repr(C)]
pub struct ExportBundleHeader4 { // Unreal Engine 4.25-4.27
    first_entry_index: u32,
    entry_count: u32,
}

impl ExportBundleHeader4 {
    fn new(first_entry_index: u32, entry_count: u32) -> Self {
        Self { first_entry_index, entry_count }
    }

    fn bundle_entry_sum(bundles: &Vec<ExportBundleHeader4>, to: usize) -> u32 {
        let mut total_bundles = 0;
        for i in 0..to {
            total_bundles += bundles[i].entry_count;
        }
        total_bundles
    }
}

impl ExportBundle for ExportBundleHeader4 {
    fn from_buffer<R: Read + Seek, E: byteorder::ByteOrder>(export_bundle_offset: u32, graph_offset: u32, reader: &mut R) -> u32 {
        // use clues from how the serialized data is structued to determine the export bundle count, since it's not stored as a field in header
        let mut predicted_export_bundles: Vec<ExportBundleHeader4> = vec![];
        loop {
            let first_index = reader.read_u32::<E>().unwrap();
            // export bundle indices must be contiguous (0, 32), (32, 20), (52, 20)
            if predicted_export_bundles.len() > 0 && first_index != predicted_export_bundles.last().unwrap().entry_count { 
                break;
            } 
            let export_bundle_entries = reader.read_u32::<E>().unwrap();
            if export_bundle_entries == 0 { // no need to continue
                break;
            }
            predicted_export_bundles.push(ExportBundleHeader4::new(first_index, export_bundle_entries));
        }
        let actual_entries = (graph_offset - export_bundle_offset - predicted_export_bundles.len() as u32 * 8) / 8; // 8 -> sizeof(FExportBumdleEntry)
        let mut actual_export_bundle_count = predicted_export_bundles.len();
        loop {
            if actual_export_bundle_count == 0 || actual_entries == ExportBundleHeader4::bundle_entry_sum(&predicted_export_bundles, actual_export_bundle_count) {
                break
            }
            actual_export_bundle_count -= 1;
        }
        let return_value = if actual_export_bundle_count > 0 { actual_export_bundle_count as u32 } else { 1 };
        return_value
    }
}

#[repr(C)]
pub struct ExportBundleHeader5 { // Unreal Engine 5.0+
    serial_offset: u64,
    first_entry_index: u32,
    entry_count: u32,
}
// impl ExportBundle for ExportBundleHeader5...

pub trait ContainerHeaderPosition {
    fn cursor_to_header<TResolver: TocResolverCommon>(resolver: &mut TResolver) -> u64;
    fn cursor_to_beginning_of_files<TResolver: TocResolverCommon>(resolver: &mut TResolver) -> u64;
    //fn get_fixed_container_size<TResolver: TocResolverCommon>(resolver: &mut TResolver) -> u64; // 4.25+ and 4.26 have a fixed size of 0x10000
}

pub struct ContainerHeaderPosition1; // 4.25+, 4.26
impl ContainerHeaderPosition for ContainerHeaderPosition1 {
    fn cursor_to_beginning_of_files<TResolver: TocResolverCommon>(resolver: &mut TResolver) -> u64 {
        0x10000
    }
    fn cursor_to_header<TResolver: TocResolverCommon>(resolver: &mut TResolver) -> u64 {
        0
    }
}

pub struct ContainerHeaderPosition2; // 4.27
impl ContainerHeaderPosition for ContainerHeaderPosition2 {
    fn cursor_to_beginning_of_files<TResolver: TocResolverCommon>(resolver: &mut TResolver) -> u64 {
        0
    }
    fn cursor_to_header<TResolver: TocResolverCommon>(resolver: &mut TResolver) -> u64 {
        0 // self.compression_blocks
    }
}

pub const CONTAINER_HEADER_PACKAGE_SERIALIZED_SIZE: u64 = 0x20;
pub const IO_PACKAGE_FEXPORTMAP_SERIALIZED_SIZE: u64 = 0x48;
pub struct ContainerHeaderPackage {
    // An export bundle's entry in a container header
    pub hash: u64,
    export_bundle_size: u64,
    export_count: u32,
    export_bundle_count: u32,
    load_order: u32,
    import_ids: Vec<u64>
}

impl ContainerHeaderPackage {
    // Parse the package file to extract the values needed to build a store entry in the container header
    pub fn from_package_summary<
        TExportBundle: ExportBundle,
        TSummary: PackageIoSummaryDeserialize,
        TReader: Read + Seek,
        TByteOrder: byteorder::ByteOrder
    >(file_reader: &mut TReader, hash: u64, size: u64, path: &str) -> Self { // consume the file object, we're only going to need it in here
        let package_summary = TSummary::to_package_summary::<TReader, TByteOrder>(file_reader).unwrap();
        let export_count = package_summary.get_export_count() as u32;
        file_reader.seek(SeekFrom::Start(package_summary.export_bundle_offset as u64)).unwrap(); // jump to FExportBundleHeader start
        let export_bundle_count = TExportBundle::from_buffer::<TReader, TByteOrder>(
            package_summary.export_bundle_offset, package_summary.graph_offset, file_reader
        ); // Go through each export bundle to look for the highest index
        file_reader.seek(SeekFrom::Start(package_summary.graph_offset as u64)).unwrap(); // go to FGraphPackage (imported_packages_count)
        let graph_packages = FGraphPackage::list_from_buffer::<TReader, TByteOrder>(file_reader);
        // previously, utoc emulator only relied on obtaining it's container header import ids from the package's graph package ids (which in itself was a bit of a hack)
        // however, this causes issues in regards to localized data, since graph package also includes ids for localization data not included in the container header
        // this causes a lot of weird behaviour. This hack involves reading the first file entries (Unreal always serializes asset file paths first, followed by script paths)
        // and then verifying that it's hash is within the graph package hashes. if both conditions are met, then it's allowed to be added as an import
        let mut import_ids = Vec::with_capacity(graph_packages.len());
        if export_bundle_count == 1 {
            for i in &graph_packages {
                import_ids.push(i.imported_package_id);
            }
        } else {
            file_reader.seek(SeekFrom::Start(package_summary.name_offset as u64)).unwrap();
            let mut names = vec![];
            for i in 0..package_summary.name_count {
                names.push(FString16::from_buffer_text::<TReader, TByteOrder>(file_reader).unwrap().unwrap());
            }
            let mut path_name_hashes = vec![];
            loop { // we only want to hash file paths
                if path_name_hashes.len() == names.len() || !names[path_name_hashes.len()].starts_with("/") {
                    break;
                }
                path_name_hashes.push(Hasher16::get_cityhash64(&names[path_name_hashes.len()]));
            }
            for i in &graph_packages {
                if path_name_hashes.contains(&i.imported_package_id) {
                    import_ids.push(i.imported_package_id);
                }
            }
        }
        //println!("ASSET {}, {} imports, {} export bundles", path, import_ids.len(), export_bundle_count);
        let load_order = 0; // This doesn't seem to matter?
        Self {
            hash,
            export_bundle_size: size,
            export_count,
            export_bundle_count,
            load_order,
            import_ids
        }
    }
    // Do a very incomplete serialization of an IO Store packaged asset to obtain it's export count, export bundle count and imported packages
    // Imports are Header.ExportMapOffset - Header.ImportMapOffset / 8
    // Export count is Header.ExportBundlesOffset - Header.ExportMapOffset) / sizeof(FExportMapEntry)
    // Export bundle count is export bundle count - export count
    // imported packages count determined (grab the hash from there and copy that)
    // Later, this code can do a more full serialization
    pub fn from_header_package<R: Read + Seek, E: byteorder::ByteOrder>(reader: &mut R, hash: u64, size: u64) -> Self { // beginning of IO store package
        reader.seek(SeekFrom::Start(0x2c));
        let export_offset = reader.read_u32::<E>().unwrap();
        let export_bundle_offset = reader.read_u32::<E>().unwrap();
        //println!("0x{:X}, 0x{:X}", export_offset, export_bundle_offset);
        let graph_offset = reader.read_u32::<E>().unwrap();
        let export_count = (export_bundle_offset - export_offset) / IO_PACKAGE_FEXPORTMAP_SERIALIZED_SIZE as u32;
        reader.seek(SeekFrom::Start(export_bundle_offset as u64 + 4)); // FExportBundleHeader->EntryCount
        let export_bundle_count_serialized = reader.read_u32::<E>().unwrap();
        let export_bundle_count = export_bundle_count_serialized - export_count;
        reader.seek(SeekFrom::Start(graph_offset as u64)); // FGraphPackage->ImportedPackagesCount
        let imported_package_count = reader.read_u32::<E>().unwrap();
        let mut import_ids: Vec<u64> = Vec::with_capacity(imported_package_count as usize);
        for _ in 0..imported_package_count {
            import_ids.push(FGraphPackage::from_buffer::<R, E>(reader).imported_package_id);
        }
        let load_order = 0; // For now, we'll see if this makes things crash
        Self {
            hash,
            export_bundle_size: size,
            export_count,
            export_bundle_count,
            load_order,
            import_ids
        }
    }

    pub fn to_buffer_store_entry<W: Write + Seek, E: byteorder::ByteOrder>(&self, writer: &mut W, base_offset: u64, curr_offset: &mut u64) -> Result<(), Box<dyn Error>> {
        writer.write_u64::<E>(self.export_bundle_size)?; // 0x0
        writer.write_u32::<E>(self.export_count)?; // 0x8
        writer.write_u32::<E>(self.export_bundle_count)?; // 0xc
        writer.write_u32::<E>(self.load_order)?; // 0x10
        writer.write_u32::<E>(0)?; // 0x14 padding
        let relative_offset = if self.import_ids.len() > 0 { Some((base_offset + *curr_offset - writer.stream_position().unwrap()) as u32) } else { None };
        writer.write_u32::<E>(self.import_ids.len() as u32)?; // 0x18 ImportedPackageCount
        writer.write_u32::<E>(match relative_offset {Some(n) => n, None => 0})?; // 0x1c RelativeOffsetToImports
        if let Some(rel) = relative_offset {
            let return_ptr = writer.stream_position().unwrap();
            writer.seek(SeekFrom::Current(rel as i64 - 8));
            for i in &self.import_ids {
                writer.write_u64::<E>(*i)?;
            }
            writer.seek(SeekFrom::Start(return_ptr));
            *curr_offset += 8 * self.import_ids.len() as u64;
        }
        Ok(())
    }
}

// Use this to check if a mod user is trying to load a cooked package
// Support for directly using cooked assets will hopefully be working soon...
pub const UASSET_MAGIC: u32 = 0x9E2A83C1;

#[derive(Debug)]
pub struct ObjectExport2 { // Unreal Engine 4.25+, 4.26-4.27 
    pub cooked_serial_offset: i64,
    pub cooked_serial_size: i64,
    pub object_name: FMappedName,
    pub outer_index: IoStoreObjectIndex, // TODO: use refs preferably
    pub class_name: IoStoreObjectIndex,
    pub super_name: IoStoreObjectIndex,
    pub template_name: IoStoreObjectIndex,
    pub global_import_name: IoStoreObjectIndex,
    pub object_flags: ObjectFlags,
    pub filter_flags: ExportFilterFlags
}

impl ObjectExport2 {
    pub fn from_pak_asset<
        N: NameMap,
        G: GameName
    >(map: &Vec<FObjectExport>, names: &N, imports: &Vec<IoStoreObjectIndex>, file_name: &str, game_name: &G) -> Vec<ObjectExport2> {
        // Convert FObjectImport into named ObjectImport
        let mut resolves = vec![];
        for (i, v) in map.into_iter().enumerate() {
            println!("{}, {:?}", i, v);
            resolves.push(v.resolve(names, imports, map, file_name, game_name));
        }
        resolves
    }

    pub fn map_to_buffer<W: Write, E: byteorder::ByteOrder>(map: &Vec<Self>, writer: &mut W) -> Result<(), Box<dyn Error>> {
        for i in map {
            i.to_buffer::<W, E>(writer)?;
        }
        Ok(())
    }

    pub fn to_buffer<W: Write, E: byteorder::ByteOrder>(&self, writer: &mut W) -> Result<(), Box<dyn Error>> {
        writer.write_i64::<E>(self.cooked_serial_offset);
        writer.write_i64::<E>(self.cooked_serial_size);
        writer.write_u64::<E>(self.object_name.into())?; // object_name
        self.outer_index.to_buffer::<W, E>(writer)?;
        self.class_name.to_buffer::<W, E>(writer)?;
        self.super_name.to_buffer::<W, E>(writer)?;
        self.template_name.to_buffer::<W, E>(writer)?;
        self.global_import_name.to_buffer::<W, E>(writer)?;
        writer.write_u32::<E>(self.object_flags)?;
        writer.write_u32::<E>(0)?; // filter flags
        Ok(())
    }
}

// Check that the first bytes of the file don't contain the magic used for cooked assets
pub fn is_valid_asset_type<R: Read + Seek, E: byteorder::ByteOrder>(reader: &mut R) -> bool {
    reader.seek(SeekFrom::Start(0));
    let magic_check = reader.read_u32::<E>().unwrap();
    magic_check != UASSET_MAGIC
}

#[cfg(test)]
mod tests {
    use std::{
        env,
        fs::File,
        io::BufReader,
        path::PathBuf
    };
    use byteorder::NativeEndian;
    use crate::{
        io_package::{ ContainerHeaderPackage, ExportBundleHeader4, PackageSummary2 },
        platform::Metadata
    };

    fn get_export_counts_for_asset(path: &str) {
        let os_file = File::open(path).unwrap();
        let file_size = Metadata::get_file_size(&os_file);
        let mut os_reader = BufReader::new(os_file);
        ContainerHeaderPackage::from_package_summary::<
            ExportBundleHeader4, PackageSummary2, BufReader<File>, NativeEndian
        >(&mut os_reader, 0, file_size, &format!("aa"));
    }

    #[test]
    fn get_bundle_count() {
        let base_path = env::var("RELOADEDIIMODS").unwrap_or_else(|err| panic!("Environment variable \"RELOADEDIIMODS\" is missing"));
        //let target_asset: PathBuf = [&base_path, "p3rpc.femc", "UnrealEssentials", "P3R", "Content", "Xrd777", "Blueprints", "UI", "SaveLoad", "BP_SaveLoadDraw.uasset"].iter().collect();
        //get_export_counts_for_asset(target_asset.to_str().unwrap());
        //let target_asset_2: PathBuf = [&base_path, "p3rpc.femc", "UnrealEssentials", "P3R", "Content", "L10N", "en", "Xrd777", "Field", "Data", "DataTable", "Texts", "DT_FldPlaceName.uasset"].iter().collect();
        //get_export_counts_for_asset(target_asset_2.to_str().unwrap());
        let target_asset_3: PathBuf = [&base_path, "p3rpc.femc", "UnrealEssentials", "P3R", "Content", "Xrd777", "Characters", "Player", "PC0002", "Models", "SK_PC0002_C991.uasset"].iter().collect();
        get_export_counts_for_asset(target_asset_3.to_str().unwrap());
    }
}