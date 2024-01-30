use crate::{
    io_package,
    platform::Metadata,
    toc_factory::TARGET_TOC
};
use std::{
    cell::RefCell,
    collections::BTreeSet,
    fs, fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    rc::{Rc, Weak},
    sync::Mutex,
    time::Instant
};

pub type TocDirectoryRef = Rc<RefCell<TocDirectory>>;
pub type TocFileRef = Rc<RefCell<TocFile>>;

pub const FILE_EMULATION_FRAMEWORK_FOLDER:  &'static str = "FEmulator";
pub const EMULATOR_NAME:                    &'static str = "UTOC";
pub const PROJECT_NAME:                     &'static str = "UnrealEssentials";

// Root TOC directory (needs to be global)
pub static mut ROOT_DIRECTORY: Option<TocDirectoryRef> = None;
pub static mut ASSET_COLLECTOR_PROFILER: Option<AssetCollectorProfiler> = None;

// Create tree of assets that can be used to build a TOC
pub fn add_from_folders(mod_id: &str, mod_path: &str) {
    unsafe { // Check profiler is active
        if ASSET_COLLECTOR_PROFILER == None {
            ASSET_COLLECTOR_PROFILER = Some(AssetCollectorProfiler::new());
        }
    }
    let mod_path: PathBuf = [mod_path, FILE_EMULATION_FRAMEWORK_FOLDER, EMULATOR_NAME, TARGET_TOC].iter().collect();
    if Path::exists(Path::new(&mod_path)) {
        let mut profiler_mod = AssetCollectorProfilerMod::new(mod_id, mod_path.to_str().unwrap());
        // Mutating a global variable is UB in a multithreaded context
        // Yes the compiler will complain about this
        // https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#accessing-or-modifying-a-mutable-static-variable
        unsafe {
            if let None = ROOT_DIRECTORY {
                ROOT_DIRECTORY = Some(TocDirectory::new_rc(PROJECT_NAME)); // ProjectName
            }
            add_from_folders_inner(Rc::clone(&ROOT_DIRECTORY.as_ref().unwrap()), &mod_path, &mut profiler_mod.data);
            profiler_mod.set_time_to_tree();
            ASSET_COLLECTOR_PROFILER.as_mut().unwrap().mods_loaded.push(profiler_mod);
        }
    }
}

//      A <--------
//      ^    ^    ^
//      |    |    | (refs from child -> parent)
//      v    |    | (owns from parent -> child and in sibling and file linked lists)
//      B -> C -> D

pub struct TocDirectory {
    pub name:           String, // leaf name only (directory name or file name)
    pub parent:         Weak        <RefCell<TocDirectory>>, // weakref to parent for path building for FIoChunkIds
    pub first_child:    Option      <TocDirectoryRef>, // first child
    last_child:         Weak        <RefCell<TocDirectory>>, // O(1) insertion on directory add
    pub next_sibling:   Option      <TocDirectoryRef>, // next sibling
    pub first_file:     Option      <TocFileRef>, // begin file linked list, owns file children
    last_file:          Weak        <RefCell<TocFile>>, // O(1) insertion on file add
}

