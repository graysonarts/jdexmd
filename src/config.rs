use std::path::PathBuf;

use color_eyre::eyre::Error;
use expanduser::expanduser;
use serde::{Deserialize, Serialize};

use crate::{jid::JohnnyId, line::parse_line, markdown::MdFormatConfig, model::*};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemConfig {
    pub system_id: String,
    pub separator: Option<String>,
    pub name: String,
    pub config: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputConfig {
    pub base_folder: String,
    pub reference_folder: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JohnnyDecimalConfig {
    #[serde(flatten)]
    pub system_config: SystemConfig,
    #[serde(flatten)]
    pub output_config: OutputConfig,
    #[serde(default)]
    pub format: MdFormatConfig,
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
                    let area_id = system.system_id.clone().area_id(start, end, topic);
                    let area = Area {
                        area_id,
                        area_range: (start, end),
                        topic: topic.to_string(),
                        categories: Vec::new(),
                    };
                    system.areas.push(area);
                }
                crate::line::LineKind::Category(id, topic) => {
                    system.areas.last_mut().and_then(|area| {
                        let category_id = area.area_id.clone().category_id(id, topic);
                        let category = Category {
                            category_id,
                            topic: topic.to_string(),
                            folders: Vec::new(),
                        };
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
                                FolderKind::File => (FolderKind::File, &topic[1..]),
                                FolderKind::Both => (FolderKind::Both, &topic[1..]),
                                FolderKind::Index => (FolderKind::Index, &topic[1..]),
                                FolderKind::Folder => (FolderKind::Folder, &topic[0..]),
                            };
                            let folder_id = category.category_id.clone().folder_id(id, bare_topic);
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
                                FolderKind::File => (FolderKind::File, &topic[1..]),
                                FolderKind::Both => (FolderKind::Folder, &topic[1..]),
                                FolderKind::Index => (FolderKind::Folder, &topic[1..]),
                                FolderKind::Folder => (FolderKind::Folder, &topic[0..]),
                            };
                            let folder_id = folder.folder_id.clone().xfolder_id(id, bare_topic);
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
