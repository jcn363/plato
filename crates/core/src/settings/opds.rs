use anyhow::Error;
use serde::{Deserialize, Serialize};
use crate::validation::validate_string_length;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct OpdsSettings {
    pub catalogs: Vec<OpdsCatalog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct OpdsCatalog {
    pub name: String,
    pub url: String,
}

impl Default for OpdsSettings {
    fn default() -> Self {
        OpdsSettings {
            catalogs: vec![
                OpdsCatalog {
                    name: "Feedbooks".to_string(),
                    url: "https://www.feedbooks.com/publicdomain/catalog.atom".to_string(),
                },
                OpdsCatalog {
                    name: "Project Gutenberg".to_string(),
                    url: "https://m.gutenberg.org/ebooks.opds/".to_string(),
                },
                OpdsCatalog {
                    name: "Standard Ebooks".to_string(),
                    url: "https://standardebooks.org/opds/all".to_string(),
                },
            ],
        }
    }
}

impl OpdsSettings {
    pub fn validate(&self) -> Result<(), Error> {
        for (i, catalog) in self.catalogs.iter().enumerate() {
            validate_string_length(&catalog.name, &format!("opds.catalogs[{}].name", i), 1, 100)?;
            if catalog.url.is_empty() {
                return Err(Error::msg(format!("opds.catalogs[{}].url cannot be empty", i)));
            }
        }
        Ok(())
    }
}
