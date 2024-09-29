use serde::{Deserialize, Serialize};

macro_rules! append_or_return {
    ($maybe_id:expr, $id:ident, $sep:expr) => {
        if let Some(id_piece) = $maybe_id {
            $id.push_str($sep);
            $id.push_str(&format!("{:02}", id_piece));
        } else {
            return $id;
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JohnnyId {
    pub system_id: Option<String>,
    pub category_id: Option<u8>,
    pub folder_id: Option<u8>,
    pub xfolder_id: Option<String>,
}

impl JohnnyId {
    pub fn system_id(self, system_id: &str) -> Self {
        Self {
            system_id: Some(system_id.to_string()),
            ..self
        }
    }

    pub fn category_id(self, category_id: u8) -> Self {
        Self {
            category_id: Some(category_id),
            ..self
        }
    }

    pub fn folder_id(self, folder_id: u8) -> Self {
        Self {
            folder_id: Some(folder_id),
            ..self
        }
    }

    pub fn xfolder_id(self, xfolder_id: &str) -> Self {
        Self {
            xfolder_id: Some(xfolder_id.to_string()),
            ..self
        }
    }

    pub fn by_seperator(&self, sep: &str) -> String {
        let mut id = String::new();
        if let Some(system_id) = &self.system_id {
            id.push_str(system_id);
        }

        append_or_return!(self.category_id, id, sep);
        append_or_return!(self.folder_id, id, sep);

        id
    }
}

impl Default for JohnnyId {
    fn default() -> Self {
        Self {
            system_id: None,
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
