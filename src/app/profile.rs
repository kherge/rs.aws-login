//! Manage access to profile templates stored in a file.

use crate::app::subcommand;
use crate::{debug, err, util};
use std::io::Write;
use std::{collections, fmt, fs, io, path};

lazy_static::lazy_static! {
    /// The path to the AWS CLI profiles configuration file.
    static ref AWS_FILE: path::PathBuf = util::AWS_CONFIG_DIR.join("config");

    /// The path to the file containing the profile templates.
    pub static ref TEMPLATES_FILE: path::PathBuf = util::CONFIG_DIR.join("templates.json");
}

/// A processed profile template ready to be installed.
#[derive(Eq)]
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

impl Ord for Profile {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for Profile {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for Profile {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.name.cmp(&other.name))
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
pub type Profiles = collections::HashMap<String, Profile>;

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

    Ok(output.split_whitespace().any(|p| p == name))
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

    write!(file, "{}", rendered)?;

    Ok(())
}

/// Processes the configuration settings from a template into the ready-to-install profile.
fn process_settings(
    profile: &mut collections::HashMap<String, String>,
    template: &collections::HashMap<String, serde_json::Value>,
) -> subcommand::Result<()> {
    for (key, value) in template {
        if !profile.contains_key(key) {
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

            profile.insert(key, value);
        }
    }

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

        process_settings(&mut profile.settings, &template.settings)?;

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

            process_settings(&mut profile.settings, &template.settings)?;

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

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn process_templates_with_extends() {
        let templates: Templates = serde_json::from_value(json!({
            "a": {
                "enabled": false,
                "settings": {
                    "first": "1"
                }
            },
            "b": {
                "enabled": false,
                "extends": "a",
                "settings": {
                    "second": "2"
                }
            },
            "c": {
                "extends": "b",
                "settings": {
                    "third": "3"
                }
            },
            "d": {
                "extends": "b",
                "settings": {
                    "fourth": "4"
                }
            }
        }))
        .unwrap();

        let profiles: Profiles = process_templates(&templates).unwrap();
        let c: &Profile = profiles.get("c").unwrap();

        assert_eq!(c.settings.get("first").unwrap(), "1");
        assert_eq!(c.settings.get("second").unwrap(), "2");
        assert_eq!(c.settings.get("third").unwrap(), "3");

        let d: &Profile = profiles.get("d").unwrap();

        assert_eq!(d.settings.get("first").unwrap(), "1");
        assert_eq!(d.settings.get("second").unwrap(), "2");
        assert_eq!(d.settings.get("fourth").unwrap(), "4");
    }
}
