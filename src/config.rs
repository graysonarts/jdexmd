use std::{fs::read_to_string, path::PathBuf};

use color_eyre::eyre::Error;
use serde::{Deserialize, Serialize};

use crate::{
    jid::JohnnyId,
    line::{parse_single, ParsedKind},
    markdown::MdFormatConfig,
    model::{Area, Category, Folder, FolderKind, System, XFolder},
};

/// The configuration for the Johnny Decimal system
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemParameters {
    /// The system id for the Johnny Decimal system
    pub system_id: String,
    /// The default separator for the system
    pub separator: Option<String>,
    /// The name of the system
    pub name: String,
    /// The configuration definition for the system
    pub config: String,
}

/// The output configuration for the Johnny Decimal system
#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    /// The folder where your note taking system wants the system
    pub base_folder: String,
    /// The folder where the reference archive should be created
    pub reference_folder: String,
}

/// The configuration for the Johnny Decimal system
#[derive(Debug, Serialize, Deserialize)]
pub struct JohnnyDecimal {
    /// Configuring the system
    #[serde(flatten)]
    pub system_config: SystemParameters,
    /// Where we are outputting files
    #[serde(flatten)]
    pub output_config: Output,
    /// The handlebar themes for the markdown output
    #[serde(default)]
    pub format: MdFormatConfig,
}

impl JohnnyDecimal {
    /// Load the configuration from a TOML file
    pub fn from_file(path: &PathBuf) -> Result<Self, Error> {
        let contents = read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }
}

impl TryFrom<SystemParameters> for System {
    type Error = Error;

    #[expect(
        clippy::arithmetic_side_effects,
        reason = "The likelihood of overflow is low"
    )]
    fn try_from(value: SystemParameters) -> Result<Self, Self::Error> {
        let system_id = JohnnyId::default().system_id(&value.system_id);
        let mut system = Self::new(system_id, &value.name);
        let lines = value.config.lines();
        for (line_no, raw) in lines.enumerate() {
            let single_line = parse_single(line_no + 1, raw);
            if let Err(err) = single_line {
                eprintln!("Invalid Line: {}", raw.trim_start());
                return Err(err);
            }
            let line = single_line?;
            match line {
                ParsedKind::Area(start, end, topic) => {
                    let area_id = system.id.clone().area_id(start, end, topic);
                    let area = Area {
                        id: area_id,
                        id_range: (start, end),
                        topic: topic.to_owned(),
                        categories: Vec::new(),
                    };
                    system.areas.push(area);
                }
                ParsedKind::Category(id, topic) => {
                    system.areas.last_mut().and_then(|area| {
                        let category_id = area.id.clone().category_id(id, topic);
                        let category = Category {
                            id: category_id,
                            topic: topic.to_owned(),
                            folders: Vec::new(),
                        };
                        area.categories.push(category);
                        None::<()>
                    });
                }
                ParsedKind::Folder(id, entry_style, topic) => {
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
                            let folder_id = category.id.clone().folder_id(id, bare_topic);
                            let folder = Folder {
                                id: folder_id,
                                topic: bare_topic.to_owned(),
                                kind,
                                folders: Vec::new(),
                            };
                            category.folders.push(folder);
                            None::<()>
                        });
                }
                ParsedKind::ExtendedFolder(id, entry_style, topic) => {
                    system
                        .areas
                        .last_mut()
                        .and_then(|area| area.categories.last_mut())
                        .and_then(|category| category.folders.last_mut())
                        .and_then(|folder| {
                            let (kind, bare_topic) = match entry_style {
                                FolderKind::File => (FolderKind::File, &topic[1..]),
                                FolderKind::Both | FolderKind::Index => {
                                    (FolderKind::Folder, &topic[1..])
                                }
                                FolderKind::Folder => (FolderKind::Folder, &topic[0..]),
                            };
                            let folder_id = folder.id.clone().xfolder_id(id, bare_topic);
                            let xfolder = XFolder {
                                id: folder_id,
                                topic: bare_topic.to_owned(),
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
