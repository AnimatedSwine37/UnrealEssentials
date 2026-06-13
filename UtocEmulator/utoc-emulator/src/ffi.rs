use std::path::Path;
use std::ptr::NonNull;
use std::sync::OnceLock;
use retoc::container_header::EIoContainerHeaderVersion;
use retoc::EIoStoreTocVersion;
use crate::log;
use crate::assets::AssetCollection;
use crate::factory::IoStoreFactory;

/// This must stay in sync with EngineVersion in UTOC.Stream.Emulator.Interfaces over in C# land!
#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum EngineVersion
{
    UE_4_25 = (4 << 0x8) + 25,
    UE_4_26 = (4 << 0x8) + 26, // 4.25+ (e.g Scarlet Nexus) is treated as 4.26
    UE_4_27 = (4 << 0x8) + 27,
    UE_5_0 = (5 << 0x8) + 0,
    UE_5_1 = (5 << 0x8) + 1,
    UE_5_2 = (5 << 0x8) + 2,
    UE_5_3 = (5 << 0x8) + 3,
    UE_5_4 = (5 << 0x8) + 4,
    UE_5_5 = (5 << 0x8) + 5,
    UE_5_6 = (5 << 0x8) + 6,
    UE_5_7 = (5 << 0x8) + 7,
    // if there are any games that require a special ID, then define them with [value] << 0x10
    // FF7R = (4 << 0x8) + 27 | 1 << 0x10
}

impl EngineVersion {
    pub fn to_retoc(&self) -> retoc::version::EngineVersion {
        match self {
            Self::UE_4_25 => retoc::version::EngineVersion::UE4_25,
            Self::UE_4_26 => retoc::version::EngineVersion::UE4_26,
            Self::UE_4_27 => retoc::version::EngineVersion::UE4_27,
            Self::UE_5_0  => retoc::version::EngineVersion::UE5_0,
            Self::UE_5_1  => retoc::version::EngineVersion::UE5_1,
            Self::UE_5_2  => retoc::version::EngineVersion::UE5_2,
            Self::UE_5_3  => retoc::version::EngineVersion::UE5_3,
            Self::UE_5_4  => retoc::version::EngineVersion::UE5_4,
            Self::UE_5_5  => retoc::version::EngineVersion::UE5_5,
            Self::UE_5_6  => retoc::version::EngineVersion::UE5_6,
            Self::UE_5_7  => retoc::version::EngineVersion::UE5_7,
        }
    }

    pub fn to_toc_version(&self) -> EIoStoreTocVersion {
        self.to_retoc().toc_version()
    }

    pub fn to_cas_header_version(&self) -> EIoContainerHeaderVersion {
        self.to_retoc().container_header_version()
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Array<T> {
    pub(crate) data: *const T,
    pub(crate) len: usize
}

impl<T> Default for Array<T> {
    fn default() -> Self {
        Self {
            data: std::ptr::null(),
            len: 0
        }
    }
}

impl<T> From<Vec<T>> for Array<T> {
    fn from(value: Vec<T>) -> Self {
        let boxed = value.leak();
        Self {
            data: boxed.as_ptr() as _,
            len: boxed.len()
        }
    }
}

#[repr(C)]
pub struct PartitionBlock {
    pub(crate) os_path: *const u8,
    pub(crate) start: u64,
    pub(crate) length: u64,
}

impl PartitionBlock {
    pub fn new(path: &str, start: u64, length: u64) -> Self {
        Self {
            os_path: format!("{}\0", path).encode_utf16().collect::<Vec<u16>>().leak().as_ptr() as _,
            start,
            length
        }
    }
}

// CSharpString from riri-mod-tools:
// https://github.com/rirurin/riri-mod-tools/blob/main/riri-mod-tools-rt/src/mod_loader_data.rs
type FreeStrFn = unsafe extern "C" fn(*const u16) -> ();

pub static FREE_CSHARP_STRING: OnceLock<FreeStrFn> = OnceLock::new();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_free_csharp_string(cb: FreeStrFn) {
    FREE_CSHARP_STRING.set(cb).unwrap();
}

#[repr(C)]
#[derive(Debug)]
pub struct CSharpString(*const u16);
impl CSharpString {
    pub fn new(p: *const u16) -> Self {
        Self(p)
    }
}
impl From<CSharpString> for String {
    fn from(value: CSharpString) -> Self {
        let mut len = 0;
        while unsafe { *value.0.add(len) } != 0 {
            len += 1;
        }
        let s = unsafe { std::slice::from_raw_parts(value.0, len) };
        String::from_utf16(s).unwrap()
    }
}
impl Drop for CSharpString {
    fn drop(&mut self) {
        unsafe { FREE_CSHARP_STRING.get().unwrap()(self.0) }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn add_from_folders(
    mod_path: CSharpString,
    version: EngineVersion) {
    AssetCollection::add_from_folder(Into::<String>::into(mod_path), version.to_retoc()).unwrap();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn add_from_folders_with_mount(
    mod_path: CSharpString,
    virtual_path: CSharpString,
    version: EngineVersion) {
    AssetCollection::add_from_folder_with_mount(
        Into::<String>::into(mod_path),
        Into::<String>::into(virtual_path),
        version.to_retoc()
    ).unwrap();
}

#[unsafe(no_mangle)]
// haiiii Reloaded!!!! :3
pub unsafe extern "C" fn build_toc(
    version: EngineVersion,
    mut toc: NonNull<Array<u8>>,
    mut blocks: NonNull<Array<PartitionBlock>>,
    mut header: NonNull<Array<u8>>
) -> bool {
    let result = IoStoreFactory::build(
        version.to_retoc(),
        unsafe { toc.as_mut() },
        unsafe { blocks.as_mut() },
        unsafe { header.as_mut() }
    );
    result.is_ok()
}