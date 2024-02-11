use std::{
    cell::RefCell,
    error::Error,
    path::{Path, PathBuf},
    fs, fs::{DirEntry, File},
    io, io::{BufReader, Cursor, Read, Seek, SeekFrom, Write},
    mem,
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard, RwLock, Weak},
    time::Instant,
};
use crate::{
    asset_collector::{
        MOUNT_POINT, SUITABLE_FILE_EXTENSIONS, ROOT_DIRECTORY, 
        TocDirectory, TocDirectorySyncRef, TocFile, TocFileSyncRef},
    io_package::{
        ContainerHeaderPackage,
        ExportBundle, ExportBundleHeader4,
        PackageIoSummaryDeserialize, 
        PackageSummary2},
    io_toc::{
        IO_FILE_INDEX_ENTRY_SERIALIZED_SIZE,
        ContainerHeader, 
        IoChunkId, IoChunkType4, IoDirectoryIndexEntry, IoFileIndexEntry, 
        IoStringPool, IoStoreTocEntryMeta, 
        IoStoreTocHeaderCommon, IoStoreTocHeaderType2, IoStoreTocHeaderType3,
        IoStoreTocCompressedBlockEntry, IoOffsetAndLength
    },
    platform::Metadata,
    string::{FString32NoHash, FStringSerializer, FStringSerializerExpectedLength, Hasher, Hasher16}
};

// Thanks to Swine's work, mod priority is now handled by UnrealEssentials, so there's no need for a _P patch name
// WIP:
//  - Implement proper unit testing for TOC building
//  - More accurate TOC structure. Ideally match Unreal Engine's TOC generation 1:1, including
//      - Sorting file entries within folders by file size
//      - Using the default compression alignment for each version
//      - Make the root mount folder have no name
//      - (Still won't generate metas by default though, that takes too long)
//  - Support for Type 1 (4.25) and Zen (5.0+) (currently only Type2 is supported (4.25+, 4.26, 4.27))
//  - Include benchmarking and code coverage tools as per the Reloaded's Rust template - 
//      https://github.com/Reloaded-Project/reloaded-templates-rust
pub const TOC_NAME:     &'static str = "UnrealEssentials";
pub const TARGET_TOC:   &'static str = "UnrealEssentials.utoc";
pub const TARGET_CAS:   &'static str = "UnrealEssentials.ucas";

//pub static CONTAINER_ENTRIES_OSPATH_POOL: Mutex<Option<Vec<String>>> = Mutex::new(None);
type ContainerEntriesType = Option<Vec<Vec<u16>>>;
pub static CONTAINER_ENTRIES_OSPATH_POOL: Mutex<ContainerEntriesType> = Mutex::new(None);
pub static CONTAINER_DATA: Mutex<Option<ContainerData>> = Mutex::new(None);

pub fn build_table_of_contents(toc_path: &str, version: u32) -> Option<Vec<u8>> {
    let path_check = PathBuf::from(toc_path); // build TOC here
    let file_name = path_check.file_name().unwrap().to_str().unwrap(); // unwrap, this is a file
    if file_name == TARGET_TOC { // check that we're targeting the correct UTOC
        let root_dir_lock = ROOT_DIRECTORY.lock().unwrap();
        match (*root_dir_lock).as_ref() {
            Some(root) => Some(build_table_of_contents_inner(Arc::clone(root), toc_path)),
            None => {
                println!("WARNING: No mod files were loaded for {}", file_name);
                None
            }
        }
    } else {
        None // Not our target TOC
    }
}

