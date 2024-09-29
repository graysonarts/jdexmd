use std::path::PathBuf;
use std::{fmt::Debug, fmt::Display};

use serde::{Deserialize, Serialize};

macro_rules! append_or_return {
    ($maybe_id:expr, $id:ident, $sep:expr) => {
        if let Some(id_piece) = $maybe_id {
            $id.push_str(&format!("{}{:02}", $sep, id_piece));
        } else {
            return $id;
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundRangeId {
    pub topic: String,
    pub start: u8,
    pub end: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundU8Id {
    pub topic: String,
    pub id: u8,
}

impl Display for BoundU8Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}", self.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundStrId {
    pub topic: String,
    pub id: String,
}

impl Display for BoundStrId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[repr(u8)]
#[derive(PartialEq, PartialOrd)]
enum Level {
    Empty = 0,
    System,
    Area,
    Category,
    Folder,
    ExtendedFolder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JohnnyId {
    pub system_id: Option<String>,
    pub area_id: Option<BoundRangeId>,
    pub category_id: Option<BoundU8Id>,
    pub folder_id: Option<BoundU8Id>,
    pub xfolder_id: Option<BoundStrId>,
}

impl JohnnyId {
    fn level(&self) -> Level {
        if self.system_id.is_none() {
            return Level::Empty;
        }
        if self.area_id.is_none() {
            return Level::System;
        }
        if self.category_id.is_none() {
            return Level::Area;
        }
        if self.folder_id.is_none() {
            return Level::Category;
        }
        if self.xfolder_id.is_none() {
            return Level::Folder;
        }
        Level::ExtendedFolder
    }

    pub fn system_id(self, system_id: &str) -> Self {
        Self {
            system_id: Some(system_id.to_string()),
            ..self
        }
    }

    pub fn area_id(self, start: u8, end: u8, topic: &str) -> Self {
        Self {
            area_id: Some(BoundRangeId {
                topic: topic.to_string(),
                start,
                end,
            }),
            ..self
        }
    }

    pub fn category_id(self, category_id: u8, topic: &str) -> Self {
        Self {
            category_id: Some(BoundU8Id {
                topic: topic.to_string(),
                id: category_id,
            }),
            ..self
        }
    }

    pub fn folder_id(self, folder_id: u8, topic: &str) -> Self {
        Self {
            folder_id: Some(BoundU8Id {
                topic: topic.to_string(),
                id: folder_id,
            }),
            ..self
        }
    }

    pub fn xfolder_id(self, xfolder_id: &str, topic: &str) -> Self {
        Self {
            xfolder_id: Some(BoundStrId {
                topic: topic.to_string(),
                id: xfolder_id.to_string(),
            }),
            ..self
        }
    }

    pub fn by_seperator(&self, sep: &str) -> String {
        let mut id = String::new();
        if let Some(system_id) = &self.system_id {
            id.push_str(system_id);
        }
        if self.level() <= Level::Area {
            if let Some(area_id) = &self.area_id {
                id.push_str(&format!("{}{:02}-{:02}", sep, area_id.start, area_id.end));
            }
        }
        append_or_return!(&self.category_id, id, sep);
        append_or_return!(&self.folder_id, id, sep);
        append_or_return!(&self.xfolder_id, id, sep);

        id
    }

    pub fn by_seperator_bound(&self, sep: &str) -> String {
        let mut id: Vec<String> = Vec::new();
        let topic = self
            .xfolder_id
            .as_ref()
            .map(|id| id.topic.as_str())
            .or_else(|| {
                self.folder_id
                    .as_ref()
                    .and_then(|id| Some(id.topic.as_str()))
            })
            .or_else(|| {
                self.category_id
                    .as_ref()
                    .and_then(|id| Some(id.topic.as_str()))
            });
        if let Some(system_id) = &self.system_id {
            id.push(system_id.to_string());
        }
        if self.level() <= Level::Area {
            if let Some(area_id) = &self.area_id {
                id.push(format!(
                    "{:02}-{:02} {}",
                    area_id.start, area_id.end, area_id.topic
                ));
            }
        }
        if let Some(category_id) = &self.category_id {
            id.push(format!("{:02}", category_id.id));
        }
        if let Some(folder_id) = &self.folder_id {
            id.push(format!("{:02}", folder_id.id));
        }
        if let Some(xfolder_id) = &self.xfolder_id {
            id.push(xfolder_id.id.to_string());
        }

        match topic {
            Some(topic) => {
                format!("{} {}", id.join(sep), topic)
            }
            None => {
                format!("{}", id.join(sep))
            }
        }
    }

    pub fn as_path(&self) -> std::path::PathBuf {
        let mut result = PathBuf::from(self.by_seperator_bound("."));
        let mut parent = self.parent();
        while parent
            .as_ref()
            .and_then(|p| p.system_id.as_ref().and_then(|i| Some(i.as_str())))
            .is_some()
        {
            let parent_id = parent.as_ref().unwrap().by_seperator_bound(".");
            result = PathBuf::from(parent_id).join(result);
            parent = parent.and_then(|parent| parent.parent());
        }

        result
    }

    pub fn parent(&self) -> Option<Self> {
        if self.system_id.is_none() {
            return None;
        }

        if self.area_id.is_none() {
            return Some(Self::default());
        }

        if self.category_id.is_none() {
            return Some(Self {
                area_id: None,
                ..self.to_owned()
            });
        }

        if self.folder_id.is_none() {
            return Some(Self {
                category_id: None,
                ..self.to_owned()
            });
        }

        if self.xfolder_id.is_none() {
            return Some(Self {
                folder_id: None,
                ..self.to_owned()
            });
        }

        Some(Self {
            xfolder_id: None,
            ..self.to_owned()
        })
    }
}

impl Default for JohnnyId {
    fn default() -> Self {
        Self {
            system_id: None,
            area_id: None,
            category_id: None,
            folder_id: None,
            xfolder_id: None,
        }
    }
}

impl std::fmt::Display for JohnnyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
