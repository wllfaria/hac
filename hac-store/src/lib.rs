use std::cell::RefCell;

use collection::Collection;

pub mod collection;
pub mod collection_meta;
pub mod slab;

use collection_meta::CollectionMeta;

thread_local! {
    pub(crate) static HAC_STORE: RefCell<HacStore> = const { RefCell::new(HacStore::new()) };
}

#[derive(Debug)]
pub struct HacStore {
    collection: Option<Collection>,
    collections_meta: Vec<CollectionMeta>,
}

impl HacStore {
    pub const fn new() -> Self {
        Self {
            collection: None,
            collections_meta: vec![],
        }
    }
}

impl Default for HacStore {
    fn default() -> Self {
        Self::new()
    }
}