// Creates a TOC + CAS given a list of loose directories and files
// This currently only officially supports 4.25+, 4.26 and 4.27, but TocResolver is implemented in a way that will hopefully make adding support for new versions of
// the engine easier.
// Some notes about the structure of TOC and CAS for future use:
//  TocResolver1 (4.25 only)
// 4.25 features a very different UTOC structure compared to the other versions, and it's pretty clear from that implementation that IO Store was still a work in progress.
// The TOC structure only contains a smaller header followed by a list of toc entries, containing a chunk id (with a different format!) and a offset + length
// There isn't even a container header in the UCAS, so it's fair to say that this is different enough to warrant it's own TocResolver.
//  TocResolver2 (4.25+, 4.26-4.27)
// TocResolver2 handles a TOC which contains a list of chunk ids, followed by a list of offsets and lengths, then a list of compression blocks, then the directory index
// (mount point, files, folders and strings) and ends with a "meta" block containing SHA1 hashes of each file
// The CAS contains each file combined into a single stream, with a container header located at the end
// The only notable difference between 4.25+/4.26 and 4.27 is the inclusion of a partition size and partition count field that allows for the CAS to
// be broken into multiple files. This is where a custom IoStoreTocHeader type can be passed
// UE5 will be dealt with at a later date
// NOTE for Scarlet Nexus (4.25+) - container header is at top
// 4.25+ and 4.26 make their container file the *first* file in the list, while it's the last in 4.27
// pub struct TocResolverType1; // 4.25 only (only has header + toc entries (chunk id, offset and length))
pub trait TocResolverCommon { // Currently for 4.25+, 4.26 and 4.27
    //type TocHeaderType: IoStoreTocHeaderCommon; // make TocHeader (IoStoreTocHeaderType2 or IoStoreTocHeaderType3)
    //type ContainerHeaderType: PackageIoSummaryDeserialize; // Container Header in UCAS
    fn new<THeaderType: IoStoreTocHeaderCommon>(toc_name: &str, block_align: u32) -> impl TocResolverCommon;

    fn flatten_toc_tree(&mut self, tracker: &mut TocFlattenTracker, root: TocDirectorySyncRef);

    fn serialize<
        TSummary: PackageIoSummaryDeserialize,
        TIoTocHeader: IoStoreTocHeaderCommon
    >(&mut self, profiler: &mut TocBuilderProfiler, toc_path: &str) -> (Vec<u8>, ContainerData);

    // Common across all versions
    fn create_chunk_id(&self, file_path: &str, chunk_type: IoChunkType4) -> IoChunkId {
        // remove Content from path
        let path_to_replace_split = file_path.split_once("/Content").unwrap();
        let path_to_replace = "/".to_owned() + path_to_replace_split.0 + path_to_replace_split.1;
        IoChunkId::new(&path_to_replace, chunk_type)
    }

    fn get_file_hash(&self, curr_file: &IoFileIndexEntry) -> IoChunkId {
        // unwrap a bunch. any errors related to this would've been handled in the asset collection stage
        let chunk_type = match SUITABLE_FILE_EXTENSIONS.iter().find(
                |exist| **exist == PathBuf::from(&curr_file.os_path).extension().unwrap().to_str().unwrap()
            ) {
            Some(io_ext) => {
                match *io_ext {
                    "uasset" | "umap" => IoChunkType4::ExportBundleData, //.uasset, .umap
                    "ubulk" => IoChunkType4::BulkData, // .ubulk
                    "uptnl" => IoChunkType4::OptionalBulkData, // .uptnl
                    _ => panic!("CRITICAL ERROR: Did not get a supported file extension. This should've been handled earlier")
                }
            }
            // this file should've been skipped, see add_from_folders_inner in asset_collector.rs
            None => panic!("CRITICAL ERROR: Did not get a supported file extension. This should've been handled earlier")
        };
        self.create_chunk_id(&curr_file.hash_path, chunk_type)
    }

}

pub const DEFAULT_COMPRESSION_BLOCK_ALIGNMENT: u32 = 0x800;

pub struct TocFlattenTracker {
    // Used to set the correct directory/file/string indices when flattening TocDirectory tree into Directory Index entries
    pub resolved_directories: u32,
    pub resolved_files: u32,
    pub resolved_strings: u32,
}

impl TocFlattenTracker {
    pub fn new() -> Self {
        Self {
            resolved_directories: 0,
            resolved_files: 0,
            resolved_strings: 0
        }
    }
}

pub struct TocResolverType2 { // Currently for 4.25+, 4.26 and 4.27
    pub directories: Vec<IoDirectoryIndexEntry>,
    pub files: Vec<IoFileIndexEntry>,
    pub strings: Vec<String>,
    compression_block_size: u32,
    compression_block_alignment: u32,
    toc_name_hash: u64,
    pub chunk_ids: Vec<IoChunkId>,
    pub offsets_and_lengths: Vec<IoOffsetAndLength>,
    pub compression_blocks: Vec<IoStoreTocCompressedBlockEntry>,
    pub metas: Vec<IoStoreTocEntryMeta>,
    pub cas_pointer: u64, // Current virtual position of container file
}

