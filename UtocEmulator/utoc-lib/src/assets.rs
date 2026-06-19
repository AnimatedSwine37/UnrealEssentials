use std::path::{Path, PathBuf};
use retoc::{lower_utf16_cityhash, FPackageId};

pub const MOUNT_POINT:  &'static str = "../../../";

pub const ENGINE_DOMAIN: &'static str = "Engine";

pub const UASSET_EXTENSION: &'static str = "uasset";
pub const UBULK_EXTENSION: &'static str = "ubulk";
pub const UPTNL_EXTENSION: &'static str = "uptnl";
pub const UMAP_EXTENSION: &'static str = "umap";

pub const UTOCMETA: &'static str = ".utocmeta";
pub const UASSETMETA_EXTENSION: &'static str = "uassetmeta";

pub static ASSET_EXTENSIONS: [&'static str; 5] = [
    UASSET_EXTENSION,
    UBULK_EXTENSION,
    UPTNL_EXTENSION,
    UMAP_EXTENSION,
    UASSETMETA_EXTENSION,
];

#[repr(u32)]
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum AssetType {
    UnrealAsset,
    BulkData,
    OptionalBulkData,
    UnrealMap,
    EssentialsAssetMetadata,
}

impl AssetType {
    pub(crate) fn get_extension(&self) -> &str {
        ASSET_EXTENSIONS[*self as usize]
    }
}

impl From<&str> for AssetType {
    fn from(value: &str) -> Self {
        match value {
            UASSET_EXTENSION => AssetType::UnrealAsset,
            UBULK_EXTENSION => AssetType::BulkData,
            UPTNL_EXTENSION => AssetType::OptionalBulkData,
            UMAP_EXTENSION => AssetType::UnrealMap,
            UASSETMETA_EXTENSION => AssetType::EssentialsAssetMetadata,
            _ => panic!("Unknown file extension for AssetType (this should have been caught earlier!)")
        }
    }
}

/// The input path is expected to be relative to the UnrealEssentials folder:
/// e.g The path's value should be P3R/Content/...
pub fn convert_to_asset_path<P0, P1>(path: P0, base: P1, vpath: Option<&PathBuf>) -> String
where P0: AsRef<Path>, P1: AsRef<Path> {
    let path = {
        let path = path.as_ref().strip_prefix(base.as_ref()).unwrap().to_str().unwrap().to_owned();
        if cfg!(target_os = "windows") {
            path.replace("\\", "/")
        } else {
            path
        }
    };
    let parts: Vec<&str> = path.splitn(3, "/").collect();
    // check that path is that long
    let domain = match parts[0] {
        ENGINE_DOMAIN => ENGINE_DOMAIN,
        _ => "Game"
    };
    match vpath {
        Some(v) => {
            let vpath = v.to_str().unwrap();
            format!("../../../{}/{}/{}", vpath, domain, parts[2])
        },
        None => format!("../../../{}/{}", domain, parts[2])
    }
}

pub fn convert_to_package_id<P0, P1>(path: P0, base: P1, vpath: Option<&PathBuf>) -> FPackageId
where P0: AsRef<Path>, P1: AsRef<Path> {
    let asset_path = convert_to_asset_path(path, base, vpath);
    asset_path_to_package_id(&asset_path)
}

pub fn asset_path_to_package_id(asset_path: &str) -> FPackageId {
    let asset_path = asset_path[MOUNT_POINT.len() - 1..].rsplit_once('.').unwrap().0;
    FPackageId(lower_utf16_cityhash(asset_path))
}