impl TocDirectory {
    // constructor
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            parent: Weak::new(),
            first_child: None,
            last_child: Weak::new(),
            next_sibling: None, // root folder has no siblings
            first_file: None,
            last_file: Weak::new(),
        }
    }
    #[inline] // convenience function to create reference counted toc directories
    pub fn new_rc(name: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(TocDirectory::new(name)))
    }
    // Returns true/false depending on if the target directory contains any child directories
    pub fn has_children(dir: TocDirectoryRef) -> bool {
        match &dir.borrow().first_child {
            Some(_) => true,
            None => false
        }
    }
    // Returns true/false depending on if the target directory contains any child files
    pub fn has_files(dir: TocDirectoryRef) -> bool {
        match &dir.borrow().first_file {
            Some(_) => true,
            None => false
        }
    }
    // Add a file child into directory that doesn't currently contain any other files
    #[inline]
    fn add_first_file(dir: TocDirectoryRef, file: TocFileRef) {
        dir.borrow_mut().first_file = Some(Rc::clone(&file));
        dir.borrow_mut().last_file = Rc::downgrade(&file);
    }
    // Replace an existing file in the file list. Kick it off the list so it drops on add_or_replace_file's scope
    #[inline]
    fn replace_file(
        dir: TocDirectoryRef, // containing directory
        prev_file: Option<TocFileRef>, // previous file, which links to replacee (unless it's the *first* file)
        replacee: TocFileRef, // the file to get replaced (file merging is a future problem)
        replacer: TocFileRef // file that'll take the place of replacee in the chain
    ) {
        if replacee.borrow().next.as_ref() == None { // replacee is the last file in chain, dir->last_file = weakref(replacer)
            dir.borrow_mut().last_file = Rc::downgrade(&replacer);
        } else { // replacee is at the start or in the middle, set replacer->next = replacee->next
            replacer.borrow_mut().next = Some(Rc::clone(replacee.borrow().next.as_ref().unwrap()));
        }
        if prev_file == None { // replacee is at the start, set dir->first_file = replacer
            dir.borrow_mut().first_file = Some(Rc::clone(&replacer));
        } else { // prev->next = replacer, replacee drops here
            prev_file.as_ref().unwrap().borrow_mut().next = Some(Rc::clone(&replacer));
        }

    }
    // Add a file to the end of the directory's file list, which contains at least 1 existing file
    #[inline]
    fn add_another_file(dir: TocDirectoryRef, file: TocFileRef) {
        dir.borrow().last_file.upgrade().unwrap().borrow_mut().next = Some(Rc::clone(&file)); // own our new child on the end of children linked list
        dir.borrow_mut().last_file = Rc::downgrade(&file); // and set the tail to weakref of the new child
    }
    // go through file list to check if the target file already exists, then replace it with our own
    // otherwise, add our file to the end
    pub fn add_or_replace_file(dir: TocDirectoryRef, file: TocFileRef) -> TocFileAddType {
        match TocDirectory::has_files(Rc::clone(&dir)) {
            true => { // :adachi_true: - search file linked list
                let mut found = false;
                let mut prev: Option<TocFileRef> = None;
                let mut curr_file = Rc::clone(dir.borrow().first_file.as_ref().unwrap());
                loop {
                    if curr_file.borrow().name == file.borrow().name { // we got the file, replace it
                        found = true;
                        break
                    }
                    match Rc::clone(&curr_file).borrow().next.as_ref() { // check if next points to next entry in chain or ends the chain
                        Some(f) => {
                            prev = Some(Rc::clone(&curr_file));
                            curr_file = Rc::clone(&f);
                        },
                        None => { // couldn't find it to replace, add it to the end
                            break // we need to escape this scope to prevent creating mut ref of last_file->next while const ref last_file->next is still valid
                        }
                    }
                }
                if !found {
                    TocDirectory::add_another_file(Rc::clone(&dir), Rc::clone(&file));
                    TocFileAddType::Addition
                } else {
                    TocDirectory::replace_file(
                        Rc::clone(&dir),
                        prev, // prev is only set with second file in list onwards
                        Rc::clone(&curr_file),
                        Rc::clone(&file)
                    );
                    TocFileAddType::Replacement
                }
            },
            false => {
                TocDirectory::add_first_file(Rc::clone(&dir), Rc::clone(&file));
                TocFileAddType::Addition
            }
        }
    }
    // get a child directory from a parent directory if it exists
    pub fn get_child_dir(parent: TocDirectoryRef, exist: &str) -> Option<TocDirectoryRef> {
        match TocDirectory::has_children(Rc::clone(&parent)) {
            true => {
                let mut curr_dir = Rc::clone(&parent.borrow().first_child.as_ref().unwrap());
                let mut result = None;
                loop {
                    if curr_dir.borrow().name == exist { // we got our directory
                        result = Some(Rc::clone(&curr_dir));
                        break;
                    }
                    match Rc::clone(&curr_dir).borrow().next_sibling.as_ref() {
                        Some(ip) => curr_dir = Rc::clone(&ip),
                        None => break
                    }
                }
                result
            },
            false => None // has no children, can only not exist
        }
    }
    pub fn add_directory(parent: TocDirectoryRef, child: TocDirectoryRef) {
        child.borrow_mut().parent = Rc::downgrade(&parent); // set child node's parent as weak ref of parent 
        if !TocDirectory::has_children(Rc::clone(&parent)) { // if parent has no nodes (if let doesn't work here since scope of &first_child extends to entire statement, overlapping &mut first_child)
            parent.borrow_mut().first_child = Some(Rc::clone(&child)); // head and tail set to new child
            parent.borrow_mut().last_child = Rc::downgrade(&child);
            return;
        }
        parent.borrow().last_child.upgrade().as_ref().unwrap().borrow_mut().next_sibling = Some(Rc::clone(&child));
        parent.borrow_mut().last_child = Rc::downgrade(&child);
    }
}

