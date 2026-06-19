use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use retoc::version::EngineVersion;
use walkdir::{DirEntry, WalkDir};
use utoc_lib::assets::*;
use utoc_lib::store::os_file_size;
use crate::GenericResult;
use crate::metadata::MetadataState;

type AssetListMap = HashMap<String, AssetEntry>;
pub static ASSET_LIST: Mutex<Option<AssetListMap>> = Mutex::new(None);

#[derive(Debug)]
pub struct AssetCollection;

impl AssetCollection {
    pub(crate) fn instance() -> MutexGuard<'static, Option<AssetListMap>> {
        let mut guard = ASSET_LIST.lock().unwrap();
        if guard.is_none() {
            *guard = Some(HashMap::new());
        }
        guard
    }

    // Essentials 1.x: utocmeta is treated as a filename only
    fn filter_utocmeta(d: &DirEntry) -> bool {
        d.depth() == 1 && d.path().file_name().map_or(
            false, |v| v.to_str().unwrap() == UTOCMETA)
    }

    pub(crate) fn filter_dir_entries(dir_entry: walkdir::Result<DirEntry>) -> Option<DirEntry> {
        dir_entry.ok()
            .and_then(|d| {
                // must be a file
                let is_file = d.metadata().ok().map_or(false, |m| m.is_file());
                // check the file format!
                let check_ext = d.path().extension().map_or(
                    false, |ext| ASSET_EXTENSIONS.contains(&ext.to_str().unwrap()))
                    || Self::filter_utocmeta(&d);
                if !is_file || !check_ext { return None }
                Some(d)
            }
        )
    }

    /// Recursively registers all the assets inside of a folder into the asset list to get replaced.
    /// If you are working with an asset type that can be partially written to such as a data table,
    /// use UE Toolkit (https://github.com/RyoTune/UE.Toolkit) as it allows for file merging
    pub(crate) fn add_from_folder<P: AsRef<Path>>(path: P, version: EngineVersion) -> GenericResult<()> {
        let path = path.as_ref().to_owned();
        if !path.exists() { return Ok(()); }
        Self::add_from_folder_inner(path, None, version)
    }

    pub(crate) fn add_from_folder_inner(path: PathBuf, mount: Option<PathBuf>,
        version: EngineVersion) -> GenericResult<()> {
        for file in WalkDir::new(&path).into_iter().filter_map(Self::filter_dir_entries) {
            let os_path = file.path().to_owned();
            match os_path.extension().map(|s| s.to_str().unwrap()) {
                Some(UASSETMETA_EXTENSION) => {
                    let asset_path = convert_to_asset_path(&os_path, path.as_path(), mount.as_ref());
                    MetadataState::instance().as_mut().unwrap().add_from_uassetmeta(
                        asset_path_to_package_id(&asset_path), os_path.as_path())?;
                },
                Some(_) => {
                    let asset_path = convert_to_asset_path(&os_path, path.as_path(), mount.as_ref());
                    let file_size = os_file_size(&file.metadata()?);
                    Self::instance().as_mut().unwrap().insert(asset_path, AssetEntry::new(os_path, file_size));
                },
                None => match os_path.file_name().map(|f| f.to_str().unwrap()) {
                    Some(UTOCMETA) => {
                        MetadataState::instance().as_mut().unwrap().add_from_utocmeta(
                            std::fs::read(file.path())?.as_slice(), version)?;
                    },
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub(crate) fn add_from_folder_with_mount<P0: AsRef<Path>, P1: AsRef<Path>>(
        path: P0, mount: P1, version: EngineVersion) -> GenericResult<()> {
        let (path, mount) = (path.as_ref().to_owned(), mount.as_ref().to_owned());
        if !path.exists() || !mount.exists() { return Ok(()); }
        Self::add_from_folder_inner(path, Some(mount), version)
    }
}