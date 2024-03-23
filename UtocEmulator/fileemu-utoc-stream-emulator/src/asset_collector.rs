use crate::{
    io_package,
    io_toc::IoChunkId,
    metadata::{UtocMetadata, UTOC_METADATA},
    platform::Metadata,
    toc_factory::TARGET_TOC
};
use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap},
    fs, fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock, Weak},
    time::Instant
};

pub type TocDirectorySyncRef = Arc<RwLock<TocDirectory>>;
pub type TocFileSyncRef = Arc<RwLock<TocFile>>;

pub static ROOT_DIRECTORY: Mutex<Option<TocDirectorySyncRef>> = Mutex::new(None);
pub static ASSET_COLLECTOR_PROFILER: Mutex<Option<AssetCollectorProfiler>> = Mutex::new(None);

// Create tree of assets that can be used to build a TOC
pub fn add_from_folders(mod_path: &str) {
    // mod loading happens synchronously, safe to unwrap
    let mut profiler_lock = ASSET_COLLECTOR_PROFILER.lock().unwrap();
    if *profiler_lock == None { // Check profiler is active
        *profiler_lock = Some(AssetCollectorProfiler::new());
    }
    let mod_path: PathBuf = PathBuf::from(mod_path);
    if Path::exists(Path::new(&mod_path)) {
        let mut profiler_mod = AssetCollectorProfilerMod::new(mod_path.to_str().unwrap());
        let mut root_dir_lock = ROOT_DIRECTORY.lock().unwrap();
        if let None = *root_dir_lock {
            *root_dir_lock = Some(TocDirectory::new_rc(None));
        }
        add_from_folders_inner(Arc::clone(&(*root_dir_lock).as_ref().unwrap()), &mod_path, &mut profiler_mod.data, true);
        (*profiler_lock).as_mut().unwrap().mods_loaded.push(profiler_mod);
    }
}

//      A <--------
//      ^    ^    ^
//      |    |    | (refs from child -> parent)
//      v    |    | (owns from parent -> child and in sibling and file linked lists)
//      B -> C -> D

pub struct TocDirectory {
    pub name:           Option<String>, // leaf name only (directory name or file name)
    pub parent:         Weak        <RwLock<TocDirectory>>, // weakref to parent for path building for FIoChunkIds
    pub first_child:    Option      <TocDirectorySyncRef>, // first child
    last_child:         Weak        <RwLock<TocDirectory>>, // O(1) insertion on directory add
    pub next_sibling:   Option      <TocDirectorySyncRef>, // next sibling
    pub first_file:     Option      <TocFileSyncRef>, // begin file linked list, owns file children
    last_file:          Weak        <RwLock<TocFile>>, // O(1) insertion on file add
}