#[derive(Debug, PartialEq)]
pub struct TocFile {
    pub next: Option<Rc<RefCell<TocFile>>>,
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
    pub fn new_rc(name: &str, file_size: u64, os_path: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(TocFile::new(name, file_size, os_path)))
    }
}

pub enum TocFileAddType {
    Addition,
    Replacement
}

pub const SUITABLE_FILE_EXTENSIONS: &'static [&'static str] = ["uasset", "ubulk", "uptnl"].as_slice();
pub const MOUNT_POINT: &'static str = "../../../";

pub fn add_from_folders_inner(parent: TocDirectoryRef, os_path: &PathBuf, profiler: &mut AssetCollectorProfilerModContents) {
    // We've already checked that this path exists in AddFromFolders, so unwrap directly
    // This folder is equivalent to /[ProjectName]/Content, so our mount point will be
    // at least ../../../[ProjectName] (../../../Game/)
    // build an unsorted n-tree of directories and files, preorder traversal
    // higher priority mods should overwrite contents of files, but not directories
    for i in fs::read_dir(os_path).unwrap() {
        match &i {
            Ok(fs_obj) => { // we have our file system object, now determine if it's a directory or folder
                let fs_obj_os_name = fs_obj.file_name(); // this does assume that the object name is valid Unicode
                let name = String::from(fs_obj_os_name.to_str().unwrap()); // if it's not i'll be very surprised
                let file_type = fs_obj.file_type().unwrap();
                if file_type.is_dir() { // new directory. mods can only expand on this
                    let mut inner_path = PathBuf::from(os_path);
                    inner_path.push(&name);
                    match TocDirectory::get_child_dir(Rc::clone(&parent), &name) {
                        // check through folder regardless since there may be new inner folders in there
                        Some(child_dir) => add_from_folders_inner(Rc::clone(&child_dir), &inner_path, profiler),
                        None => {
                            // this is a new directory, create it and then check inside it
                            let new_dir = TocDirectory::new_rc(&name);
                            TocDirectory::add_directory(Rc::clone(&parent), Rc::clone(&new_dir));
                            add_from_folders_inner(Rc::clone(&new_dir), &inner_path, profiler);
                            profiler.add_directory();
                        }
                    }
                } else if file_type.is_file() {
                    let file_size = Metadata::get_file_size(fs_obj);
                    match PathBuf::from(&name).extension() {
                        Some(ext) => {
                            let ext_str = ext.to_str().unwrap();
                            match SUITABLE_FILE_EXTENSIONS.iter().find(|exist| **exist == ext_str) {
                                // it's a matter of either replacing an existing file or adding a new file
                                // ,,,at least until we start thinking about merging P3RE persona tables (lol)
                                Some(io_ext) => {
                                    if *io_ext == "uasset" { // export bundles - requires checking file header to ensure that it doesn't have the cooked asset signature
                                        let current_file = File::open(fs_obj.path().to_str().unwrap()).unwrap();
                                        let mut file_reader = BufReader::with_capacity(4, current_file);
                                        if !io_package::is_valid_asset_type::<BufReader<File>, byteorder::NativeEndian>(&mut file_reader) {
                                            profiler.add_skipped_file(os_path.to_str().unwrap(), format!("Uses cooked package"), file_size);
                                            continue
                                        }
                                    }
                                    let new_file = TocFile::new_rc(&name, file_size, fs_obj.path().to_str().unwrap());
                                    match TocDirectory::add_or_replace_file(Rc::clone(&parent), Rc::clone(&new_file)) {
                                        TocFileAddType::Addition => profiler.add_added_file(file_size),
                                        TocFileAddType::Replacement => profiler.add_replaced_file(file_size)
                                    }
                                },
                                // TODO: Unsupported file extensions go into PAK
                                // Io Store forces you to also make a pak file (hopefully DC's patches can fix this)
                                None => profiler.add_skipped_file(fs_obj.path().to_str().unwrap(), format!("Unsupported file type"), file_size)
                            }
                        }
                        None => profiler.add_skipped_file(fs_obj.path().to_str().unwrap(), format!("No file extension"), file_size)
                    }
                }
            },
            Err(e) => profiler.add_failed_fs_object(os_path.to_str().unwrap(), e.to_string())
        }
    }
}

