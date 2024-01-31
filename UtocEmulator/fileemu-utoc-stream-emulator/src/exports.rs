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
pub unsafe extern "C" fn AddFromFolders(modId: *const c_char, modPath: *const c_char) {
    asset_collector::add_from_folders(CStr::from_ptr(modId).to_str().unwrap(), CStr::from_ptr(modPath).to_str().unwrap());
}

#[no_mangle]
#[allow(non_snake_case)]
// haiiii Reloaded!!!! :3
pub unsafe extern "C" fn BuildTableOfContents(tocPath: *const c_char, settings: *const u32, settings_length: u32, length: *mut u64) -> *const u8 {
    match toc_factory::build_table_of_contents(CStr::from_ptr(tocPath).to_str().unwrap(), 0) {
        Some(n) => {
            *length = n.len() as u64; // set length parameter
            n.leak().as_ptr() // leak memory lol
        },
        None => 0 as *const u8 // couldn't build toc, let C# side know with a null pointer
    }
}

#[no_mangle]
#[allow(non_snake_case)]
// haiiii Reloaded!!!! :3
pub unsafe extern "C" fn BuildTableOfContentsEx(
    // UTOC
    basePath: *const c_char,
    version: u32,
    tocData: *mut *const u8,
    tocLength: *mut u64,
    // UCAS
    blocks: *mut *const PartitionBlock,
    blockCount: *mut usize,
    header: *mut *const u8,
    headerSize: *mut usize
    ) -> bool {
    let base_path_owned = CStr::from_ptr(basePath).to_str().unwrap();
    let toc_path = base_path_owned.to_owned() + "\\" + TARGET_TOC;
    let cas_path = base_path_owned.to_owned() + "\\" + TARGET_CAS;
    match toc_factory::build_table_of_contents(&toc_path, version) {
        Some(n) => {
            println!("Built table of contents");
            // UTOC
            *tocLength = n.len() as u64; // set length parameter
            *tocData = n.leak().as_ptr(); // leak memory lol (toc data needs to live for rest of program)
            /* 
            match toc_factory::get_virtual_partition(&cas_path) {
                Some(n) => {
                    *blockCount = n.0.len(); // container blocks
                    *blocks = n.0.as_ptr();
                    *headerSize = n.1.len(); // container header
                    *header = n.1.as_ptr();
                    true
                }, 
                None => false
            }
            */
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
pub unsafe extern "C" fn GetContainerBlocks(
    casPath: *const c_char, 
    blocks: *mut *const PartitionBlock, blockCount: *mut usize, 
    header: *mut *const u8, headerSize: *mut usize
) -> bool {
    false
    /* 
    let block_managed = toc_factory::get_virtual_partition(CStr::from_ptr(casPath).to_str().unwrap());
    match block_managed {
        Some(n) => {
            *blockCount = n.0.len(); // container blocks
            *blocks = n.0.as_ptr();
            *headerSize = n.1.len(); // container header
            *header = n.1.as_ptr();
            true
        },
        None => false
    }
    */
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn PrintAssetCollectorResults() {
    asset_collector::print_asset_collector_results();
}