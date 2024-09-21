mod collection_list;
mod create_collection;
use std::sync::mpsc::Sender;

use collection_list::CollectionList;
use create_collection::CreateCollection;
use hac_core::command::Command;
use hac_loader::collection_loader::CollectionMeta;
use ratatui::layout::Rect;

use crate::router::Router;
use crate::{HacColors, HacConfig};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Default)]
pub enum Routes {
    #[default]
    ListCollections,
    CreateCollection,
}

impl Into<u8> for Routes {
    fn into(self) -> u8 {
        match self {
            Self::ListCollections => 0,
            Self::CreateCollection => 1,
        }
    }
}

pub fn make_collection_list_router(
    command_sender: Sender<Command>,
    collections: Vec<CollectionMeta>,
    size: Rect,
    config: HacConfig,
    colors: HacColors,
) -> Router {
    let mut router = Router::new(command_sender, colors.clone());
    let collection_list = CollectionList::new(collections, size, config.clone(), colors.clone());
    let create_collection = CreateCollection::new(size, config.clone(), colors.clone());

    router.add_route(Routes::ListCollections.into(), Box::new(collection_list));
    router.add_dialog(Routes::CreateCollection.into(), Box::new(create_collection));
    router
}
