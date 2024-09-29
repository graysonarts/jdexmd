use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::jid::JohnnyId;

/// An item that has a Johnny Decimal id
pub trait HasJohnnyId {
    /// Returns the Johnny Decimal id of the item
    fn jid(&self) -> &JohnnyId;
    /// Returns the name of the item
    fn name(&self) -> &str;
}

/// An item that has a full id in the Johnny Decimal system
pub trait FullId: HasJohnnyId {
    /// Returns the id of the item
    fn id(&self) -> String {
        format!("{} {}", self.jid().by_seperator("."), self.name())
    }

    /// Returns the path of the item
    fn as_path(&self) -> PathBuf {
        self.jid().as_path()
    }
}

/// an item that has a `FolderKind` associated with it
pub trait HasFolderKind {
    /// Returns the `FolderKind` of the item
    fn kind(&self) -> &FolderKind;
}

/// Represents a system in the Johnny Decimal system
#[derive(Debug, Serialize)]
pub struct System {
    /// The id of the system
    pub id: JohnnyId,
    /// The name of the system
    pub name: String,
    /// The areas under this system
    pub areas: Vec<Area>,
}

impl System {
    /// Create a new system with the given id and name
    pub fn new(system_id: JohnnyId, name: &str) -> Self {
        Self {
            id: system_id,
            name: name.to_owned(),
            areas: Vec::new(),
        }
    }
}

impl FullId for System {}
impl HasJohnnyId for System {
    fn jid(&self) -> &JohnnyId {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
}

/// Represents an area in the Johnny Decimal system
#[derive(Debug, Serialize)]
pub struct Area {
    /// The id of the area
    pub id: JohnnyId,
    /// The range of ids in the area
    pub id_range: (u8, u8),
    /// The Title of the area
    pub topic: String,
    /// The categories under this area
    pub categories: Vec<Category>,
}

impl FullId for Area {}
impl HasJohnnyId for Area {
    fn jid(&self) -> &JohnnyId {
        &self.id
    }
    fn name(&self) -> &str {
        &self.topic
    }
}

/// Represents a category in the Johnny Decimal system
#[derive(Debug, Serialize)]
pub struct Category {
    /// The id of the category
    pub id: JohnnyId,
    /// The Title of the category
    pub topic: String,
    /// The folders under this category
    pub folders: Vec<Folder>,
}

impl FullId for Category {}
impl HasJohnnyId for Category {
    fn jid(&self) -> &JohnnyId {
        &self.id
    }
    fn name(&self) -> &str {
        &self.topic
    }
}

/// The kind of "folder" we are dealing with. "folder" is a concept in the Johnny Decimal system
/// and does not always represent an actual folder on your file system.
#[derive(Debug, Serialize, Deserialize, Default)]
pub enum FolderKind {
    #[default]
    /// This is just the directory
    Folder,
    /// This is just the file in the parent directory
    File,
    /// This is both the directory and a file in the parent directory
    Both,
    /// This is a jdex file
    Index,
}

impl FolderKind {
    /// Create a `FolderKind` from a directive character
    pub const fn from_char(directive: char) -> Self {
        match directive {
            '-' => Self::File,
            '+' => Self::Both,
            '!' => Self::Index,
            _ => Self::Folder,
        }
    }

    /// Helper function for the markdown formatter
    pub const fn is_folder(&self) -> bool {
        matches!(self, &Self::Folder)
    }
}

/// This is a "folder" which should have an ID in the form "##" where ## is a number
#[derive(Debug, Serialize)]
pub struct Folder {
    /// The id of the "folder"
    pub id: JohnnyId,
    /// The Title of the "folder"
    pub topic: String,
    /// The kind of "folder"
    pub kind: FolderKind,
    /// The extended folders under this folder. Normally empty
    pub folders: Vec<XFolder>,
}

impl FullId for Folder {}
impl HasJohnnyId for Folder {
    fn jid(&self) -> &JohnnyId {
        &self.id
    }
    fn name(&self) -> &str {
        &self.topic
    }
}

impl HasFolderKind for Folder {
    fn kind(&self) -> &FolderKind {
        &self.kind
    }
}

/// This is an "extended folder" which should have an ID in the form "X##" where ## is a number
#[derive(Debug, Serialize)]
pub struct XFolder {
    /// The id of the "folder"
    pub id: JohnnyId,
    /// The Title of the "folder"
    pub topic: String,
    /// The kind of "folder"
    pub kind: FolderKind,
}

impl FullId for XFolder {}
impl HasJohnnyId for XFolder {
    fn jid(&self) -> &JohnnyId {
        &self.id
    }
    fn name(&self) -> &str {
        &self.topic
    }
}

impl HasFolderKind for XFolder {
    fn kind(&self) -> &FolderKind {
        &self.kind
    }
}
