
use std::{
    env,
    time::Instant
};
use fileemu_utoc_stream_emulator;
#[allow(unused_imports, unused_braces)]
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::HANDLE,
        Storage::FileSystem
    }
};

#[allow(unused_variables)]
fn main() {
    // TOC building test program without having to start Reloaded
    let r3_mod_path = env::var("RELOADEDIIMODS");
    if let Err(_) = r3_mod_path {
        println!("Environment variables is missing an entry for \"RELOADEDIIMODS\"");
        return;
    }
    let r3_mod_path = r3_mod_path.unwrap();
    println!("Using Reloaded mods directory at {}", r3_mod_path);
    // get our first test mods, convert to byte*
    let test_mod_1_id = "uetest.loosefiletest1";
    let test_mod_1 = r3_mod_path.clone() + "/" + test_mod_1_id;
    let test_mod_2_id = "uetest.loosefiletest2";
    let test_mod_2 = r3_mod_path.clone() + "/" + test_mod_2_id;
    let unreal_essentials_toc = r3_mod_path.clone() + "/UnrealEssentials/Unreal/UnrealEssentials_P.utoc";
    let unreal_essentials_partition = r3_mod_path.clone() + "/UnrealEssentials/Unreal/UnrealEssentials_P.ucas";

    fileemu_utoc_stream_emulator::asset_collector::add_from_folders(test_mod_1_id, &test_mod_1);
    fileemu_utoc_stream_emulator::asset_collector::add_from_folders(test_mod_2_id, &test_mod_2);
    unsafe { fileemu_utoc_stream_emulator::asset_collector::print_asset_collector_results(); }
    let toc = fileemu_utoc_stream_emulator::toc_factory::build_table_of_contents(&unreal_essentials_toc);
    match toc {
        Some(n) => {
            match std::fs::write(&unreal_essentials_toc, &n) {
                Ok(_) => {
                    fileemu_utoc_stream_emulator::toc_factory::build_container_test(&unreal_essentials_partition);
                }
                Err(_) => ()
            }
        },
        None => {
            println!("Failed to make TOC");
        }
    }
    // open TOC file handle
    /* 
    let toc_filename_win32 = unreal_essentials_toc.clone() + "\0";
    let start_making_toc = Instant::now();
    let mut add_from_first_folder = 0;
    let mut add_from_second_folder = 0;
    let mut build_toc = 0;
    unsafe {
        // https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilea
        // https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Storage/FileSystem/fn.CreateFileA.html
        match FileSystem::CreateFileA(
            PCSTR::from_raw(toc_filename_win32.as_ptr()),
            FileSystem::FILE_GENERIC_WRITE.0,
            FileSystem::FILE_SHARE_MODE(0),
            None,
            FileSystem::FILE_CREATION_DISPOSITION(FileSystem::CREATE_ALWAYS.0),
            FileSystem::FILE_FLAGS_AND_ATTRIBUTES(FileSystem::FILE_ATTRIBUTE_NORMAL.0),
            HANDLE::default()
        ) {
            Ok(handle) => {
                println!("Got TOC handle!");
                /* fileemu_utoc_stream_emulator::toc_factory::add_from_folders(&test_mod_1);
                add_from_first_folder = start_making_toc.elapsed().as_micros();
                //fileemu_utoc_stream_emulator::add_from_folders(&test_mod_2);
                //add_from_second_folder = start_making_toc.elapsed().as_micros() - add_from_first_folder;
                fileemu_utoc_stream_emulator::toc_factory::build_table_of_contents(handle.0, &unreal_essentials_toc, &unreal_essentials_partition);
                build_toc = start_making_toc.elapsed().as_micros() - add_from_second_folder;
                */
            }
            Err(e) => println!("Error occurred trying to open file: {}", e.to_string())
        }
    }
    */
    /* 
    println!("Added files from first mod in {} ms", add_from_first_folder as f64 / 1000f64);
    println!("Added files from second mod in {} ms", add_from_second_folder as f64 / 1000f64);
    println!("Built Table of Contents in {} ms", build_toc as f64 / 1000f64); // This section is slow...
    println!("Total: {} ms", (add_from_first_folder + add_from_second_folder + build_toc) as f64 / 1000f64);
    */

}