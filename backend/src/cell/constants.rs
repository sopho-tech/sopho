use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CellType {
    Text,
    Code,
    Markdown,
    Sql,
    Chart,
}

impl fmt::Display for CellType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CellType::Text => "TEXT",
            CellType::Code => "CODE",
            CellType::Markdown => "MARKDOWN",
            CellType::Sql => "SQL",
            CellType::Chart => "CHART",
        })
    }
}

impl CellType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "TEXT" => Ok(CellType::Text),
            "CODE" => Ok(CellType::Code),
            "MARKDOWN" => Ok(CellType::Markdown),
            "SQL" => Ok(CellType::Sql),
            "CHART" => Ok(CellType::Chart),
            _ => Err(format!("Invalid cell type: {}", s)),
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

    pub fn deserialize_option_from_str<'de, D>(deserializer: D) -> Result<Option<Self>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) => Self::from_str(&s)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }

    pub fn serialize_option_to_str<S>(
        value: &Option<Self>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match value {
            Some(v) => {
                let s = v.to_string();
                serializer.serialize_some(&s)
            }
            None => serializer.serialize_none(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CellStatus {
    Active,
    Inactive,
}

impl CellStatus {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "ACTIVE" => Ok(CellStatus::Active),
            "INACTIVE" => Ok(CellStatus::Inactive),
            _ => Err(format!("Invalid cell status: {}", s)),
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

impl std::fmt::Display for CellStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            CellStatus::Active => "ACTIVE",
            CellStatus::Inactive => "INACTIVE",
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ChartOrientation {
    Horizontal,
    Vertical,
}

impl ChartOrientation {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "HORIZONTAL" => Ok(ChartOrientation::Horizontal),
            "VERTICAL" => Ok(ChartOrientation::Vertical),
            _ => Err(format!("Invalid chart orientation: {}", s)),
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

    pub fn deserialize_option_from_str<'de, D>(deserializer: D) -> Result<Option<Self>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) => Self::from_str(&s)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }

    pub fn serialize_option_to_str<S>(
        value: &Option<Self>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match value {
            Some(v) => {
                let s = v.to_string();
                serializer.serialize_some(&s)
            }
            None => serializer.serialize_none(),
        }
    }
}

impl std::fmt::Display for ChartOrientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ChartOrientation::Horizontal => "HORIZONTAL",
            ChartOrientation::Vertical => "VERTICAL",
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SortOrder {
    None,
    Asc,
    Desc,
}

impl SortOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortOrder::None => "NONE",
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "NONE" => Ok(SortOrder::None),
            "ASC" => Ok(SortOrder::Asc),
            "DESC" => Ok(SortOrder::Desc),
            _ => Err(format!("Invalid sort order: {}", s)),
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

    pub fn deserialize_option_from_str<'de, D>(deserializer: D) -> Result<Option<Self>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) => Self::from_str(&s)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }

    pub fn serialize_option_to_str<S>(
        value: &Option<Self>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match value {
            Some(v) => {
                let s = v.to_string();
                serializer.serialize_some(&s)
            }
            None => serializer.serialize_none(),
        }
    }
}

impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AxisTickShow {
    Show,
    Hide,
}

impl AxisTickShow {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "SHOW" => Ok(AxisTickShow::Show),
            "HIDE" => Ok(AxisTickShow::Hide),
            _ => Err(format!("Invalid axis tick show: {}", s)),
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

    pub fn deserialize_option_from_str<'de, D>(deserializer: D) -> Result<Option<Self>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) => Self::from_str(&s)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }

    pub fn serialize_option_to_str<S>(
        value: &Option<Self>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match value {
            Some(v) => {
                let s = v.to_string();
                serializer.serialize_some(&s)
            }
            None => serializer.serialize_none(),
        }
    }
}

impl std::fmt::Display for AxisTickShow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AxisTickShow::Show => "SHOW",
            AxisTickShow::Hide => "HIDE",
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AxisMinorTickShow {
    Show,
    Hide,
}

impl AxisMinorTickShow {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "SHOW" => Ok(AxisMinorTickShow::Show),
            "HIDE" => Ok(AxisMinorTickShow::Hide),
            _ => Err(format!("Invalid axis minor tick show: {}", s)),
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

    pub fn deserialize_option_from_str<'de, D>(deserializer: D) -> Result<Option<Self>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) => Self::from_str(&s)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }

    pub fn serialize_option_to_str<S>(
        value: &Option<Self>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match value {
            Some(v) => {
                let s = v.to_string();
                serializer.serialize_some(&s)
            }
            None => serializer.serialize_none(),
        }
    }
}

impl std::fmt::Display for AxisMinorTickShow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AxisMinorTickShow::Show => "SHOW",
            AxisMinorTickShow::Hide => "HIDE",
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum MetricFormat {
    Default,
    Percentage,
    Currency,
}

impl MetricFormat {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "DEFAULT" => Ok(MetricFormat::Default),
            "PERCENTAGE" => Ok(MetricFormat::Percentage),
            "CURRENCY" => Ok(MetricFormat::Currency),
            _ => Err(format!("Invalid metric format: {}", s)),
        }
    }

    pub fn deserialize_option_from_str<'de, D>(deserializer: D) -> Result<Option<Self>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) => Self::from_str(&s)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }

    pub fn serialize_option_to_str<S>(
        value: &Option<Self>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match value {
            Some(v) => {
                let s = v.to_string();
                serializer.serialize_some(&s)
            }
            None => serializer.serialize_none(),
        }
    }
}

impl std::fmt::Display for MetricFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MetricFormat::Default => "DEFAULT",
            MetricFormat::Percentage => "PERCENTAGE",
            MetricFormat::Currency => "CURRENCY",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChartType {
    Bar,
    Line,
    Pie,
    Metric,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CellDisplayOrderMovement {
    Up,
    Down,
    Top,
    Bottom,
}

impl CellDisplayOrderMovement {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "UP" => Ok(CellDisplayOrderMovement::Up),
            "DOWN" => Ok(CellDisplayOrderMovement::Down),
            "TOP" => Ok(CellDisplayOrderMovement::Top),
            "BOTTOM" => Ok(CellDisplayOrderMovement::Bottom),
            _ => Err(format!("Invalid movement type: {}", s)),
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

impl std::fmt::Display for CellDisplayOrderMovement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            CellDisplayOrderMovement::Up => "UP",
            CellDisplayOrderMovement::Down => "DOWN",
            CellDisplayOrderMovement::Top => "TOP",
            CellDisplayOrderMovement::Bottom => "BOTTOM",
        })
    }
}
