use serde::{de::Visitor, Deserialize};
use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Deserialize)]
pub enum BlockStates {
    #[serde(rename = "variants")]
    Variants(HashMap<VariantCondition, Variant>),
    #[serde(rename = "multipart")]
    Multipart(Vec<Multipart>),
}

// enum ConditionValue {
//     Bool(bool),
//     Number(i64),
//     Face(String),
//     Facing(Direction)
// }

#[derive(Debug, PartialEq, Eq)]
pub struct VariantCondition {
    _properties: String,
    pub properties: HashMap<String, String>,
}

impl Hash for VariantCondition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self._properties.hash(state);
    }
}

struct StringVisitor;

impl<'de> Visitor<'de> for StringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }
}

impl<'de> Deserialize<'de> for VariantCondition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let result = deserializer.deserialize_string(StringVisitor)?;
        let mut properties = HashMap::default();

        for key_value in result.split(',') {
            let mut key_value = key_value.trim().split('=');

            if let (Some(key), Some(value)) = (key_value.next(), key_value.next()) {
                properties.insert(key.trim().to_owned(), value.trim().to_owned());
            } else {
                return Err(serde::de::Error::custom(
                    "variant name must follow next syntax: key=value,key1=value",
                ));
            }
        }

        Ok(Self {
            _properties: result,
            properties,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Variant {
    One(Model),
    Many(Vec<Model>),
}

#[derive(Debug, Deserialize)]
pub struct Model {
    pub model: String,
    pub x: Option<usize>,
    pub y: Option<usize>,
    #[serde(default)]
    pub uvlock: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MultipartCondition {
    Or {
        #[serde(rename = "OR")]
        elements: Vec<MultipartCondition>,
    },
    And {
        #[serde(rename = "AND")]
        elements: Vec<MultipartCondition>,
    },
    Condition(HashMap<String, String>),
}

#[derive(Debug, Deserialize)]
pub struct Multipart {
    pub when: MultipartCondition,
    pub apply: Model,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::BlockStates;

    #[test]
    fn test_parse() {
        println!(
            "{:#?}",
            serde_json::from_slice::<'_, BlockStates>(
                &fs::read("/run/media/aiving/Drive/dev/maqo/assets/blockstates/redstone_wire.json")
                    .unwrap()
            )
        )
    }
}