impl TocResolverCommon for TocResolverType2 {
    //type TocHeaderType = IoStoreTocHeaderType2;
    //type ContainerHeaderType = PackageSummary2;
    fn new<
        THeaderType: IoStoreTocHeaderCommon
    >(toc_name: &str, block_align: u32) -> impl TocResolverCommon {
        Self { 
            // Directory block
            directories: vec![], // The resulting directory list will be serialized as an FIoDirectoryIndexEntry
            files: vec![], // Our file list will be serialized as an FIoFileIndexEntry
            strings: vec![], // Strings will be owned by a string pool where there'll be serialized into an FString32NoHash array
            compression_block_size: 0x10000, // default for UE 4.26/4.27 - used for offset + length offset
            compression_block_alignment: if block_align < 0x10 { 0x10 } else { block_align }, // 0x800 is default for UE 4.27 (isn't saved in toc), 0x0 is used for UE 4.26
            // every file is virtually put on an alignment of [compression_block_size] (in reality, they're only aligned to nearest 16 bytes)
            // offset section defines where each file's data starts, while compress blocks section defines each compression block
            toc_name_hash: Hasher16::get_cityhash64("Game"), // used for container id (is also the last file in partition) (verified)
            chunk_ids: vec![],
            offsets_and_lengths: vec![],
            compression_blocks: vec![],
            metas: vec![],
            cas_pointer: 0
        }
    }
    // Flatten the tree of directories + files into a list of directories and list of files
    fn flatten_toc_tree(&mut self, tracker: &mut TocFlattenTracker, root: TocDirectorySyncRef) {
        self.directories = self.flatten_toc_tree_dir(tracker, Arc::clone(&root));
    }
    fn serialize<
        TSummary: PackageIoSummaryDeserialize,
        TIoTocHeader: IoStoreTocHeaderCommon
    >(
        &mut self, 
        profiler: &mut TocBuilderProfiler, 
        toc_path: &str
    ) -> (Vec<u8>, ContainerData) {
        type CV = Cursor<Vec<u8>>;
        type EN = byteorder::NativeEndian;
        let mut toc_storage: CV = Cursor::new(vec![]); // TOC Storage gets stored as a MemoryStream
        // CAS storage will be a MultiStream of FileStreams with a MemoryStream of gaps between it
        // Set capacity so that vec doesn't realloc
        let mut container_string_pool = CONTAINER_ENTRIES_OSPATH_POOL.lock().unwrap();
        *container_string_pool = Some(Vec::with_capacity(self.files.len()));
        let mut container_header = ContainerHeader::new(self.toc_name_hash);
        let mut container_data = ContainerData { header: vec![], virtual_blocks: vec![] };
        let file_count = self.files.len();
        for i in 0..self.files.len() {
            container_data.virtual_blocks.push(self.serialize_entry::<TSummary>(
                i, &mut container_header, &mut container_string_pool
            ));
        }
        container_data.header = self.serialize_container_header::<EN>(&mut container_header);
        // Write our TOC
        let toc_header = TIoTocHeader::new(
            self.toc_name_hash, 
            self.files.len() as u32 + 1, // + 1 for container header
            self.compression_blocks.len() as u32,
            self.compression_block_size,
            self.get_directory_index_size()
        );
        // FIoStoreTocHeader
        toc_header.to_buffer::                          <CV, EN>(&mut toc_storage).unwrap(); // FIoStoreTocHeader
        IoChunkId::list_to_buffer::                     <CV, EN>(&self.chunk_ids, &mut toc_storage).unwrap(); // FIoChunkId
        IoOffsetAndLength::list_to_buffer::             <CV, EN>(&self.offsets_and_lengths, &mut toc_storage).unwrap(); // FIoOffsetAndLength
        IoStoreTocCompressedBlockEntry::list_to_buffer::<CV, EN>(&self.compression_blocks, &mut toc_storage).unwrap(); // FIoStoreTocCompressedBlockEntry
        FString32NoHash::to_buffer::                    <CV, EN>(MOUNT_POINT, &mut toc_storage).unwrap(); // Mount Point
        IoDirectoryIndexEntry::list_to_buffer::         <CV, EN>(&self.directories, &mut toc_storage).unwrap(); // FIoDirectoryIndexEntry
        IoFileIndexEntry::list_to_buffer::              <CV, EN>(&self.files, &mut toc_storage).unwrap(); // FIoFileIndexEntry
        IoStringPool::list_to_buffer::                  <CV, EN>(&self.strings, &mut toc_storage).unwrap(); // FIoStringIndexEntry
        IoStoreTocEntryMeta::list_to_buffer::           <CV, EN>(&self.metas, &mut toc_storage).unwrap(); // FIoStoreTocEntryMeta

        (toc_storage.into_inner(), container_data)
    }
}

