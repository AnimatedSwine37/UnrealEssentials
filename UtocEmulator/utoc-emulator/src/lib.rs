pub(crate) mod ffi;
pub(crate) mod logger;
pub(crate) mod metadata;
pub(crate) mod assets;
pub(crate) mod factory;
pub(crate) mod progress;

use std::error::Error;

pub(crate) type GenericResult<T> = Result<T, Box<dyn Error>>;

#[cfg(test)]
pub mod tests {
    use std::path::Path;
    use std::ptr::NonNull;
    use retoc::version::EngineVersion;
    use crate::GenericResult;
    use crate::assets::AssetCollection;
    use crate::factory::IoStoreFactory;
    use crate::ffi::Array;
    use crate::logger::{invoke_println, set_reloaded_logger};

    #[test]
    fn package_test() -> GenericResult<()> {
        // for Persona 3 Reload
        let version = EngineVersion::UE4_27;
        unsafe { set_reloaded_logger(invoke_println) };
        AssetCollection::add_from_folder(
            Path::new("E:/Reloaded-II/Mods/p3rpc.isitworking/UnrealEssentials"),
            version)?;
        let mut toc = Array::default();
        let mut partition = Array::default();
        let mut header = Array::default();
        IoStoreFactory::build(
            version,
            &mut toc,
            &mut partition,
            &mut header
        )?;
        Ok(())
    }
}