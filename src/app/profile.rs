//! Manage access to profile templates stored in a file.

use crate::app::subcommand;
use crate::{debug, err, util};
use std::io::Write;
use std::{collections, fmt, fs, io, path};

lazy_static::lazy_static! {
    /// The path to the AWS CLI profiles configuration file.
    static ref AWS_FILE: path::PathBuf = util::AWS_CONFIG_DIR.join("config");

    /// The path to the file containing the profile templates.
    static ref TEMPLATES_FILE: path::PathBuf = util::CONFIG_DIR.join("templates.json");
}

/// A processed profile template ready to be installed.
pub struct Profile {
    /// The enabled state of the profile.
    enabled: bool,

    /// The name of the profile.
    name: String,

    /// The profile configuration settings.
    settings: collections::HashMap<String, String>,
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Profile {
    /// Returns the enabled state of the profile.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the name of the profile.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// A specialized HashMap type for a named collection of profile templates.
type Profiles = collections::HashMap<String, Profile>;

/// An unprocessed profile template as read from the file.
#[derive(serde::Deserialize)]
pub struct Template {
    /// The enabled state of the profile.
    #[serde(default = "Template::enabled_default")]
    enabled: bool,

    /// The name of another profile template to extend.
    extends: Option<String>,

    /// The profile configuration settings.
    settings: collections::HashMap<String, serde_json::Value>,
}

impl Template {
    /// Returns the default enabled state of a profile.
    fn enabled_default() -> bool {
        true
    }
}

/// A specialized HashMap type for a named collection of profile templates.
type Templates = collections::HashMap<String, Template>;

/// Checks if an AWS CLI profile exists.
pub fn exists_in_cli(context: &impl subcommand::Context, name: &str) -> subcommand::Result<bool> {
    let output = util::aws(context)
        .arg("configure")
        .arg("list-profiles")
        .output()?;

    let profiles = output.split_whitespace().collect::<Vec<&str>>();

    Ok(profiles.contains(&name))
}

/// Installs a profile into the AWS CLI.
pub fn install_in_cli(profile: &Profile) -> subcommand::Result<()> {
    let mut rendered = String::new();

    rendered.push('[');

    if profile.name == "default" {
        rendered.push_str("default");
    } else {
        rendered.push_str("profile ");
        rendered.push_str(&profile.name);
    }

    rendered.push(']');
    rendered.push('\n');

    for (key, value) in &profile.settings {
        rendered.push_str(key);
        rendered.push_str(" = ");
        rendered.push_str(value);
        rendered.push('\n');
    }

    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(AWS_FILE.as_path())?;

    writeln!(file, "{}", rendered)?;

    Ok(())
}

/// Processes the raw profile templates into ready-to-install AWS CLI profiles.
fn process_templates(templates: &Templates) -> subcommand::Result<Profiles> {
    let mut profiles = Profiles::new();

    for (name, template) in templates {
        let mut profile = Profile {
            enabled: template.enabled,
            name: name.to_owned(),
            settings: collections::HashMap::new(),
        };

        let mut extends = &template.extends;
        let mut visited: Vec<&String> = Vec::new();

        while let Some(extends_name) = extends {
            if visited.contains(&extends_name) {
                err!(1, "");
            }

            visited.push(extends_name);

            let template = match templates.get(extends_name) {
                Some(template) => template,
                None => err!(1, "The profile template, {}, does not exist.", extends_name),
            };

            for (key, value) in &template.settings {
                if !profile.settings.contains_key(key) {
                    use serde_json::Value;

                    let key = key.to_owned();
                    let value = match value {
                        Value::Bool(value) => value.to_string(),
                        Value::Null => "".to_owned(),
                        Value::Number(value) => value.to_string(),
                        Value::String(value) => value.to_owned(),
                        _ => {
                            err!(
                                1,
                                "The JSON encoded values of array or object type are not supported."
                            )
                        }
                    };

                    profile.settings.insert(key, value);
                }
            }

            extends = &template.extends;
        }

        profiles.insert(name.to_owned(), profile);
    }

    Ok(profiles)
}

/// Reads and processes the templates from a file into ready-to-install AWS CLI profiles.
pub fn read_templates() -> subcommand::Result<Profiles> {
    debug!("Reading profiles from: {}", TEMPLATES_FILE.display());

    let file = fs::File::open(TEMPLATES_FILE.as_path())?;
    let reader = io::BufReader::new(file);

    let templates = match serde_json::from_reader(reader) {
        Ok(templates) => templates,
        Err(error) => err!(1, format!("{}", error)),
    };

    let profiles = process_templates(&templates)?;

    Ok(profiles)
}
