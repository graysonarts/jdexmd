use color_eyre::eyre::Error;
use handlebars::{handlebars_helper, Handlebars};
use serde::{Deserialize, Serialize};

use crate::{
    jid::JohnnyId,
    model::{Area, Category, Folder, FolderKind, System, XFolder},
};

/// Handlebar template strings from the config file
#[derive(Debug, Serialize, Deserialize)]
pub struct MdFormatConfig {
    /// Handlebar template for systems
    system: String,
    /// Handlebar template for areas
    area: String,
    /// Handlebar template for categories
    category: String,
    /// Handlebar template for folders
    folder: String,
    /// Handlebar template for extended folders
    xfolder: String,
    /// Handlebar template for new markdown files
    markdown: String,
}

impl Default for MdFormatConfig {
    fn default() -> Self {
        Self {
            system: "# {{name}}".to_owned(),
            area: "## {{id}}.{{start id_range}}-{{end id_range}} {{topic}}".to_owned(),
            category: "- {{full_id id}} {{topic}}".to_owned(),
            folder: "  - {{#if (is_folder kind)}}{{full_id id}} {{topic}}{{else}}[[{{full_id id}} {{topic}}]]{{/if}}".to_owned(),
            xfolder: "    - {{#if (is_folder kind)}}{{full_id id}} {{topic}}{{else}}[[{{full_id id}} {{topic}}]]{{/if}}".to_owned(),
            markdown: "---
tags: [johnny-decimal, Librarian]
---".to_owned(),
        }
    }
}

/// A markdown formatter for Johnny Decimal
pub struct MdFormatter<'hbar> {
    /// The handlebars instance used to generate the markdown
    handlebars: Handlebars<'hbar>,
}

/// Bind the area to the system id
#[derive(Debug, Serialize)]
pub struct AreaWithParentId<'area> {
    #[serde(flatten)]
    /// The area to format
    area: &'area Area,
    /// The id of the containing system
    system_id: &'area str,
}

handlebars_helper!(full_id: |id: JohnnyId| id.by_seperator("."));
handlebars_helper!(start: |range: (u8, u8)| format!("{:02}", range.0));
handlebars_helper!(end: |range: (u8, u8)| format!("{:02}", range.1));
handlebars_helper!(is_folder: |kind: FolderKind| kind.is_folder());

impl<'hbar> MdFormatter<'hbar> {
    /// Create markdown for a System
    pub fn system(&self, system: &System) -> Result<String, Error> {
        let mut markdown = String::default();
        markdown.push_str(&self.handlebars.render("system", system)?);
        markdown.push('\n');
        for area in &system.areas {
            markdown.push_str(&self.area(&AreaWithParentId {
                area,
                system_id: &system.id.by_seperator("."),
            })?);
        }

        Ok(markdown)
    }

    /// Create markdown for an Area
    pub fn area(&self, area: &AreaWithParentId) -> Result<String, Error> {
        let mut markdown = String::default();
        markdown.push_str(&self.handlebars.render("area", area)?);
        markdown.push('\n');
        for category in &area.area.categories {
            markdown.push_str(&self.category(category)?);
        }

        Ok(markdown)
    }

    /// Create markdown for a Category
    pub fn category(&self, category: &Category) -> Result<String, Error> {
        let mut markdown = String::default();
        markdown.push_str(&self.handlebars.render("category", category)?);
        markdown.push('\n');
        for folder in &category.folders {
            markdown.push_str(&self.folder(folder)?);
        }

        Ok(markdown)
    }

    /// Create markdown for a Folder
    pub fn folder(&self, folder: &Folder) -> Result<String, Error> {
        let mut markdown = self.handlebars.render("folder", folder)?;
        markdown.push('\n');
        for xfolder in &folder.folders {
            markdown.push_str(&self.xfolder(xfolder)?);
        }
        Ok(markdown)
    }

    /// Create markdown for an Extended Folder
    pub fn xfolder(&self, folder: &XFolder) -> Result<String, Error> {
        let mut markdown = self.handlebars.render("xfolder", folder)?;
        markdown.push('\n');
        Ok(markdown)
    }

    pub fn markdown(&self) -> Result<String, Error> {
        let mut markdown = self.handlebars.render("md", &String::new())?;
        markdown.push('\n');
        Ok(markdown)
    }
}

impl<'hbar> TryFrom<MdFormatConfig> for MdFormatter<'hbar> {
    type Error = Error;

    fn try_from(config: MdFormatConfig) -> Result<Self, Self::Error> {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("full_id", Box::new(full_id));
        handlebars.register_helper("start", Box::new(start));
        handlebars.register_helper("end", Box::new(end));
        handlebars.register_helper("is_folder", Box::new(is_folder));
        let templates = vec![
            ("system", config.system),
            ("area", config.area),
            ("category", config.category),
            ("folder", config.folder),
            ("xfolder", config.xfolder),
            ("md", config.markdown),
        ];
        templates
            .into_iter()
            .map(|(name, template)| handlebars.register_template_string(name, template))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { handlebars })
    }
}
