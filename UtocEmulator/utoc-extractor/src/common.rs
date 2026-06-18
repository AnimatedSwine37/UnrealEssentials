use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use clap::ValueEnum;
use retoc::Toc;

#[derive(ValueEnum, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum AssetMetadata {
    None,
    Table,
    PerAsset
}

impl Display for AssetMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::None => "None",
            Self::PerAsset => "Per Asset",
            Self::Table => "Table",
        })
    }
}

pub(crate) fn get_root_path<P: AsRef<Path>>(output: P, mount_point: &str, toc: &Toc, root_folder: &str) -> PathBuf {
    if mount_point.len() > utoc_lib::assets::MOUNT_POINT.len() {
        output.as_ref().join(&mount_point[utoc_lib::assets::MOUNT_POINT.len()..])
    } else {
        // Determine if we need to make the /Game/Content folders ourselves:
        let mut make_folders = true;
        for (file, _) in &toc.file_map {
            if let Some(s) = file.split_once('/') && s.0 == "Engine" {
                make_folders = false;
                break;
            }
        }
        match make_folders {
            true => output.as_ref().join(root_folder).join("Content"),
            false => output.as_ref().to_owned()
        }
    }
}