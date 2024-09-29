use color_eyre::eyre::Error;
use handlebars::{handlebars_helper, Handlebars};
use serde::{Deserialize, Serialize};

use crate::{
    config::{Area, Category, Folder, System},
    jid::JohnnyId,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct MdFormatConfig {
    system: String,
    area: String,
    category: String,
    folder: String,
    xfolder: String,
}

impl Default for MdFormatConfig {
    fn default() -> Self {
        Self {
            system: "# {{name}}".to_string(),
            area: "## {{system_id}}.{{start area_range}}-{{end area_range}} {{topic}}".to_string(),
            category: "- {{full_id category_id}} {{topic}}".to_string(),
            folder: "  - [[{{full_id folder_id}} {{topic}}]] - {{kind}}".to_string(),
            xfolder: "    - [[{{full_id folder_id}} {{topic}}]]".to_string(),
        }
    }
}

pub struct MdFormatter<'a> {
    handlebars: Handlebars<'a>,
}

#[derive(Debug, Serialize)]
pub struct AreaWithParentId<'a> {
    #[serde(flatten)]
    area: &'a Area,
    system_id: &'a str,
}

handlebars_helper!(full_id: |id: JohnnyId| id.by_seperator("."));
handlebars_helper!(start: |range: (u8, u8)| format!("{:02}", range.0));
handlebars_helper!(end: |range: (u8, u8)| format!("{:02}", range.1));

impl<'a> MdFormatter<'a> {
    pub fn system(&self, system: &System) -> Result<String, Error> {
        let mut markdown = String::default();
        markdown.push_str(&self.handlebars.render("system", system)?);
        markdown.push('\n');
        for area in &system.areas {
            markdown.push_str(&self.area(&AreaWithParentId {
                area,
                system_id: &system.system_id.by_seperator("."),
            })?);
        }

        Ok(markdown)
    }

    pub fn area(&self, area: &AreaWithParentId) -> Result<String, Error> {
        let mut markdown = String::default();
        markdown.push_str(&self.handlebars.render("area", area)?);
        markdown.push('\n');
        for category in &area.area.categories {
            markdown.push_str(&self.category(category)?);
        }

        Ok(markdown)
    }

    pub fn category(&self, category: &Category) -> Result<String, Error> {
        let mut markdown = String::default();
        markdown.push_str(&self.handlebars.render("category", category)?);
        markdown.push('\n');
        for folder in &category.folders {
            markdown.push_str(&self.folder(folder)?);
        }

        Ok(markdown)
    }

    pub fn folder(&self, folder: &Folder) -> Result<String, Error> {
        let mut markdown = self.handlebars.render("folder", folder)?;
        markdown.push('\n');
        Ok(markdown)
    }

    pub fn xfolder(&self, folder: &Folder) -> Result<String, Error> {
        let mut markdown = self.handlebars.render("xfolder", folder)?;
        markdown.push('\n');
        Ok(markdown)
    }
}

impl<'a> TryFrom<MdFormatConfig> for MdFormatter<'a> {
    type Error = color_eyre::eyre::Error;

    fn try_from(config: MdFormatConfig) -> Result<Self, Self::Error> {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("full_id", Box::new(full_id));
        handlebars.register_helper("start", Box::new(start));
        handlebars.register_helper("end", Box::new(end));
        let templates = vec![
            ("system", config.system),
            ("area", config.area),
            ("category", config.category),
            ("folder", config.folder),
            ("folder", config.xfolder),
        ];
        templates
            .into_iter()
            .map(|(name, template)| handlebars.register_template_string(name, template))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { handlebars })
    }
}
