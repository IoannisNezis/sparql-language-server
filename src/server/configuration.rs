use config::{Config, ConfigError};
use log::info;
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
            align_predicates: true,
            align_prefixes: false,
            separate_prolouge: false,
            capitalize_keywords: true,
            insert_spaces: None,
            tab_size: None,
            where_new_line: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub format: FormatSettings,
}

fn load_user_configuration() -> Result<Settings, ConfigError> {
    Config::builder()
        .add_source(config::File::with_name("qlue-ls"))
        .build()?
        .try_deserialize::<Settings>()
}

impl Settings {
    pub fn new() -> Self {
        match load_user_configuration() {
            Ok(settings) => {
                info!("Loaded user configuration\n{:?}", settings);
                settings
            }
            Err(error) => {
                info!(
                    "Did not load user-configuration:\n{}\n falling back to default values",
                    error
                );
                Settings::default()
            }
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            format: Default::default(),
        }
    }
}