impl TocDirectory {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            parent: Weak::new(),
            first_child: None,
            last_child: Weak::new(),
            next_sibling: None, // root folder has no siblings
            first_file: None,
            last_file: Weak::new(),
        }
    }
    #[inline] // convenience function to create reference counted toc directories
    pub fn new_rc(name: Option<String>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(TocDirectory::new(name)))
    }
    // Returns true/false depending on if the target directory contains any child directories
    pub fn has_children(dir: TocDirectorySyncRef) -> bool {
        match &dir.read().unwrap().first_child {
            Some(_) => true,
            None => false
        }
    }
    // Returns true/false depending on if the target directory contains any child files
    pub fn has_files(dir: TocDirectorySyncRef) -> bool {
        match &dir.read().unwrap().first_file {
            Some(_) => true,
            None => false
        }
    }
    // Add a file child into directory that doesn't currently contain any other files
    #[inline]
    fn add_first_file(dir: TocDirectorySyncRef, file: TocFileSyncRef) {
        dir.write().unwrap().first_file = Some(Arc::clone(&file));
        dir.write().unwrap().last_file = Arc::downgrade(&file);
    }
    // Replace an existing file in the file list. Kick it off the list so it drops on add_or_replace_file's scope
    #[inline]
    fn replace_file(
        dir: TocDirectorySyncRef, // containing directory
        prev_file: Option<TocFileSyncRef>, // previous file, which links to replacee (unless it's the *first* file)
        replacee: TocFileSyncRef, // the file to get replaced (file merging is a future problem)
        replacer: TocFileSyncRef // file that'll take the place of replacee in the chain
    ) {
        if let None = replacee.read().unwrap().next.as_ref() { // replacee is the last file in chain, dir->last_file = weakref(replacer)
            dir.write().unwrap().last_file = Arc::downgrade(&replacer);
        } else { // replacee is at the start or in the middle, set replacer->next = replacee->next
            replacer.write().unwrap().next = Some(Arc::clone(replacee.read().unwrap().next.as_ref().unwrap()));
        }
        if let None = prev_file { // replacee is at the start, set dir->first_file = replacer
            dir.write().unwrap().first_file = Some(Arc::clone(&replacer));
        } else { // prev->next = replacer, replacee drops here
            prev_file.as_ref().unwrap().write().unwrap().next = Some(Arc::clone(&replacer));
        }

    }
    // Add a file to the end of the directory's file list, which contains at least 1 existing file
    #[inline]
    fn add_another_file(dir: TocDirectorySyncRef, file: TocFileSyncRef) {
        dir.read().unwrap().last_file.upgrade().unwrap().write().unwrap().next = Some(Arc::clone(&file)); // own our new child on the end of children linked list
        dir.write().unwrap().last_file = Arc::downgrade(&file); // and set the tail to weakref of the new child
    }
    // go through file list to check if the target file already exists, then replace it with our own
    // otherwise, add our file to the end
    pub fn add_or_replace_file(dir: TocDirectorySyncRef, file: TocFileSyncRef) -> TocFileAddType {
        match TocDirectory::has_files(Arc::clone(&dir)) {
            true => { // :adachi_true: - search file linked list
                let mut found = false;
                let mut prev: Option<TocFileSyncRef> = None;
                let mut curr_file = Arc::clone(dir.read().unwrap().first_file.as_ref().unwrap());
                loop {
                    if curr_file.read().unwrap().name == file.read().unwrap().name { // we got the file, replace it
                        found = true;
                        break
                    }
                    match Arc::clone(&curr_file).read().unwrap().next.as_ref() { // check if next points to next entry in chain or ends the chain
                        Some(f) => {
                            prev = Some(Arc::clone(&curr_file));
                            curr_file = Arc::clone(&f);
                        },
                        None => { // couldn't find it to replace, add it to the end
                            break // we need to escape this scope to prevent creating mut ref of last_file->next while const ref last_file->next is still valid
                        }
                    }
                }
                if !found {
                    TocDirectory::add_another_file(Arc::clone(&dir), Arc::clone(&file));
                    TocFileAddType::Addition
                } else {
                    TocDirectory::replace_file(
                        Arc::clone(&dir),
                        prev, // prev is only set with second file in list onwards
                        Arc::clone(&curr_file),
                        Arc::clone(&file)
                    );
                    TocFileAddType::Replacement
                }
            },
            false => {
                TocDirectory::add_first_file(Arc::clone(&dir), Arc::clone(&file));
                TocFileAddType::Addition
            }
        }
    }
    // get a child directory from a parent directory if it exists
    pub fn get_child_dir(parent: TocDirectorySyncRef, exist: &str) -> Option<TocDirectorySyncRef> {
        match TocDirectory::has_children(Arc::clone(&parent)) {
            true => {
                let mut curr_dir = Arc::clone(&parent.read().unwrap().first_child.as_ref().unwrap());
                let mut result = None;
                loop {
                    if let Some(dir_name) = curr_dir.read().unwrap().name.as_ref() {
                        if dir_name == exist { // we got our directory
                            result = Some(Arc::clone(&curr_dir));
                            break;
                        }
                    }
                    match Arc::clone(&curr_dir).read().unwrap().next_sibling.as_ref() {
                        Some(ip) => curr_dir = Arc::clone(&ip),
                        None => break
                    }
                }
                result
            },
            false => None // has no children, can only not exist
        }
    }
    pub fn add_directory(parent: TocDirectorySyncRef, child: TocDirectorySyncRef) {
        child.write().unwrap().parent = Arc::downgrade(&parent); // set child node's parent as weak ref of parent 
        if !TocDirectory::has_children(Arc::clone(&parent)) { // if parent has no nodes (if let doesn't work here since scope of &first_child extends to entire statement, overlapping &mut first_child)
            parent.write().unwrap().first_child = Some(Arc::clone(&child)); // head and tail set to new child
            parent.write().unwrap().last_child = Arc::downgrade(&child);
            return;
        }
        parent.read().unwrap().last_child.upgrade().as_ref().unwrap().write().unwrap().next_sibling = Some(Arc::clone(&child));
        parent.write().unwrap().last_child = Arc::downgrade(&child);
    }
}

