use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum AxisDirection {
    Positive,
    Negative,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum Axis {
    #[serde(rename = "x")]
    X,
    #[serde(rename = "y")]
    Y,
    #[serde(rename = "z")]
    Z,
}