impl TocResolverType2 {
    fn get_flat_string_index(&mut self, tracker: &mut TocFlattenTracker, name: &str) -> u32 {
        // check that our string is unique, else get the index for that....
        (match self.strings.iter().position(|exist| exist == name) {
            Some(i) => i,
            None => {
                self.strings.push(name.to_string());
                tracker.resolved_strings += 1;
                self.strings.len() - 1
            },
        }) as u32
    }
    fn flatten_toc_tree_dir(&mut self, tracker: &mut TocFlattenTracker, node: TocDirectorySyncRef) -> Vec<IoDirectoryIndexEntry> {
        let mut values = vec![];
        let mut flat_value = IoDirectoryIndexEntry {
            name: match node.read().unwrap().name.as_ref() {
                Some(t) => self.get_flat_string_index(tracker, t),
                None => u32::MAX
            },
            first_child: u32::MAX,
            next_sibling: u32::MAX,
            first_file: u32::MAX
        };
        // Iterate through each file
        if TocDirectory::has_files(Arc::clone(&node)) {
            let mut curr_file = Arc::clone(node.read().unwrap().first_file.as_ref().unwrap());
            flat_value.first_file = tracker.resolved_files;
            loop {
                let mut flat_file = IoFileIndexEntry {
                    name: self.get_flat_string_index(tracker, &curr_file.read().unwrap().name),
                    next_file: u32::MAX,
                    user_data: tracker.resolved_files,
                    file_size: curr_file.read().unwrap().file_size,
                    os_path: curr_file.read().unwrap().os_file_path.clone(),
                    hash_path: String::new()

                };
                // travel upwards through parents to build hash path
                // calculate hash after validation so it's easier to remove incorrectly formatted uassets
                let mut path_comps: Vec<String> = vec![];
                let mut curr_parent = Arc::clone(&node);
                loop {
                    if let Some(t) = curr_parent.read().unwrap().name.as_ref() {
                        path_comps.insert(0, t.to_owned());
                    }
                    match Arc::clone(&curr_parent).read().unwrap().parent.upgrade() {
                        Some(ip) => curr_parent = Arc::clone(&ip),
                        None => break
                    }
                }
                let filename_buf = PathBuf::from(&curr_file.read().unwrap().name);
                let path = path_comps.join("/") + "/" + filename_buf.file_stem().unwrap().to_str().unwrap();
                //println!("{} PATH: {}, OS: {}", &curr_file.borrow().name, &path, &curr_file.borrow().os_file_path);
                flat_file.hash_path = path;
                // go to next file
                tracker.resolved_files += 1;
                match Arc::clone(&curr_file).read().unwrap().next.as_ref() {
                    Some(next) => {
                        flat_file.next_file = tracker.resolved_files;
                        self.files.push(flat_file);
                        curr_file = Arc::clone(next)
                    },
                    None => {
                        self.files.push(flat_file);
                        break
                    }
                }
            }
        }
        // Iterate through inner directories
        tracker.resolved_directories += 1;
        //println!("flatten(): {}, id {}", &node.borrow().name, self.resolved_directories - 1);
        if TocDirectory::has_children(Arc::clone(&node)) {
            flat_value.first_child = tracker.resolved_directories;
            values.push(flat_value);
            let mut curr_child = Arc::clone(node.read().unwrap().first_child.as_ref().unwrap());
            loop {
                let mut children = self.flatten_toc_tree_dir(tracker, Arc::clone(&curr_child));
                match Arc::clone(&curr_child).read().unwrap().next_sibling.as_ref() { // get the next child (if they exist)
                    Some(next) => {
                        children[0].next_sibling = tracker.resolved_directories;
                        values.extend(children);
                        curr_child = Arc::clone(next);
                    },
                    None => {
                        values.extend(children);
                        break
                    }
                }
            }
        } else {
            values.push(flat_value);
        }
        values
    }
    fn create_compression_blocks(file_size: u64, pointer: u64, block_size: u32) -> Vec<IoStoreTocCompressedBlockEntry> {
        let compression_block_count = (file_size / block_size as u64) + 1; // need at least 1 compression block
        let mut size_remaining = file_size as u32;
        let mut gen_blocks = Vec::with_capacity(compression_block_count as usize);
        for i in 0..compression_block_count {
            let cmp_size = if size_remaining > block_size {block_size} else {size_remaining}; // cmp_size = decmp_size
            let offset = pointer + block_size as u64 * i;
            let new_cmp_block = IoStoreTocCompressedBlockEntry::new(offset, cmp_size);
            gen_blocks.push(new_cmp_block);
            if size_remaining > block_size {size_remaining -= block_size};
        }
        gen_blocks
    }

