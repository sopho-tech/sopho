use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum SourceType {
    Postgresql,
    MySql,
    Sqlite,
    MsSql,
    Oracle,
    MongoDb,
    Redis,
    Elasticsearch,
    Kafka,
    Api,
    File,
    GoogleSheets,
    Airtable,
}

impl SourceType {
    pub fn to_string(&self) -> String {
        match self {
            SourceType::Postgresql => "POSTGRESQL".to_string(),
            SourceType::MySql => "MYSQL".to_string(),
            SourceType::Sqlite => "SQLITE".to_string(),
            SourceType::MsSql => "MS_SQL".to_string(),
            SourceType::Oracle => "ORACLE".to_string(),
            SourceType::MongoDb => "MONGODB".to_string(),
            SourceType::Redis => "REDIS".to_string(),
            SourceType::Elasticsearch => "ELASTICSEARCH".to_string(),
            SourceType::Kafka => "KAFKA".to_string(),
            SourceType::Api => "API".to_string(),
            SourceType::File => "FILE".to_string(),
            SourceType::GoogleSheets => "GOOGLE_SHEETS".to_string(),
            SourceType::Airtable => "AIRTABLE".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "POSTGRESQL" => Ok(SourceType::Postgresql),
            "MYSQL" => Ok(SourceType::MySql),
            "SQLITE" => Ok(SourceType::Sqlite),
            "MS_SQL" => Ok(SourceType::MsSql),
            "ORACLE" => Ok(SourceType::Oracle),
            "MONGODB" => Ok(SourceType::MongoDb),
            "REDIS" => Ok(SourceType::Redis),
            "ELASTICSEARCH" => Ok(SourceType::Elasticsearch),
            "KAFKA" => Ok(SourceType::Kafka),
            "API" => Ok(SourceType::Api),
            "FILE" => Ok(SourceType::File),
            "GOOGLE_SHEETS" => Ok(SourceType::GoogleSheets),
            "AIRTABLE" => Ok(SourceType::Airtable),
            _ => Err(format!("Invalid source type: {}", s)),
        }
    }

    pub fn deserialize_from_str<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }

    pub fn serialize_to_str<S>(value: &Self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = value.to_string();
        serializer.serialize_str(&s)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Active,
    Inactive,
    Failed,
}

impl ConnectionStatus {
    pub fn to_string(&self) -> String {
        match self {
            ConnectionStatus::Active => "ACTIVE".to_string(),
            ConnectionStatus::Inactive => "INACTIVE".to_string(),
            ConnectionStatus::Failed => "FAILED".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "ACTIVE" => Ok(ConnectionStatus::Active),
            "INACTIVE" => Ok(ConnectionStatus::Inactive),
            "FAILED" => Ok(ConnectionStatus::Failed),
            _ => Err(format!("Invalid connection status: {}", s)),
        }
    }

    pub fn deserialize_from_str<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }

    pub fn serialize_to_str<S>(value: &Self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = value.to_string();
        serializer.serialize_str(&s)
    }
}
