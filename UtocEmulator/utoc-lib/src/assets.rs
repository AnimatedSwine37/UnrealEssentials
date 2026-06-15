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