    fn serialize_container_header<TEndian: byteorder::ByteOrder>(&mut self, container_header: &mut ContainerHeader) -> Vec<u8> {
        let mut container_header_buffer = Cursor::new(vec![]);
        let container_header = container_header.to_buffer::<Cursor<Vec<u8>>, TEndian>(&mut container_header_buffer).unwrap(); // write our container header in the buffer
        self.chunk_ids.push(IoChunkId::new_from_hash(self.toc_name_hash, IoChunkType4::ContainerHeader)); // header chunk id
        let header_offset = self.compression_blocks.len() as u64 * self.compression_block_size as u64; 
        self.offsets_and_lengths.push(IoOffsetAndLength::new(header_offset, container_header.len() as u64)); // header offset + length
        self.compression_blocks.append(&mut TocResolverType2::create_compression_blocks(container_header.len() as u64, self.cas_pointer, self.compression_block_size));
        self.metas.push(IoStoreTocEntryMeta::new_empty());
        container_header
    }

    fn serialize_entry<TSummary: PackageIoSummaryDeserialize>(&mut self, index: usize, container_header: &mut ContainerHeader, pool_guard: &mut MutexGuard<ContainerEntriesType>) -> PartitionBlock {
        let target_file = &self.files[index];
        let generated_chunk_id = self.get_file_hash(target_file); // create the hash for the new file
        self.chunk_ids.push(generated_chunk_id); // push once we're sure that the file's valid
        let curr_file = &self.files[index]; // Generate FIoOffsetAndLength
        let file_offset = self.compression_blocks.len() as u64 * self.compression_block_size as u64;
        let generated_offset_length = IoOffsetAndLength::new(file_offset, curr_file.file_size);
        self.offsets_and_lengths.push(generated_offset_length);
        // Generate compression blocks
        self.compression_blocks.append(&mut TocResolverType2::create_compression_blocks(target_file.file_size, self.cas_pointer, self.compression_block_size));
        self.metas.push(IoStoreTocEntryMeta::new_empty()); // Generate meta - SHA1 hash of the file's contents (doesn't seem to be required)
        if self.chunk_ids[index].get_type() == IoChunkType4::ExportBundleData {
            let os_file = File::open(&target_file.os_path).unwrap(); // Export Bundles (.uasset) have store entry data written
            let mut file_reader = BufReader::with_capacity(Self::FILE_SUMMARY_READER_ALLOC, os_file);
            container_header.packages.push(ContainerHeaderPackage::from_package_summary::<
                ExportBundleHeader4, TSummary, BufReader<File>, byteorder::NativeEndian
            >(
                &mut file_reader, 
                self.chunk_ids[index].get_raw_hash(), curr_file.file_size
            ));
        }
        // write into container data
        // Bug fix for 1.0.3 - support for unicode characters (UTF-16) in file path
        (**pool_guard).as_mut().unwrap().push(target_file.os_path.encode_utf16().collect::<Vec<u16>>());
        let curr_ospath = &mut (**pool_guard).as_mut().unwrap()[index];
        curr_ospath.push(0);
        //println!("adding {} to partition block", curr_ospath);
        let new_partition_block = PartitionBlock {
            os_path: curr_ospath.as_ptr() as usize,
            start: self.cas_pointer,
            length: target_file.file_size
        };
        self.cas_pointer += target_file.file_size; // move cas pointer
        let alignment_amount = self.cas_pointer % self.compression_block_alignment as u64;
        if alignment_amount > 0 { // align to compression block alignment
            let diff = self.compression_block_alignment as u64 - alignment_amount;
            self.cas_pointer += diff;
        }
        new_partition_block
    }

