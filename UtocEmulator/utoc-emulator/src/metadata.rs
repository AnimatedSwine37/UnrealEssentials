use std::{
    sync::Mutex
};
use std::ops::{Deref, DerefMut};
use std::sync::MutexGuard;
use utoc_lib::metadata::UtocMetadata;

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