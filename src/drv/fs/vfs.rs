use alloc::{
    collections::BTreeMap, format, string::{String, ToString}, vec::Vec
};

#[derive(Clone)]
pub enum NodeType {
    File,
    Directory,
}

pub struct VfsNode {
    pub inode: u64,
    pub kind: NodeType,
    pub name: String,
    pub data: Vec<u8>,
}

pub struct Vfs {
    next_inode: u64,
    nodes: BTreeMap<String, VfsNode>,
}

impl Vfs {
    pub fn new() -> Self {
        let mut nodes = BTreeMap::new();

        nodes.insert(
            "/".to_string(),
            VfsNode {
                inode: 0,
                kind: NodeType::Directory,
                name: "/".to_string(),
                data: Vec::new(),
            },
        );

        Self {
            next_inode: 1,
            nodes,
        }
    }

    pub fn create_file(&mut self, path: &str) -> bool {
        if self.nodes.contains_key(path) {
            return false;
        }

        self.nodes.insert(
            path.to_string(),
            VfsNode {
                inode: self.next_inode,
                kind: NodeType::File,
                name: path.to_string(),
                data: Vec::new(),
            },
        );

        self.next_inode += 1;

        true
    }

    pub fn create_dir(&mut self, path: &str) -> bool {
        if self.nodes.contains_key(path) {
            return false;
        }

        self.nodes.insert(
            path.to_string(),
            VfsNode {
                inode: self.next_inode,
                kind: NodeType::Directory,
                name: path.to_string(),
                data: Vec::new(),
            },
        );

        self.next_inode += 1;

        true
    }

    pub fn write_file(&mut self, path: &str, data: &[u8]) -> bool {
        match self.nodes.get_mut(path) {
            Some(node) => {
                if matches!(node.kind, NodeType::File) {
                    node.data.clear();
                    node.data.extend_from_slice(data);
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    pub fn append_file(&mut self, path: &str, data: &[u8]) -> bool {
        match self.nodes.get_mut(path) {
            Some(node) => {
                if matches!(node.kind, NodeType::File) {
                    node.data.extend_from_slice(data);
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    pub fn read_file(&self, path: &str) -> Option<&[u8]> {
        self.nodes.get(path).map(|n| n.data.as_slice())
    }

    pub fn exists(&self, path: &str) -> bool {
        self.nodes.contains_key(path)
    }

    pub fn remove(&mut self, path: &str) -> bool {
        self.nodes.remove(path).is_some()
    }

    pub fn list_dir(&self, path: &str) -> Vec<&str> {
        let prefix = if path.ends_with('/') {
            path.to_string()
        } else {
            format!("{}/", path)
        };

        self.nodes
            .keys()
            .filter(|p| p.starts_with(&prefix))
            .map(|p| p.as_str())
            .collect()
    }
}

pub fn init() -> Vfs {
    Vfs::new()
}