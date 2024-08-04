use serde::{de::Visitor, Deserialize};
use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Deserialize)]
pub enum BlockStates {
    #[serde(rename = "variants")]
    Variants(HashMap<VariantCondition, Variant>),
    #[serde(rename = "multipart")]
    Multipart(Vec<Multipart>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum PropertyValue {
    Number(i64),
    Boolean(bool),
    String(String),
}

impl From<i64> for PropertyValue {
    fn from(value: i64) -> Self {
        Self::Number(value)
    }
}

impl From<bool> for PropertyValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<String> for PropertyValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for PropertyValue {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Property {
    pub value: PropertyValue,
}

impl Property {
    pub fn new(default: PropertyValue) -> Self {
        Self { value: default }
    }
}

impl<T: Into<PropertyValue>> From<T> for Property {
    fn from(value: T) -> Self {
        Self::new(value.into())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct VariantCondition {
    _properties: String,
    pub properties: HashMap<String, Property>,
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

        if result.trim().is_empty() {
            return Ok(Self {
                _properties: result,
                properties,
            });
        }

        for key_value in result.split(',') {
            let mut key_value = key_value.trim().split('=');

            if let (Some(key), Some(value)) = (key_value.next(), key_value.next()) {
                let value = value
                    .parse::<i64>()
                    .map(PropertyValue::Number)
                    .or_else(|_| value.parse::<bool>().map(PropertyValue::Boolean))
                    .unwrap_or_else(|_| PropertyValue::String(value.to_string()));

                properties.insert(key.trim().to_owned(), Property::new(value));
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
    #[serde(default)]
    pub x: u16,
    #[serde(default)]
    pub y: u16,
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
