use crate::support::constants::{METADATA_DATABASE_PREFIX, TEST_DATABASE_VARIABLE};
use crate::support::postgres_server::PostgresDatabase;
use std::str::FromStr;
use strum::EnumString;
use tempfile::TempDir;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum MetadataDatabaseKind {
    Sqlite,
    Postgres,
}

pub fn metadata_database_kind() -> MetadataDatabaseKind {
    let value = std::env::var(TEST_DATABASE_VARIABLE).unwrap_or_else(|error| {
        panic!("{TEST_DATABASE_VARIABLE} must be set to sqlite or postgres: {error}")
    });
    MetadataDatabaseKind::from_str(&value).unwrap_or_else(|_| {
        panic!("{TEST_DATABASE_VARIABLE} must be sqlite or postgres, got {value}")
    })
}

pub enum MetadataDatabase {
    Sqlite { directory: TempDir },
    Postgres { database: PostgresDatabase },
}

impl MetadataDatabase {
    pub async fn create() -> Self {
        match metadata_database_kind() {
            MetadataDatabaseKind::Sqlite => Self::Sqlite {
                directory: tempfile::tempdir().expect("create a temporary directory"),
            },
            MetadataDatabaseKind::Postgres => Self::Postgres {
                database: PostgresDatabase::create(METADATA_DATABASE_PREFIX).await,
            },
        }
    }

    pub fn url(&self) -> String {
        match self {
            Self::Sqlite { directory } => format!(
                "sqlite://{}?mode=rwc",
                directory.path().join("metadata.sqlite").display()
            ),
            Self::Postgres { database } => database.url.clone(),
        }
    }
}
