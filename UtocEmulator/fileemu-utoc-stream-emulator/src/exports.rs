use crate::{
    asset_collector, 
    toc_factory, toc_factory::{CONTAINER_DATA, CONTAINER_ENTRIES_OSPATH_POOL, TARGET_TOC, TARGET_CAS, PartitionBlock}
};
use std::{
    ffi::CStr,
    os::raw::c_char
};

#[no_mangle]
#[allow(non_snake_case)]
// modId is used by the asset collector profiler
pub unsafe extern "C" fn AddFromFolders(modPath: *const u16, modPathLength: usize) {
    let mod_path_slice = std::slice::from_raw_parts(modPath, modPathLength);
    asset_collector::add_from_folders(&String::from_utf16(mod_path_slice).unwrap());
}

#[no_mangle]
#[allow(non_snake_case)]
// haiiii Reloaded!!!! :3
pub unsafe extern "C" fn BuildTableOfContentsEx(
    // UTOC
    basePath: *const u16,
    basePathLength: usize,
    version: u32,
    tocData: *mut *const u8,
    tocLength: *mut u64,
    // UCAS
    blocks: *mut *const PartitionBlock,
    blockCount: *mut usize,
    header: *mut *const u8,
    headerSize: *mut usize
    ) -> bool {
    let mod_path_slice = std::slice::from_raw_parts(basePath, basePathLength);
    let base_path_owned = &String::from_utf16(mod_path_slice).unwrap();
    let toc_path = base_path_owned.to_owned() + "\\" + TARGET_TOC;
    let cas_path = base_path_owned.to_owned() + "\\" + TARGET_CAS;
    match toc_factory::build_table_of_contents(&toc_path, version) {
        Some(n) => {
            println!("Built table of contents");
            // UTOC
            *tocLength = n.len() as u64; // set length parameter
            *tocData = n.leak().as_ptr(); // leak memory lol (toc data needs to live for rest of program)
            // UCAS
            let container_lock = CONTAINER_DATA.lock().unwrap();
            match (*container_lock).as_ref() {
                Some(n) => {
                    println!("Built container file");
                    *blocks = n.virtual_blocks.as_ptr();
                    *blockCount = n.virtual_blocks.len();
                    *header = n.header.as_ptr();
                    *headerSize = n.header.len();
                    true
                },
                None => false
            }
        },
        None => false
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn PrintAssetCollectorResults() {
    asset_collector::print_asset_collector_results();
}