use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum DashboardStatus {
    Active,
    Inactive,
}

impl DashboardStatus {
    pub fn to_string(&self) -> String {
        match self {
            DashboardStatus::Active => "ACTIVE".to_string(),
            DashboardStatus::Inactive => "INACTIVE".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "ACTIVE" => Ok(DashboardStatus::Active),
            "INACTIVE" => Ok(DashboardStatus::Inactive),
            _ => Err(format!("Invalid dashboard status: {}", s)),
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
