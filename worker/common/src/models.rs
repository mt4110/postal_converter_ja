use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct PostalCode {
    pub zip_code: String,
    pub prefecture_id: i16,
    pub city_id: String,
    pub prefecture: String,
    pub city: String,
    pub town: String,
}
