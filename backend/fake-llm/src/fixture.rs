use crate::constants::FIXTURE_FILE_EXTENSION;
use crate::provider::Provider;
use anyhow::{Context, bail};
use serde::Deserialize;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureFile {
    #[serde(rename = "match")]
    match_text: String,
    response: serde_json::Value,
}

#[derive(Clone, Debug)]
pub struct Fixture {
    pub name: String,
    pub provider: Provider,
    pub match_text: String,
    pub response: serde_json::Value,
}

pub fn load_fixtures(fixture_directory: &Path) -> anyhow::Result<Vec<Fixture>> {
    let mut fixtures = Vec::new();
    let provider_directories = fs::read_dir(fixture_directory).with_context(|| {
        format!(
            "cannot read fixture directory {}",
            fixture_directory.display()
        )
    })?;
    for provider_directory in provider_directories {
        let provider_directory = provider_directory?.path();
        let directory_name = provider_directory
            .file_name()
            .and_then(OsStr::to_str)
            .with_context(|| format!("invalid directory name {}", provider_directory.display()))?;
        let provider = Provider::from_str(directory_name).with_context(|| {
            format!(
                "{} is not a provider directory",
                provider_directory.display()
            )
        })?;
        for fixture_path in fs::read_dir(&provider_directory)? {
            let fixture_path = fixture_path?.path();
            fixtures.push(load_fixture(provider, &fixture_path)?);
        }
    }
    if fixtures.is_empty() {
        bail!("no fixtures found in {}", fixture_directory.display());
    }
    Ok(fixtures)
}

fn load_fixture(provider: Provider, fixture_path: &Path) -> anyhow::Result<Fixture> {
    if fixture_path.extension() != Some(OsStr::new(FIXTURE_FILE_EXTENSION)) {
        bail!(
            "{} is not a .{FIXTURE_FILE_EXTENSION} fixture",
            fixture_path.display()
        );
    }
    let name = fixture_path
        .file_stem()
        .and_then(OsStr::to_str)
        .with_context(|| format!("invalid fixture name {}", fixture_path.display()))?
        .to_string();
    let content = fs::read_to_string(fixture_path)
        .with_context(|| format!("cannot read fixture {}", fixture_path.display()))?;
    let fixture_file: FixtureFile = serde_json::from_str(&content)
        .with_context(|| format!("invalid fixture {}", fixture_path.display()))?;
    if fixture_file.match_text.is_empty() {
        bail!("fixture {} has an empty match", fixture_path.display());
    }
    Ok(Fixture {
        name,
        provider,
        match_text: fixture_file.match_text,
        response: fixture_file.response,
    })
}