    pub const FILE_SUMMARY_READER_ALLOC: usize = 0x2000;

    fn get_directory_index_size(&self) -> u32 {
        // Get DirectoryIndexSize = Directory Entries + File Entries + Strings
        // Each section contains a u32 to note the object count
        let directory_index_bytes = (self.directories.len() * std::mem::size_of::<IoDirectoryIndexEntry>() + mem::size_of::<u32>()) as u32;
        let file_index_bytes = (self.files.len() * IO_FILE_INDEX_ENTRY_SERIALIZED_SIZE + mem::size_of::<u32>()) as u32;
        let mut string_index_bytes = mem::size_of::<u32>() as u32;
        self.strings.iter().for_each(|name| string_index_bytes += FString32NoHash::get_expected_length(name) as u32);
        directory_index_bytes + file_index_bytes + string_index_bytes
    }
}

// TODO: Set the mount point further up in mods where the file structure doesn't diverge at root
// TODO: Pass version param (probably as trait) to customize how TOC is produced depenending on the target version
// TODO: Support UE5 (sometime soon)

pub fn build_table_of_contents_inner(root: TocDirectorySyncRef, toc_path: &str) -> Vec<u8> {
    //println!("BUILD TABLE OF CONTENTS FOR {}", TARGET_TOC);
    let mut profiler = TocBuilderProfiler::new();
    let mut resolver = TocResolverType2::new::<
        IoStoreTocHeaderType2
    >(TARGET_TOC, DEFAULT_COMPRESSION_BLOCK_ALIGNMENT);
    resolver.flatten_toc_tree(&mut TocFlattenTracker::new(), Arc::clone(&root));
    let serialize_results = resolver.serialize::<PackageSummary2, IoStoreTocHeaderType3>(&mut profiler, toc_path);
    let mut container_lock = CONTAINER_DATA.lock().unwrap();
    *container_lock = Some(serialize_results.1);
    serialize_results.0
}

pub struct ContainerData {
    pub header: Vec<u8>,
    pub virtual_blocks: Vec<PartitionBlock>
}

#[repr(C)]
pub struct PartitionBlock {
    //os_path: *const u8, // 0x0
    os_path: usize, // 0x0 (pointers don't implement Send or Sync)
    start: u64, // 0x8
    length: u64, // 0x10
}

pub struct TocBuilderProfiler {
    // All file sizes are in bytes
    successful_files: u64,
    successful_files_size: u64,
    incorrect_asset_format: Vec<String>, // list of offending files, print out to console
    incorrect_asset_format_size: u64,
    failed_to_read: Vec<String>,
    failed_to_read_size: u64,
    container_header_hash: u64,
    compression_block_count: u64,
    mount_point: String,
    directory_index_size: u64,
    file_index_size: u64,
    string_index_size: u64,
    generated_meta_hashes: bool,
    start_time: Instant,
    time_to_flatten: u128,
    time_to_serialize: u128
}

impl TocBuilderProfiler {
    fn new() -> Self {
        Self {
            successful_files: 0,
            successful_files_size: 0,
            incorrect_asset_format: vec![],
            incorrect_asset_format_size: 0,
            failed_to_read: vec![],
            failed_to_read_size: 0,
            container_header_hash: 0,
            compression_block_count: 0,
            mount_point: String::new(),
            directory_index_size: 0,
            file_index_size: 0,
            string_index_size: 0,
            generated_meta_hashes: false,
            start_time: Instant::now(),
            time_to_flatten: 0,
            time_to_serialize: 0
        }
    }

    fn set_flatten_time(&mut self) {
        self.time_to_flatten = self.start_time.elapsed().as_micros();
    }
    fn set_serialize_time(&mut self) {
        self.time_to_serialize = self.start_time.elapsed().as_micros();
    }
    fn display_results(&self) {
        // TODO: Advanced display results
        println!("Flatten Time: {} ms", self.time_to_flatten as f64 / 1000f64);
        println!("Serialize Time: {} ms", self.time_to_serialize as f64 / 1000f64);
    }
}