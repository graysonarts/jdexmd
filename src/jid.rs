use core::fmt::Formatter;
use core::fmt::Write as _;
use core::fmt::{Debug, Display, Result as FmtResult};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Internal Macro to either append a part of the id or return everything
/// that has been written so far
macro_rules! append_or_return {
    ($maybe_id:expr, $id:ident, $sep:expr) => {
        if let Some(id_piece) = $maybe_id {
            let _ = write!($id, "{}{:02}", $sep, id_piece);
        } else {
            return $id;
        }
    };
}

/// Binds a range of u8 ids to a topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundRangeId {
    /// The topic of the range
    pub topic: String,
    /// The start of the range
    pub start: u8,
    /// The end of the range
    pub end: u8,
}

/// Binds a u8 id to a topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundU8Id {
    /// The topic of the id
    pub topic: String,
    /// The id
    pub id: u8,
}

impl Display for BoundU8Id {
    #[expect(
        clippy::min_ident_chars,
        reason = "This is the preferred default name for the variable"
    )]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:02}", self.id)
    }
}

/// Binds a string id to a topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundStrId {
    /// The topic of the id
    pub topic: String,
    /// The id
    pub id: String,
}

impl Display for BoundStrId {
    #[expect(
        clippy::min_ident_chars,
        reason = "This is the preferred default name for the variable"
    )]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.id)
    }
}

/// Represents the level of the id, used for testing how deep the id is
#[repr(u8)]
#[derive(PartialEq, PartialOrd)]
enum Level {
    /// Nothing is set
    Empty = 0,
    /// The system is set
    System,
    /// The system and area are set
    Area,
    /// The system, area, and category are set
    Category,
    /// The system, area, category, and folder are set
    Folder,
    /// The system, area, category, folder, and extended folder are set
    ExtendedFolder,
}

/// Represents a Johnny Decimal id
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JohnnyId {
    /// The system id `L##`
    pub system: Option<String>,
    /// The area id `L##.##-##`
    pub area: Option<BoundRangeId>,
    /// The category id `L##.##` (notice that we don't include the area id here)
    pub category: Option<BoundU8Id>,
    /// The folder id `L##.##.##`
    pub folder: Option<BoundU8Id>,
    /// The extended folder id `L##.##.##.X##`
    pub xfolder: Option<BoundStrId>,
}

impl JohnnyId {
    /// Get the level of the id
    const fn level(&self) -> Level {
        if self.system.is_none() {
            return Level::Empty;
        }
        if self.area.is_none() {
            return Level::System;
        }
        if self.category.is_none() {
            return Level::Area;
        }
        if self.folder.is_none() {
            return Level::Category;
        }
        if self.xfolder.is_none() {
            return Level::Folder;
        }
        Level::ExtendedFolder
    }

    /// Set the system id
    pub fn system_id(self, system_id: &str) -> Self {
        Self {
            system: Some(system_id.to_owned()),
            ..self
        }
    }

    /// Set the area id
    pub fn area_id(self, start: u8, end: u8, topic: &str) -> Self {
        Self {
            area: Some(BoundRangeId {
                topic: topic.to_owned(),
                start,
                end,
            }),
            ..self
        }
    }

    /// Set the category id
    pub fn category_id(self, category_id: u8, topic: &str) -> Self {
        Self {
            category: Some(BoundU8Id {
                topic: topic.to_owned(),
                id: category_id,
            }),
            ..self
        }
    }

    /// Set the folder id
    pub fn folder_id(self, folder_id: u8, topic: &str) -> Self {
        Self {
            folder: Some(BoundU8Id {
                topic: topic.to_owned(),
                id: folder_id,
            }),
            ..self
        }
    }

    /// Set the extended folder id
    pub fn xfolder_id(self, xfolder_id: &str, topic: &str) -> Self {
        Self {
            xfolder: Some(BoundStrId {
                topic: topic.to_owned(),
                id: xfolder_id.to_owned(),
            }),
            ..self
        }
    }

    /// Get the id by a separator
    pub fn by_seperator(&self, sep: &str) -> String {
        let mut id = String::new();
        if let Some(system_id) = &self.system {
            id.push_str(system_id);
        }
        if self.level() <= Level::Area {
            if let Some(area_id) = &self.area {
                let _ = write!(id, "{}{:02}-{:02}", sep, area_id.start, area_id.end);
            }
        }
        append_or_return!(&self.category, id, sep);
        append_or_return!(&self.folder, id, sep);
        append_or_return!(&self.xfolder, id, sep);

        id
    }

    /// Get the id by a separator with the names included in the id parts
    pub fn by_seperator_bound(&self, sep: &str) -> String {
        let mut id: Vec<String> = Vec::new();
        let maybe_topic = self
            .xfolder
            .as_ref()
            .map(|bound_id| bound_id.topic.as_str())
            .or_else(|| self.folder.as_ref().map(|bound_id| bound_id.topic.as_str()))
            .or_else(|| {
                self.category
                    .as_ref()
                    .map(|bound_id| bound_id.topic.as_str())
            });
        if let Some(system_id) = &self.system {
            id.push(system_id.to_string());
        }
        if self.level() <= Level::Area {
            if let Some(area_id) = &self.area {
                id.push(format!(
                    "{:02}-{:02} {}",
                    area_id.start, area_id.end, area_id.topic
                ));
            }
        }
        if let Some(category_id) = &self.category {
            id.push(format!("{:02}", category_id.id));
        }
        if let Some(folder_id) = &self.folder {
            id.push(format!("{:02}", folder_id.id));
        }
        if let Some(xfolder_id) = &(self.xfolder) {
            id.push(xfolder_id.id.clone());
        }

        maybe_topic.map_or_else(
            || id.join(sep),
            |topic| format!("{} {}", id.join(sep), topic),
        )
    }

