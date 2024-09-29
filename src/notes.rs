/// Everything needed for generating the system for a notetaking system
use core::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::{fs, path::PathBuf};

use color_eyre::eyre::Error;
use color_eyre::eyre::OptionExt;
use expanduser::expanduser;

use crate::{
    markdown::MdFormatter,
    model::{FolderKind, FullId, HasFolderKind, System},
};

/// Expand the `~` into the home directory path
pub fn expand(path: &str) -> Result<PathBuf, Error> {
    Ok(expanduser(path)?)
}

/// Actions that can be taken to create the system
#[derive(Debug)]
pub enum Action<'sys> {
    /// Create a basic markdown file
    CreateFile(PathBuf),
    /// Create a directory
    CreateDirectory(PathBuf),
    /// Write the jdex index file
    WriteIndex(PathBuf, &'sys System),
}

impl<'sys> Action<'sys> {
    /// Execute the action by creating the file or directory, or writing the jdex
    pub fn execute(&self, formatter: &MdFormatter) -> Result<(), Error> {
        match self {
            Action::CreateFile(path) => {
                fs::create_dir_all(path.parent().ok_or_eyre("Unable to create parents")?)?;
                fs::write(
                    path,
                    "
---
tags:
  - librarian
---",
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

    /// Print out what would be done if the action was executed
    pub fn dry_run(&self) -> String {
        if need_to_apply(self) {
            format!("Would {self}\n")
        } else {
            String::new()
        }
    }
}

impl<'sys> Display for Action<'sys> {
    #[expect(
        clippy::min_ident_chars,
        reason = "This is the preferred default name for the variable"
    )]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Action::CreateFile(path) => write!(f, "Create File {}", path.display()),
            Action::CreateDirectory(path) => write!(f, "Create Directory {}", path.display()),
            Action::WriteIndex(path, _system) => write!(f, "Write Index {}", path.display()),
        }
    }
}

/// Check to see if the action needs to be applied to match the expected state of the system
pub fn need_to_apply(action: &Action) -> bool {
    match action {
        Action::CreateFile(path) | Action::CreateDirectory(path) => !path.exists(),
        Action::WriteIndex(_, _) => true,
    }
}

/// Get all of the actions for a system definition
pub fn get_all_actions<'sys>(base_folder: &str, system: &'sys System) -> Vec<Action<'sys>> {
    let mut actions = Vec::new();
    #[expect(clippy::expect_used, reason = "We are not expecting ~ to fail")]
    let base_path = expand(base_folder).expect("Cannot expand ~ in base folder");
    for area in &system.areas {
        let area_path = base_path.join(area.id.as_path());
        actions.push(Action::CreateDirectory(area_path));
        for category in &area.categories {
            let category_path = base_path.join(category.id.as_path());
            actions.push(Action::CreateDirectory(category_path));
            for folder in &category.folders {
                get_actions_for_folder(base_folder, system, category, folder)
                    .into_iter()
                    .for_each(|action| actions.push(action));
                for xfolder in &folder.folders {
                    get_actions_for_folder(base_folder, system, folder, xfolder)
                        .into_iter()
                        .for_each(|action| actions.push(action));
                }
            }
        }
    }

    actions
}

/// Gets the actions for a folder or xfolder (or really anything that has a `FolderKind`)
fn get_actions_for_folder<'sys, F: FullId + HasFolderKind, J: FullId + Debug>(
    base_folder: &str,
    root: &'sys System,
    parent: &J,
    folder: &F,
) -> Vec<Action<'sys>> {
    #[expect(clippy::expect_used, reason = "We are not expecting ~ to fail")]
    let base_path = expand(base_folder)
        .expect("Cannot expand ~ in base folder")
        .join(parent.as_path());
    let mut actions = Vec::new();
    let name = folder.id();

    match *folder.kind() {
        FolderKind::Folder => actions.push(Action::CreateDirectory(base_path.join(&name))),
        FolderKind::File => actions.push(Action::CreateFile(base_path.join(format!("{name}.md")))),
        FolderKind::Index => actions.push(Action::WriteIndex(
            base_path.join(format!("{name}.md")),
            root,
        )),
        FolderKind::Both => {
            actions.push(Action::CreateDirectory(base_path.join(&name)));
            actions.push(Action::CreateFile(base_path.join(format!("{name}.md"))));
        }
    }
    actions
}