#[derive(Debug/* , PartialEq*/)]
pub struct TocFile {
    pub next: Option<Arc<RwLock<TocFile>>>,
    pub name: String,
    pub file_size: u64,
    pub os_file_path: String // needed so we can open it, copy it then write it into partition
}

impl TocFile {
    // constructor
    fn new(name: &str, file_size: u64, os_path: &str) -> Self {
        Self {
            next: None,
            name: String::from(name),
            file_size,
            os_file_path: String::from(os_path)
        }
    }
    #[inline] // convenience function to create reference counted toc files
    pub fn new_rc(name: &str, file_size: u64, os_path: &str) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(TocFile::new(name, file_size, os_path)))
    }
}

pub enum TocFileAddType {
    Addition,
    Replacement
}

pub const SUITABLE_FILE_EXTENSIONS: &'static [&'static str] = ["uasset", "ubulk", "uptnl", "umap"].as_slice();
pub const MOUNT_POINT: &'static str = "../../../";
pub const GAME_ROOT: &'static str = "Game";

pub fn add_from_folders_inner(parent: TocDirectorySyncRef, os_path: &PathBuf, profiler: &mut AssetCollectorProfilerModContents, first: bool) {
    // We've already checked that this path exists in AddFromFolders, so unwrap directly
    // This folder is equivalent to /[ProjectName]/Content, so our mount point will be
    // at least ../../../[ProjectName] (../../../Game/)
    // build an unsorted n-tree of directories and files, preorder traversal
    // higher priority mods should overwrite contents of files, but not directories
    for i in fs::read_dir(os_path).unwrap() {
        match &i {
            Ok(fs_obj) => { // we have our file system object, now determine if it's a directory or folder
                let fs_obj_os_name = fs_obj.file_name(); // this does assume that the object name is valid Unicode
                let mut name = String::from(fs_obj_os_name.to_str().unwrap()); // if it's not i'll be very surprised
                let file_type = fs_obj.file_type().unwrap();
                if file_type.is_dir() { // new directory. mods can only expand on this
                    let mut inner_path = PathBuf::from(os_path);
                    inner_path.push(&name);
                    match TocDirectory::get_child_dir(Arc::clone(&parent), &name) {
                        // check through folder regardless since there may be new inner folders in there
                        Some(child_dir) => add_from_folders_inner(Arc::clone(&child_dir), &inner_path, profiler, false),
                        None => {
                            // Set the root directory to Game if it isn't engine so people can use the game name (assuming only Engine and Game)
                            if first && name != "Engine"
                            {
                                println!("Setting root directory {} to Game", name);
                                name = GAME_ROOT.to_string();
                            }

                            // this is a new directory, create it and then check inside it
                            let new_dir = TocDirectory::new_rc(Some((&name).to_owned()));
                            TocDirectory::add_directory(Arc::clone(&parent), Arc::clone(&new_dir));
                            add_from_folders_inner(Arc::clone(&new_dir), &inner_path, profiler, false);
                            profiler.add_directory();
                        }
                    }
                } else if file_type.is_file() {
                    let file_size = Metadata::get_object_size(fs_obj);
                    match PathBuf::from(&name).extension() {
                        Some(ext) => {
                            let ext_str = ext.to_str().unwrap();
                            match SUITABLE_FILE_EXTENSIONS.iter().find(|exist| **exist == ext_str) {
                                // it's a matter of either replacing an existing file or adding a new file
                                // ,,,at least until we start thinking about merging P3RE persona tables (lol)
                                Some(io_ext) => {
                                    if *io_ext == "uasset" || *io_ext == "umap" { // export bundles - requires checking file header to ensure that it doesn't have the cooked asset signature
                                        let current_file = File::open(fs_obj.path().to_str().unwrap()).unwrap();
                                        let mut file_reader = BufReader::with_capacity(4, current_file);
                                        if !io_package::is_valid_asset_type::<BufReader<File>, byteorder::NativeEndian>(&mut file_reader) {
                                            profiler.add_skipped_file(os_path.to_str().unwrap(), format!("Uses cooked package"), file_size);
                                            continue
                                        }
                                    }
                                    let new_file = TocFile::new_rc(&name, file_size, fs_obj.path().to_str().unwrap());
                                    match TocDirectory::add_or_replace_file(Arc::clone(&parent), Arc::clone(&new_file)) {
                                        TocFileAddType::Addition => profiler.add_added_file(file_size),
                                        TocFileAddType::Replacement => profiler.add_replaced_file(file_size)
                                    }
                                },
                                // TODO: Unsupported file extensions go into PAK
                                // Io Store forces you to also make a pak file (hopefully DC's patches can fix this)
                                None => profiler.add_skipped_file(fs_obj.path().to_str().unwrap(), format!("Unsupported file type"), file_size)
                            }
                        },
                        None => {
                            if &name == ".utocmeta" {
                                let mut utoc_meta_lock = UTOC_METADATA.lock().unwrap();
                                if *utoc_meta_lock == None {
                                    *utoc_meta_lock = Some(UtocMetadata::new());
                                }
                                utoc_meta_lock.as_mut().unwrap().add_entries::<byteorder::NativeEndian>(fs::read(fs_obj.path()).unwrap());
                            } else {
                                profiler.add_skipped_file(fs_obj.path().to_str().unwrap(), format!("No file extension"), file_size);
                            }
                        }
                    }
                }
            },
            Err(e) => profiler.add_failed_fs_object(os_path.to_str().unwrap(), e.to_string())
        }
    }
}

