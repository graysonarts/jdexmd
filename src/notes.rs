use std::{
    fmt::{Debug, Display},
    fs,
    path::PathBuf,
};

use color_eyre::eyre::{Error, OptionExt};

use crate::{
    config::OutputConfig,
    markdown::MdFormatter,
    model::{FolderKind, FullId, Styled, System},
};

#[derive(Debug)]
pub enum Action<'a> {
    CreateFile(PathBuf),
    CreateDirectory(PathBuf),
    WriteIndex(PathBuf, &'a System),
}

impl<'a> Action<'a> {
    pub fn execute(&self, formatter: &MdFormatter) -> Result<(), Error> {
        match self {
            Action::CreateFile(path) => {
                fs::create_dir_all(path.parent().ok_or_eyre("Unable to create parents")?)?;
                fs::write(
                    path,
                    r#"
---
tags:
  - librarian
---"#,
                )?;
            }
            Action::CreateDirectory(path) => {
                fs::create_dir_all(path)?;
            }
            Action::WriteIndex(path, system) => {
                let index = formatter.system(system)?;
                fs::write(path, index)?;
            }
        }

        Ok(())
    }
    pub fn dry_run(&self) -> String {
        if need_to_apply(self) {
            format!("Would {}", self)
        } else {
            format!("Would Skip {}", self)
        }
    }
}

impl<'a> Display for Action<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::CreateFile(p) => write!(f, "Create File {}", p.display()),
            Action::CreateDirectory(p) => write!(f, "Create Directory {}", p.display()),
            Action::WriteIndex(p, s) => write!(f, "Write Index {}", p.display()),
        }
    }
}

pub fn need_to_apply(action: &Action) -> bool {
    match action {
        Action::CreateFile(path) => !path.exists(),
        Action::CreateDirectory(path) => !path.exists(),
        Action::WriteIndex(_, _) => true,
    }
}

pub fn get_all_actions<'a>(output_config: &OutputConfig, system: &System) -> Vec<Action<'a>> {
    let mut actions = Vec::new();
    let base_path = output_config.expand(&output_config.base_folder).unwrap();
    for area in &system.areas {
        let area_path = base_path.join(&area.area_id.as_path());
        actions.push(Action::CreateDirectory(area_path));
        for category in &area.categories {
            let category_path = base_path.join(&category.category_id.as_path());
            actions.push(Action::CreateDirectory(category_path));
            for folder in &category.folders {
                get_actions_for_folder(output_config, category, folder)
                    .into_iter()
                    .for_each(|a| actions.push(a));
                for xfolder in &folder.folders {
                    get_actions_for_folder(output_config, folder, xfolder)
                        .into_iter()
                        .for_each(|a| actions.push(a));
                }
            }
        }
    }

    return actions;
}

fn get_actions_for_folder<'a, F: FullId + Styled, J: FullId + Debug>(
    config: &OutputConfig,
    parent: &J,
    f: &F,
) -> Vec<Action<'a>> {
    let base_path = config
        .expand(&config.base_folder)
        .unwrap()
        .join(parent.as_path());
    let mut actions = Vec::new();
    let name = f.id();

    match f.style() {
        FolderKind::Folder => actions.push(Action::CreateDirectory(base_path.join(&name))),
        FolderKind::File => actions.push(Action::CreateFile(base_path.join(format!("{name}.md")))),
        FolderKind::Both => {
            actions.push(Action::CreateDirectory(base_path.join(&name)));
            actions.push(Action::CreateFile(base_path.join(format!("{name}.md"))));
        }
    }
    actions
}
