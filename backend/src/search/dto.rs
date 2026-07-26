use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

fn deserialize_filters<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct FiltersVisitor;

    impl<'de> Visitor<'de> for FiltersVisitor {
        type Value = Option<Vec<String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or a sequence of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(
                    value.split(',').map(|s| s.trim().to_string()).collect(),
                ))
            }
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(elem) = seq.next_element::<String>()? {
                vec.push(elem);
            }
            if vec.is_empty() {
                Ok(None)
            } else {
                Ok(Some(vec))
            }
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            Deserialize::deserialize(deserializer)
        }
    }

    deserializer.deserialize_any(FiltersVisitor)
}

#[derive(Deserialize)]
pub struct SearchRequestDto {
    #[serde(rename = "query")]
    pub query: Option<String>,
    #[serde(rename = "filters", deserialize_with = "deserialize_filters")]
    pub filters: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SearchResultItemDto {
    pub id: Uuid,
    pub name: Option<String>,
    pub entity_type: String,
    pub updated_at: DateTime<FixedOffset>,
    pub canvas_id: Option<Uuid>,
}
