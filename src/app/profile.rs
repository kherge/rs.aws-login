//! Manages loading and parsing of profile templates.

use crate::util::config::CONFIG_DIR;
use carli::error::{Context, Error, Result};
use carli::{err, error};
use std::{collections, fmt, fs, io, path};

lazy_static::lazy_static! {
    /// The path to the file containing the profile templates.
    pub static ref TEMPLATES_FILE: path::PathBuf = CONFIG_DIR.join("templates.json");
}

/// Manages an AWS CLI profile that is ready to be installed.
pub struct Profile {
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
    /// Returns the name of the profile.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the profile configuration settings.
    pub fn settings(&self) -> &collections::HashMap<String, String> {
        &self.settings
    }
}

/// A specialized [`Result`] type for a named collection of [`Profile`].
pub type Profiles = collections::HashMap<String, Profile>;

/// Manages a profile template that can be used to generate an AWS CLI profile.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Template {
    /// The flag used to determine if the profile can be use directly.
    #[serde(default = "Template::enabled_default")]
    enabled: bool,

    /// The name of the profile to extend.
    extends: Option<String>,

    /// The profile configuration settings.
    settings: collections::HashMap<String, serde_json::Value>,
}

impl Template {
    /// Returns the default enabled state for all profiles.
    fn enabled_default() -> bool {
        true
    }

    /// Generates a new [`Profile`] from the template.
    ///
    /// ```
    /// let templates = get_templates()?;
    /// let template = templates.get("example").unwrap();
    /// let profile = template.to_profile(&templates);
    /// ```
    fn to_profile(&self, name: &str, templates: &Templates) -> Result<Profile> {
        let mut extends = &self.extends;
        let mut settings = collections::HashMap::new();

        for (key, value) in &self.settings {
            settings.insert(
                key.clone(),
                convert_value(value)
                    .context(|| format!("Could not convert the value of, {}.", key))?,
            );
        }

        while let Some(extends_name) = extends {
            if let Some(template) = templates.get(extends_name) {
                for (key, value) in &template.settings {
                    if !settings.contains_key(key) {
                        settings.insert(
                            key.clone(),
                            convert_value(value).context(|| {
                                format!(
                                    "Could not convert the value of, {}, in the profile template, {}.",
                                    key,
                                    extends_name
                            )})?,
                        );
                    }
                }

                extends = &template.extends;
            } else {
                err!(
                    1,
                    "{}: The profile template, {}, does not exist.",
                    name,
                    extends_name
                );
            }
        }

        Ok(Profile {
            name: name.to_owned(),
            settings,
        })
    }
}

/// A specialized [`Result`] type for a named collection of [`Template`].
pub type Templates = collections::HashMap<String, Template>;

/// Converts a [`serde_json::Value`] into a [`String`].
fn convert_value(value: &serde_json::Value) -> Result<String> {
    use serde_json::Value;

    match value {
        Value::Bool(value) => Ok(value.to_string()),
        Value::Null => Ok("".to_owned()),
        Value::Number(value) => Ok(value.to_string()),
        Value::String(value) => Ok(value.to_owned()),
        _ => err!(1, "The array and object values are not supported."),
    }
}

/// Processes profile templates into AWS CLI profiles and returns them.
pub fn get_profiles() -> Result<Profiles> {
    let mut profiles = Profiles::new();
    let templates = get_templates()?;

    for (name, template) in &templates {
        if template.enabled {
            profiles.insert(name.clone(), template.to_profile(name, &templates)?);
        }
    }

    Ok(profiles)
}

/// Reads and parses profile templates from the local file.
pub fn get_templates() -> Result<Templates> {
    if !TEMPLATES_FILE.exists() {
        return Ok(Templates::new());
    }

    read_templates(&*TEMPLATES_FILE)
}

/// Parses profile templates from the given stream reader.
pub fn parse_templates(reader: impl io::Read) -> Result<Templates> {
    match serde_json::from_reader(reader) {
        Ok(templates) => Ok(templates),
        Err(error) => err!(1, "{}", error),
    }
}

/// Reads and parses profile templates from a JSON encoded file.
fn read_templates(path: &path::Path) -> Result<Templates> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) => {
            return Err(Error::from(error).context(format!(
                "Could not read the profile templates file: {}",
                path.display()
            )))
        }
    };

    let reader = io::BufReader::new(file);

    parse_templates(reader).context(|| {
        format!(
            "Could not parse the profile templates file: {}",
            path.display()
        )
    })
}

/// Saves the templates to the local file.
pub fn set_templates(templates: &Templates) -> Result<()> {
    let file = match fs::File::create(&*TEMPLATES_FILE) {
        Ok(file) => file,
        Err(error) => {
            return Err(Error::from(error).context(format!(
                "Could not write to the profile templates file: {}",
                TEMPLATES_FILE.display()
            )))
        }
    };

    let writer = io::BufWriter::new(file);

    if let Err(error) = serde_json::to_writer_pretty(writer, templates) {
        return Err(error!(1, "{}", error).context(format!(
            "Could not serialize the templates to the local file: {}",
            TEMPLATES_FILE.display()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    fn create_settings<T>(
        builder: impl FnOnce(&mut collections::HashMap<String, T>),
    ) -> collections::HashMap<String, T> {
        let mut map = collections::HashMap::new();

        builder(&mut map);

        map
    }

    #[test]
    fn template_to_profile() {
        let mut templates = Templates::new();

        templates.insert(
            "a".to_owned(),
            Template {
                extends: None,
                enabled: false,
                settings: create_settings(|map| {
                    map.insert("alpha".to_owned(), json!(1));
                    map.insert("gamma".to_owned(), json!(2));
                    map.insert("epsilon".to_owned(), json!(3));
                }),
            },
        );

        templates.insert(
            "b".to_owned(),
            Template {
                extends: Some("a".to_owned()),
                enabled: false,
                settings: create_settings(|map| {
                    map.insert("beta".to_owned(), json!(4));
                    map.insert("delta".to_owned(), json!(5));
                    map.insert("zeta".to_owned(), json!(6));
                }),
            },
        );

        templates.insert(
            "c".to_owned(),
            Template {
                extends: Some("b".to_owned()),
                enabled: true,
                settings: create_settings(|map| {
                    map.insert("alpha".to_owned(), json!(7));
                    map.insert("zeta".to_owned(), json!(8));
                }),
            },
        );

        let profile = templates
            .get("c")
            .unwrap()
            .to_profile("c", &templates)
            .unwrap();

        assert_eq!(profile.settings.get("alpha").unwrap(), "7");
        assert_eq!(profile.settings.get("beta").unwrap(), "4");
        assert_eq!(profile.settings.get("gamma").unwrap(), "2");
        assert_eq!(profile.settings.get("delta").unwrap(), "5");
        assert_eq!(profile.settings.get("epsilon").unwrap(), "3");
        assert_eq!(profile.settings.get("zeta").unwrap(), "8");
    }
}
