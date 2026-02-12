use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash, ToSchema)]
pub struct PostalCode {
    pub zip_code: String,
    pub prefecture_id: i16,
    pub city_id: String,
    pub prefecture: String,
    pub city: String,
    pub town: String,
}