// Safety: this checks if ASSET_COLLECTOR_PROFILER has been assigned a value first, which only happens after loading a mod
pub unsafe fn print_asset_collector_results() {
    if ASSET_COLLECTOR_PROFILER == None {
        return;
    }
    ASSET_COLLECTOR_PROFILER.as_ref().unwrap().print();
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
    timer: Instant,
    time_to_tree: u128,
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
            timer: Instant::now(),
            time_to_tree: 0,
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
    pub fn get_tree_time(&mut self) {
        self.time_to_tree = self.timer.elapsed().as_micros();
    }

    pub fn print(&self) {
        println!("Created tree in {} ms", self.time_to_tree as f64 / 1000f64);
        println!("{} directories added", self.directory_count);
        println!("{} added files ({} KB)", self.added_files_count, self.added_files_size / 1024);
        println!("{} replaced files ({} KB)", self.replaced_files_count, self.replaced_files_size / 1024);
        if self.skipped_files.len() > 0 {
            println!("{}", "-".repeat(80));
            println!("SKIPPED FILES: {} FILES ({} KB)", self.skipped_files.len(), self.skipped_file_size / 1024);
            for i in &self.skipped_files {
                println!("File \"{}\", reason \"{}\"", i.os_path, i.reason);
            }
        }
        if self.incorrect_asset_header.len() > 0 {
            println!("{}", "-".repeat(AssetCollectorProfiler::get_terminal_length()));
            println!("INCORRECT ASSET FORMAT: {} FILES", self.incorrect_asset_header.len());
            for i in &self.incorrect_asset_header {
                println!("{}", i);
            }
            println!("If you're the mod author, please make sure that you've followed the guide at [insert docs here] to create correctly formatted assets");
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
    uid: String, // p3rpc.modname
    os_path: String,
    data: AssetCollectorProfilerModContents
}

impl AssetCollectorProfilerMod {
    pub fn new(mod_id: &str, mod_path: &str) -> Self {
        let modified_os_path = mod_path.split_once("C:\\Users");
        Self {
            uid: mod_id.to_owned(),
            os_path: mod_path.to_owned(),
            data: AssetCollectorProfilerModContents::new()
        }
    }
    pub fn set_time_to_tree(&mut self) {
        self.data.time_to_tree = self.data.timer.elapsed().as_micros();
    }

    fn print(&self) {
        println!("{}", self.uid);
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