pub fn print_asset_collector_results() {
    let profiler_lock = ASSET_COLLECTOR_PROFILER.lock().unwrap();
    if *profiler_lock != None {
        (*profiler_lock).as_ref().unwrap().print();
    }
}

#[derive(Debug, PartialEq)]
pub struct AssetCollectorProfilerFailedFsObject {
    os_path: String,
    reason: String
}

#[derive(Debug, PartialEq)]
pub struct AssetCollectorSkippedFileEntry {
    os_path: String,
    reason: String,
}

#[derive(Debug, PartialEq)]
pub struct AssetCollectorProfilerModContents {
    failed_file_system_objects: Vec<AssetCollectorProfilerFailedFsObject>,
    directory_count: u64,
    added_files_count: u64,
    added_files_size: u64,
    replaced_files_count: u64,
    replaced_files_size: u64,
    incorrect_asset_header: Vec<String>,
    skipped_files: Vec<AssetCollectorSkippedFileEntry>,
    skipped_file_size: u64,
}

impl AssetCollectorProfilerModContents {
    pub fn new() -> Self {
        Self {
            failed_file_system_objects: vec![],
            directory_count: 0,
            added_files_size: 0,
            added_files_count: 0,
            replaced_files_count: 0,
            replaced_files_size: 0,
            incorrect_asset_header: vec![],
            skipped_files: vec![],
            skipped_file_size: 0,
        }
    }
    
    pub fn add_failed_fs_object(&mut self, parent_dir: &str, reason: String) {
        self.failed_file_system_objects.push(AssetCollectorProfilerFailedFsObject { os_path: parent_dir.to_owned(), reason })
    }

    pub fn add_skipped_file(&mut self, os_path: &str, reason: String, size: u64) {
        self.skipped_files.push(AssetCollectorSkippedFileEntry { os_path: os_path.to_owned(), reason });
        self.skipped_file_size += size;
    }
    pub fn add_directory(&mut self) {
        self.directory_count += 1;
    }
    pub fn add_added_file(&mut self, size: u64) {
        self.added_files_count += 1;
        self.added_files_size += size;
    }
    pub fn add_replaced_file(&mut self, size: u64) {
        self.replaced_files_count += 1;
        self.replaced_files_size += size;
    }