    /// Convert the id into a full path
    pub fn as_path(&self) -> PathBuf {
        let mut result = PathBuf::from(self.by_seperator_bound("."));
        let mut parent = self.parent();
        while parent
            .as_ref()
            .and_then(|current_parent| current_parent.system.as_deref())
            .is_some()
        {
            #[expect(clippy::expect_used, reason = "parent should always be set")]
            let parent_id = parent
                .as_ref()
                .expect("Parent should always be valid")
                .by_seperator_bound(".");
            result = PathBuf::from(parent_id).join(result);
            parent = parent.and_then(|new_parent| new_parent.parent());
        }

        result
    }

    /// Get the parent of the current id
    pub fn parent(&self) -> Option<Self> {
        self.system.as_ref()?;

        if self.area.is_none() {
            return Some(Self::default());
        }

        if self.category.is_none() {
            return Some(Self {
                area: None,
                ..self.to_owned()
            });
        }

        if self.folder.is_none() {
            return Some(Self {
                category: None,
                ..self.to_owned()
            });
        }

        if self.xfolder.is_none() {
            return Some(Self {
                folder: None,
                ..self.to_owned()
            });
        }

        Some(Self {
            xfolder: None,
            ..self.to_owned()
        })
    }
}

impl Display for JohnnyId {
    #[expect(
        clippy::min_ident_chars,
        reason = "This is the preferred default name for the variable"
    )]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.by_seperator("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_jid() {
        let id = JohnnyId::default()
            .system_id("system")
            .area_id(0, 1, "area")
            .category_id(1, "cat")
            .folder_id(2, "fold")
            .xfolder_id("xfolder", "xfold");

        assert_eq!(id.by_seperator("."), "system.01.02.xfolder");
        assert_eq!(id.by_seperator_bound("."), "system.01.02.xfolder xfold");
    }

    #[test]
    fn test_system_jid() {
        let id = JohnnyId::default().system_id("system");
        assert_eq!(id.by_seperator("."), "system");
    }

    #[test]
    fn test_area_jid() {
        let id = JohnnyId::default()
            .system_id("system")
            .area_id(0, 1, "area");
        assert_eq!(id.by_seperator("."), "system.00-01");
    }

    #[test]
    fn test_category_jid() {
        let id = JohnnyId::default()
            .system_id("system")
            .area_id(0, 1, "area")
            .category_id(1, "cat");
        assert_eq!(id.by_seperator("."), "system.01");
    }

    #[test]
    fn test_folder_jid() {
        let id = JohnnyId::default()
            .system_id("system")
            .area_id(0, 1, "area")
            .category_id(1, "cat")
            .folder_id(2, "fold");
        assert_eq!(id.by_seperator("."), "system.01.02");
    }

    #[test]
    fn test_parent_of_full() {
        let id = JohnnyId::default()
            .system_id("system")
            .area_id(0, 1, "area")
            .category_id(1, "cat")
            .folder_id(2, "fold")
            .xfolder_id("xfolder", "xfold");

        let parent = id.parent();
        assert_eq!(
            parent.map(|p| p.by_seperator(".")),
            Some("system.01.02".to_string())
        );
    }

    #[test]
    fn test_path_of_full() {
        let id = JohnnyId::default()
            .system_id("system")
            .area_id(0, 1, "area")
            .category_id(1, "cat")
            .folder_id(2, "fold")
            .xfolder_id("xfolder", "xfold");

        let path = id.as_path();
        assert_eq!(
            path.to_str().unwrap(),
            "system/system.00-01 area/system.01 cat/system.01.02 fold/system.01.02.xfolder xfold"
        );
    }

    #[test]
    fn test_path_system() {
        let id = JohnnyId::default().system_id("system");
        let path = id.as_path();
        assert_eq!(path.to_str().unwrap(), "system");
    }

    #[test]
    fn test_area_path() {
        let id = JohnnyId::default()
            .system_id("system")
            .area_id(0, 1, "area");

        let path = id.as_path();
        assert_eq!(path.to_str().unwrap(), "system/system.00-01 area");
    }

    #[test]
    fn test_path_category() {
        let id = JohnnyId::default()
            .system_id("system")
            .area_id(0, 1, "area")
            .category_id(1, "cat");
        let path = id.as_path();
        assert_eq!(
            path.to_str().unwrap(),
            "system/system.00-01 area/system.01 cat"
        );
    }

    #[test]
    fn test_path_folder() {
        let id = JohnnyId::default()
            .system_id("system")
            .area_id(0, 1, "area")
            .category_id(1, "cat")
            .folder_id(2, "fold");
        let path = id.as_path();
        assert_eq!(
            path.to_str().unwrap(),
            "system/system.00-01 area/system.01 cat/system.01.02 fold"
        );
    }
}
