use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

pub struct JsonParse<T>(pub T);

impl<T> FromStr for JsonParse<T>
where
    T: for<'de> Deserialize<'de>,
{
    type Err = serde_json::Error;

    fn from_str(s: &str) -> serde_json::Result<Self> {
        Ok(JsonParse(serde_json::from_str(s)?))
    }
}

impl<T> Display for JsonParse<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&serde_json::to_string(&self.0).unwrap())
    }
}
