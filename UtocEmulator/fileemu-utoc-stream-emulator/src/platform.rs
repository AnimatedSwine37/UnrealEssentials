use std::fs::DirEntry;

#[cfg(target_os = "linux")]
use std::os::linux;

#[cfg(target_os = "unix")]
use std::os::unix;

#[cfg(target_os = "windows")]
use std::os::windows;

pub struct Metadata;

impl Metadata {
    #[cfg(target_os = "linux")]
    pub fn get_file_size(fs_obj: &DirEntry) -> u64 {
        let meta = fs_obj.metadata().unwrap();
        linux::fs::MetadataExt::st_size(&meta)
    }

    #[cfg(target_os = "unix")]
    pub fn get_file_size(fs_obj: &DirEntry) -> u64 {
        let meta = fs_obj.metadata().unwrap();
        linux::fs::MetadataExt::size(&meta)
    }

    #[cfg(target_os = "windows")]
    pub fn get_file_size(fs_obj: &DirEntry) -> u64 {
        let meta = fs_obj.metadata().unwrap();
        windows::fs::MetadataExt::file_size(&meta)
    }
}