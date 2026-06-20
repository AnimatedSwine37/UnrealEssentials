use std::{
    sync::Mutex
};
use std::io::{Read, Seek};
use std::ops::{Deref, DerefMut};
use std::sync::MutexGuard;
use retoc::container_header::StoreEntry;
use retoc::FPackageId;
use retoc::zen::{ExternalPackageDependency, FZenPackageSummary};
use utoc_lib::metadata::{UtocMetaImportType, UtocMetadata};
use utoc_lib::store::{LegacyImportIdResolver, MetadataProvider};

pub static UTOC_METADATA: MetadataState = MetadataState::new();

pub(crate) type MetadataInner = Mutex<Option<UtocMetadata>>;

#[derive(Debug)]
pub struct MetadataState(MetadataInner);

impl MetadataState {
    pub const fn new() -> Self {
        Self(Mutex::new(None))
    }

    pub(crate) fn instance() -> MutexGuard<'static, Option<UtocMetadata>> {
        let mut guard = UTOC_METADATA.lock().unwrap();
        if guard.is_none() {
            *guard = Some(UtocMetadata::default());
        }
        guard
    }
}

impl Deref for MetadataState {
    type Target = MetadataInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MetadataState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct MetadataAdapter;
impl MetadataProvider for MetadataAdapter {
    fn check_v2_import(&self, package_id: FPackageId) -> Option<StoreEntry> {
        MetadataState::instance().as_ref().unwrap()
            .get_manual_v2_import(package_id)
    }
    fn get_imports_ue4<T: Read + Seek>(
        &self,
        store_entry: &mut StoreEntry,
        reader: &mut T,
        package_id: FPackageId,
        package_header: &FZenPackageSummary,
        package_dependencies: &[ExternalPackageDependency],
    ) {
        let metadata = MetadataState::instance();
        store_entry.imported_packages = match metadata.as_ref().unwrap().get_import_type(package_id) {
            UtocMetaImportType::GraphPackageUnvalidated => LegacyImportIdResolver::from_graph_packages_unvalidated(&package_dependencies),
            UtocMetaImportType::GraphPackageValidated => LegacyImportIdResolver::from_graph_packages_validated(reader, &package_header, &package_dependencies),
            UtocMetaImportType::ManualV1 => LegacyImportIdResolver::from_metadata_v1(metadata.as_ref().unwrap(), package_id),
            UtocMetaImportType::ManualV2 => unreachable!()
        }
    }
}