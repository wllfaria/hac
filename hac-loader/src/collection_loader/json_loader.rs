use hac_store::collection::{
    BodyKind, Collection, CollectionInfo, Folder, HeaderEntry, ReqMethod, ReqTree, ReqTreeNode, Request, WhichSlab,
};
use hac_store::slab::{Key, Slab};

use super::json_collection::{
    JsonBodyKind, JsonCollection, JsonCollectionInfo, JsonFolder, JsonHeaderEntry, JsonReqMethod, JsonRequest, ReqKind,
};
use super::{IntoCollection, Result};

pub struct JsonLoader(JsonCollection);

impl IntoCollection for JsonLoader {
    fn into_collection(self) -> hac_store::collection::Collection {
        from_file_to_collection(self.0)
    }
}

impl JsonLoader {
    pub fn parse(content: &str) -> Result<Self> {
        match serde_json::from_str(content) {
            Ok(collection) => Ok(Self(collection)),
            Err(e) => todo!("{e:?}"),
        }
    }
}

fn from_file_to_collection(file_collection: JsonCollection) -> Collection {
    let mut folders = Slab::<Folder>::new();
    let mut requests = Slab::<Request>::new();
    let mut root_requests = Slab::<Request>::new();

    for item in file_collection.requests {
        match item {
            ReqKind::Req(inner) => _ = root_requests.push(inner.into()),
            ReqKind::Folder(inner) => _ = collect_folder(inner, &mut requests, &mut folders),
        };
    }

    let selected_request = match (root_requests.is_empty(), folders.is_empty()) {
        (true, true) => None,
        (false, _) => Some((WhichSlab::RootRequests, 0)),
        (true, false) => {
            let folder = folders.get(0);
            match folder.requests.is_empty() {
                true => Some((WhichSlab::Folders, 0)),
                false => Some((WhichSlab::Requests, folder.requests[0])),
            }
        }
    };

    let mut nodes: Vec<ReqTreeNode> = (0..root_requests.len()).map(ReqTreeNode::Req).collect();
    nodes.extend(
        folders
            .iter()
            .enumerate()
            .map(|(idx, folder)| ReqTreeNode::Folder(idx, folder.requests.clone()))
            .collect::<Vec<_>>(),
    );

    let layout = ReqTree { nodes };

    Collection {
        info: file_collection.info.into(),
        folders,
        requests,
        root_requests,
        layout,
        hovered_request: selected_request,
        selected_request,
    }
}

fn collect_folder(file_folder: JsonFolder, requests: &mut Slab<Request>, folders: &mut Slab<Folder>) -> Key {
    // inserting a folder to be modified later to ensure we reserve an idx
    let idx = folders.push(Folder::default());
    let mut childrens = vec![];

    for item in file_folder.requests {
        let mut req: Request = item.into();
        req.parent = Some(idx);
        let req_idx = requests.push(req);
        childrens.push(req_idx);
    }

    let folder = folders.get_mut(idx);
    folder.name = file_folder.name;
    folder.requests = childrens;
    idx
}

impl From<JsonBodyKind> for BodyKind {
    fn from(body_kind: JsonBodyKind) -> Self {
        match body_kind {
            JsonBodyKind::Json => Self::Json,
            JsonBodyKind::NoBody => Self::NoBody,
        }
    }
}

impl From<JsonHeaderEntry> for HeaderEntry {
    fn from(entry: JsonHeaderEntry) -> Self {
        Self {
            key: entry.key,
            val: entry.val,
            enabled: entry.enabled,
        }
    }
}

impl From<JsonReqMethod> for ReqMethod {
    fn from(file_req: JsonReqMethod) -> Self {
        match file_req {
            JsonReqMethod::Get => Self::Get,
            JsonReqMethod::Post => Self::Post,
            JsonReqMethod::Put => Self::Put,
            JsonReqMethod::Patch => Self::Patch,
            JsonReqMethod::Delete => Self::Delete,
        }
    }
}

impl From<JsonRequest> for Request {
    fn from(file_req: JsonRequest) -> Self {
        Self {
            parent: None,
            body: file_req.body,
            name: file_req.name,
            uri: file_req.uri,
            body_kind: file_req.body_kind.into(),
            method: file_req.method.into(),
            headers: file_req.headers.into_iter().map(Into::into).collect::<Vec<_>>(),
        }
    }
}

impl From<JsonCollectionInfo> for CollectionInfo {
    fn from(collection_info: JsonCollectionInfo) -> Self {
        Self {
            name: collection_info.name,
            description: collection_info.description,
        }
    }
}

impl From<JsonFolder> for Folder {
    fn from(file_folder: JsonFolder) -> Self {
        Self {
            name: file_folder.name,
            requests: vec![],
            collapsed: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // simple collection with just a few entries
    const BASE_COLLECTION: &str = include_str!("../../test_collections/basic.json");
    // collection with complex structure of directories and requests
    const FULL_COLLECTION: &str = include_str!("../../test_collections/complex.json");

    #[test]
    fn load_from_json_to_inner_collection() {
        let result = JsonLoader::parse(BASE_COLLECTION).unwrap();
        let collection = result.into_collection();

        assert!(collection.requests.len() == 1);
        assert!(collection.folders.len() == 1);
        assert!(collection.root_requests.len() == 1);
        assert_eq!(collection.root_requests.get(0).name, "Sample Root Request");
        assert_eq!(collection.requests.get(0).name, "Sample Parented Request");
        assert_eq!(collection.folders.get(0).name, "Sample Folder");

        let result = JsonLoader::parse(FULL_COLLECTION).unwrap();
        let collection = result.into_collection();

        assert!(collection.requests.len() == 4);
        assert!(collection.folders.len() == 3);
        assert!(collection.root_requests.len() == 1);
        assert_eq!(collection.root_requests.get(0).name, "Root Request 1");
        assert_eq!(collection.requests.get(0).name, "Parented Request 1 Folder 1 - 1");
        assert_eq!(collection.requests.get(1).name, "Parented Request 1 Folder 1 - 2");
        assert_eq!(collection.requests.get(2).name, "Parented Request 2 Folder 1 - 1");
        assert_eq!(collection.requests.get(3).name, "Parented Request 1 Folder 1");
        assert_eq!(collection.folders.get(0).name, "Folder 1");
        assert_eq!(collection.folders.get(1).name, "Folder 1 - 1");
        assert_eq!(collection.folders.get(2).name, "Folder 1 - 2");
    }
}
