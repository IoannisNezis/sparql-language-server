use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct FormatSettings {
    pub align_predicates: bool,
    pub align_prefixes: bool,
    pub separate_prolouge: bool,
    pub capitalize_keywords: bool,
    pub insert_spaces: Option<bool>,
    pub tab_size: Option<u8>,
    pub where_new_line: bool,
}

impl Default for FormatSettings {
    fn default() -> Self {
        Self {
            align_prefixes: false,
            align_predicates: true,
            separate_prolouge: false,
            capitalize_keywords: true,
            insert_spaces: None,
            tab_size: None,
            where_new_line: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub format: FormatSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            format: Default::default(),
        }
    }
}
