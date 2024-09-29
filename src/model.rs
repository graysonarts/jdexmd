use std::path::PathBuf;

use serde::Serialize;

use crate::jid::JohnnyId;

pub trait HasJohnnyId {
    fn jid(&self) -> &JohnnyId;
    fn name(&self) -> &str;
}

pub trait FullId: HasJohnnyId {
    fn id(&self) -> String {
        format!("{} {}", self.jid().by_seperator("."), self.name())
    }
    fn as_path(&self) -> PathBuf {
        self.jid().as_path()
    }
}

pub trait Styled {
    fn style(&self) -> &FolderKind;
}

#[derive(Debug, Serialize)]
pub struct System {
    pub system_id: JohnnyId,
    pub name: String,
    pub areas: Vec<Area>,
}

impl System {
    pub fn new(system_id: JohnnyId, name: &str) -> Self {
        Self {
            system_id,
            name: name.to_string(),
            areas: Vec::new(),
        }
    }
}

impl FullId for System {}
impl HasJohnnyId for System {
    fn jid(&self) -> &JohnnyId {
        &self.system_id
    }
    fn name(&self) -> &str {
        &self.name
    }
}
#[derive(Debug, Serialize)]
pub struct Area {
    pub area_id: JohnnyId,
    pub area_range: (u8, u8),
    pub topic: String,
    pub categories: Vec<Category>,
}

impl FullId for Area {}
impl HasJohnnyId for Area {
    fn jid(&self) -> &JohnnyId {
        &self.area_id
    }
    fn name(&self) -> &str {
        &self.topic
    }
}

#[derive(Debug, Serialize)]
pub struct Category {
    pub category_id: JohnnyId,
    pub topic: String,
    pub folders: Vec<Folder>,
}

impl FullId for Category {}
impl HasJohnnyId for Category {
    fn jid(&self) -> &JohnnyId {
        &self.category_id
    }
    fn name(&self) -> &str {
        &self.topic
    }
}

#[derive(Debug, Serialize, Default)]
pub enum FolderKind {
    #[default]
    Folder,
    File,
    Both,
    Index,
}

impl FolderKind {
    pub fn from_char(c: char) -> Self {
        match c {
            '-' => FolderKind::File,
            '+' => FolderKind::Both,
            '!' => FolderKind::Index,
            _ => FolderKind::Folder,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Folder {
    pub folder_id: JohnnyId,
    pub topic: String,
    pub kind: FolderKind,
    pub folders: Vec<XFolder>,
}

impl FullId for Folder {}
impl HasJohnnyId for Folder {
    fn jid(&self) -> &JohnnyId {
        &self.folder_id
    }
    fn name(&self) -> &str {
        &self.topic
    }
}

impl Styled for Folder {
    fn style(&self) -> &FolderKind {
        &self.kind
    }
}

#[derive(Debug, Serialize)]
pub struct XFolder {
    pub folder_id: JohnnyId,
    pub topic: String,
    pub kind: FolderKind,
}

impl FullId for XFolder {}
impl HasJohnnyId for XFolder {
    fn jid(&self) -> &JohnnyId {
        &self.folder_id
    }
    fn name(&self) -> &str {
        &self.topic
    }
}

impl Styled for XFolder {
    fn style(&self) -> &FolderKind {
        &self.kind
    }
}
