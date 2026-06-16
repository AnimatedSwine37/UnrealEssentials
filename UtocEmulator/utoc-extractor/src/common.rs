use std::fmt::{Display, Formatter};
use clap::ValueEnum;

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