    pub fn print(&self) {
        //println!("Created tree in {} ms", self.time_to_tree as f64 / 1000f64);
        println!("{} directories added", self.directory_count);
        println!("{} added files ({} KB)", self.added_files_count, self.added_files_size / 1024);
        println!("{} replaced files ({} KB)", self.replaced_files_count, self.replaced_files_size / 1024);
        if self.incorrect_asset_header.len() > 0 {
            println!("{}", "-".repeat(AssetCollectorProfiler::get_terminal_length()));
            println!("INCORRECT ASSET FORMAT: {} FILES", self.incorrect_asset_header.len());
            for i in &self.incorrect_asset_header {
                println!("{}", i);
            }
            println!("If you're the mod author, please make sure that you've followed the guide at \"https://github.com/AnimatedSwine37/UnrealEssentials\" to create correctly formatted assets");
        }
        if self.failed_file_system_objects.len() > 0 {
            println!("{}", "-".repeat(AssetCollectorProfiler::get_terminal_length()));
            println!("FAILED TO LOAD: {} FILES", self.failed_file_system_objects.len());
            for i in &self.failed_file_system_objects {
                println!("Inside folder \"{}\", reason \"{}\"", i.os_path, i.reason);
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct AssetCollectorProfilerMod {
    os_path: String,
    data: AssetCollectorProfilerModContents
}

impl AssetCollectorProfilerMod {
    pub fn new(mod_path: &str) -> Self {
        Self {
            os_path: mod_path.to_owned(),
            data: AssetCollectorProfilerModContents::new()
        }
    }

    fn print(&self) {
        println!("{}", self.os_path);
        self.data.print();
    }
}

#[derive(Debug, PartialEq)]
pub struct AssetCollectorProfiler {
    mods_loaded: Vec<AssetCollectorProfilerMod>,
}

impl AssetCollectorProfiler {
    pub fn get_terminal_length() -> usize {
        80
    }
    pub fn new() -> Self {
        Self {
            mods_loaded: vec![],
        }
    }
    pub fn print_centered(text: &str) {
        let left_spaces = (AssetCollectorProfiler::get_terminal_length() - text.len()) / 2;
        println!("{}{}", " ".repeat(left_spaces), text);
    }
    pub fn print(&self) {
        println!("{}", "#".repeat(AssetCollectorProfiler::get_terminal_length()));
        AssetCollectorProfiler::print_centered(&format!("ASSET COLLECTOR: Collected files from {} mods", self.mods_loaded.len()));
        println!("{}", "=".repeat(AssetCollectorProfiler::get_terminal_length()));
        for m in &self.mods_loaded {
            m.print();
            println!("{}", "=".repeat(AssetCollectorProfiler::get_terminal_length()));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::{asset_collector, asset_collector::ROOT_DIRECTORY, io_package::PackageSummary2, io_toc::IoStoreTocHeaderType3, toc_factory, 
        toc_factory::TocResolverCommon, toc_factory::TARGET_TOC, toc_factory::DEFAULT_COMPRESSION_BLOCK_ALIGNMENT};
    /* 
    #[test]
    fn test_collect_assets() {
        let mods = vec!["p3rpc.catherinefont", "p3rpc.classroomcheatsheet", "p3rpc.controlleruioverhaul.xbox", 
        "p3rpc.femc", "p3rpc.isitworking", "p3rpc.modmenu", "p3rpc.nocinematicbars", "p3rpc.removetalkfromdialogue",
        "p3rpc.rewatchtv", "p3rpc.ryojioutfit", "p3rpc.usefuldescriptions"];
        //let mods = vec!["p3rpc.femc"];
        let instant = std::time::Instant::now();
        let mut timers: Vec<u128> = Vec::with_capacity(mods.len());
        let base_path = std::env::var("RELOADEDIIMODS").unwrap_or_else(|err| panic!("Environment variable \"RELOADEDIIMODS\" is missing"));
        for (i, curr_mod) in mods.iter().enumerate() {
            asset_collector::add_from_folders(curr_mod, &(base_path.clone() + "/" + curr_mod + "/UnrealEssentials"));
            timers.insert(i, instant.elapsed().as_micros());
        }
        asset_collector::print_asset_collector_results();
        for (i, time) in timers.iter().enumerate() {
            println!("{}: {} ms", mods[i], (time - if i > 0 { timers[i - 1]} else { 0 }) as f64 / 1000f64);
        }
        /* 
        let mut profiler = toc_factory::TocBuilderProfiler::new();
        let mut resolver = toc_factory::TocResolverType2::new::<
            crate::io_toc::IoStoreTocHeaderType2
        >(TARGET_TOC, DEFAULT_COMPRESSION_BLOCK_ALIGNMENT);
        let root_dir_lock = ROOT_DIRECTORY.lock().unwrap();
        match (*root_dir_lock).as_ref() {
            Some(root) => {
                resolver.flatten_toc_tree(&mut toc_factory::TocFlattenTracker::new(), Arc::clone(&root));
                let serialize_results = resolver.serialize::<PackageSummary2, IoStoreTocHeaderType3>(&mut profiler, "");
            },
            None => todo!(),
        }
        */
    }
    */
}