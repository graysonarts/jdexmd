use std::path::PathBuf;

use color_eyre::eyre::Error;
use serde::{Deserialize, Serialize};

use crate::{jid::JohnnyId, line::parse_line, markdown::MdFormatConfig};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemConfig {
    pub system_id: String,
    pub base_folder: PathBuf,
    pub reference_folder: PathBuf,
    pub separator: Option<String>,
    pub name: String,
    pub config: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JohnnyDecimalConfig {
    #[serde(flatten)]
    pub system_config: SystemConfig,
    #[serde(default)]
    pub format: MdFormatConfig,
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

#[derive(Debug, Serialize)]
pub struct Area {
    pub area_range: (u8, u8),
    pub topic: String,
    pub categories: Vec<Category>,
}

#[derive(Debug, Serialize)]
pub struct Category {
    pub category_id: JohnnyId,
    pub topic: String,
    pub folders: Vec<Folder>,
}

#[derive(Debug, Serialize)]
pub enum FolderKind {
    Folder,
    File,
    Both,
}

#[derive(Debug, Serialize)]
pub struct Folder {
    pub folder_id: JohnnyId,
    pub topic: String,
    pub kind: FolderKind,
    pub folders: Vec<XFolder>,
}

#[derive(Debug, Serialize)]
pub struct XFolder {
    pub folder_id: JohnnyId,
    pub topic: String,
    pub kind: FolderKind,
}

impl JohnnyDecimalConfig {
    pub fn from_file(path: &std::path::PathBuf) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path)?;
        let config: JohnnyDecimalConfig = toml::from_str(&contents)?;
        Ok(config)
    }
}

impl TryFrom<SystemConfig> for System {
    type Error = Error;

    fn try_from(value: SystemConfig) -> Result<Self, Self::Error> {
        let id = JohnnyId::default().system_id(&value.system_id);
        let mut system = System::new(id, &value.name);
        let lines = value.config.lines();
        for (line_no, raw) in lines.enumerate() {
            let line = parse_line(line_no + 1, raw);
            if let Err(e) = line {
                eprintln!("Invalid Line: {}", raw.trim_start());
                return Err(e);
            }
            let line = line?;
            match line {
                crate::line::LineKind::Area(start, end, topic) => {
                    let area = Area {
                        area_range: (start, end),
                        topic: topic.to_string(),
                        categories: Vec::new(),
                    };
                    system.areas.push(area);
                }
                crate::line::LineKind::Category(id, topic) => {
                    let category_id = system.system_id.clone().category_id(id);
                    let category = Category {
                        category_id,
                        topic: topic.to_string(),
                        folders: Vec::new(),
                    };
                    system.areas.last_mut().and_then(|area| {
                        area.categories.push(category);
                        None::<()>
                    });
                }
                crate::line::LineKind::Folder(id, entry_style, topic) => {
                    system
                        .areas
                        .last_mut()
                        .and_then(|area| area.categories.last_mut())
                        .and_then(|category| {
                            let (kind, bare_topic) = match entry_style {
                                crate::line::EntryStyle::File => (FolderKind::File, &topic[1..]),
                                crate::line::EntryStyle::Folder => {
                                    (FolderKind::Folder, &topic[1..])
                                }
                                crate::line::EntryStyle::Default => (FolderKind::Both, &topic[0..]),
                            };
                            let folder_id = category.category_id.clone().folder_id(id);
                            let folder = Folder {
                                folder_id,
                                topic: bare_topic.to_string(),
                                kind,
                                folders: Vec::new(),
                            };
                            category.folders.push(folder);
                            None::<()>
                        });
                }
                crate::line::LineKind::ExtendedFolder(id, entry_style, topic) => {
                    system
                        .areas
                        .last_mut()
                        .and_then(|area| area.categories.last_mut())
                        .and_then(|category| category.folders.last_mut())
                        .and_then(|folder| {
                            let (kind, bare_topic) = match entry_style {
                                crate::line::EntryStyle::File => (FolderKind::File, &topic[1..]),
                                crate::line::EntryStyle::Folder => {
                                    (FolderKind::Folder, &topic[1..])
                                }
                                crate::line::EntryStyle::Default => (FolderKind::Both, &topic[0..]),
                            };
                            let folder_id = folder.folder_id.clone().xfolder_id(id);
                            let xfolder = XFolder {
                                folder_id,
                                topic: bare_topic.to_string(),
                                kind,
                            };
                            folder.folders.push(xfolder);
                            None::<()>
                        });
                }
            }
        }
        Ok(system)
    }
}
