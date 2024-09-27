use std::path::PathBuf;
use std::time::SystemTime;

use chrono::{Datelike, Timelike};

use crate::HAC_STORE;

// TODO: i probably want to introduce some metadata storage, or caching to know
// total requests, total saved responses, and other infos without having to read
// the entire collection.
#[derive(Debug, Clone)]
pub struct CollectionMeta {
    name: String,
    path: std::path::PathBuf,
    size: u64,
    modified: CollectionModifiedMeta,
}

impl CollectionMeta {
    pub fn new(name: String, path: std::path::PathBuf, size: u64, modified: CollectionModifiedMeta) -> Self {
        Self {
            name,
            path,
            size,
            modified,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn path_mut(&mut self) -> &mut PathBuf {
        &mut self.path
    }

    pub fn modified(&self) -> &CollectionModifiedMeta {
        &self.modified
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionModifiedMeta {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minutes: u32,
    system_time: SystemTime,
}

impl CollectionModifiedMeta {
    pub fn new() -> Self {
        std::time::SystemTime::now().into()
    }
}

impl Default for CollectionModifiedMeta {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialOrd for CollectionModifiedMeta {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.system_time.cmp(&other.system_time))
    }
}

impl Ord for CollectionModifiedMeta {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.system_time.cmp(&other.system_time)
    }
}

impl std::fmt::Display for CollectionModifiedMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}-{} {}:{}",
            self.year, self.month, self.day, self.hour, self.minutes,
        )
    }
}

pub trait ReadableByteSize {
    fn readable_byte_size(&self) -> String;
}

fn readable_byte_size<N>(val: N) -> String
where
    N: Copy + Into<u64>,
{
    let units = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = val.into() as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < units.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2}{}", size, units[unit_index])
}

impl ReadableByteSize for u64 {
    fn readable_byte_size(&self) -> String {
        readable_byte_size(*self)
    }
}

impl From<SystemTime> for CollectionModifiedMeta {
    fn from(value: SystemTime) -> Self {
        let datetime = chrono::DateTime::<chrono::Utc>::from(value);
        let year = datetime.year();
        let month = datetime.day();
        let day = datetime.month();
        let hour = datetime.hour();
        let minutes = datetime.minute();

        CollectionModifiedMeta {
            year,
            month,
            day,
            hour,
            minutes,
            system_time: value,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CollectionMetaSorting {
    #[default]
    Recent,
    Name,
    Size,
}

impl CollectionMetaSorting {
    pub fn next(&self) -> Self {
        match self {
            CollectionMetaSorting::Recent => CollectionMetaSorting::Name,
            CollectionMetaSorting::Name => CollectionMetaSorting::Size,
            CollectionMetaSorting::Size => CollectionMetaSorting::Recent,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            CollectionMetaSorting::Recent => CollectionMetaSorting::Size,
            CollectionMetaSorting::Name => CollectionMetaSorting::Recent,
            CollectionMetaSorting::Size => CollectionMetaSorting::Name,
        }
    }
}

pub fn set_collections_meta(collections_meta: Vec<CollectionMeta>) {
    HAC_STORE.with_borrow_mut(|store| store.collections_meta = collections_meta);
}

pub fn sort_collection_meta(sorting_kind: CollectionMetaSorting) {
    HAC_STORE.with_borrow_mut(|store| match sorting_kind {
        CollectionMetaSorting::Name => store.collections_meta.sort_by(|a, b| a.name().cmp(b.name())),
        CollectionMetaSorting::Recent => store.collections_meta.sort_by(|a, b| b.modified().cmp(a.modified())),
        CollectionMetaSorting::Size => store.collections_meta.sort_by_key(|a| std::cmp::Reverse(a.size())),
    })
}

pub fn collections_meta<F>(f: F)
where
    F: FnOnce(&[CollectionMeta]),
{
    HAC_STORE.with_borrow(|store| f(&store.collections_meta))
}

pub fn get_collection_meta<F, R>(idx: usize, f: F) -> R
where
    F: FnOnce(&CollectionMeta) -> R,
{
    HAC_STORE.with_borrow(|store| f(&store.collections_meta[idx]))
}

pub fn get_collection_meta_mut<F, R>(idx: usize, f: F) -> R
where
    F: FnOnce(&mut CollectionMeta) -> R,
{
    HAC_STORE.with_borrow_mut(|store| f(&mut store.collections_meta[idx]))
}

pub fn push_collection_meta(collection_meta: CollectionMeta) {
    HAC_STORE.with_borrow_mut(|store| store.collections_meta.push(collection_meta))
}

pub fn remove_collection_meta<F, R>(file_name: String, f: F) -> R
where
    F: FnOnce(CollectionMeta) -> R,
{
    HAC_STORE.with_borrow_mut(|store| {
        let index = store
            .collections_meta
            .iter()
            .position(|meta| !meta.path().to_string_lossy().contains(&file_name))
            .expect("no collection metadata with file name");
        let collection_meta = store.collections_meta.remove(index);
        f(collection_meta)
    })
}
