use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Items<T> {
    pub items: Vec<T>,
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
    pub total_pages: u64,
}

impl<T> Items<T> {
    pub fn new(items: Vec<T>, page: u64, per_page: u64, total: u64) -> Self {
        let per = per_page.clamp(1, 100);
        let total_pages = total.div_ceil(per).max(1);
        Self {
            items,
            page: page.max(1),
            per_page: per,
            total,
            total_pages,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct PaginationParams {
    #[serde(deserialize_with = "deserialize_opt_u64")]
    pub page: Option<u64>,
    #[serde(deserialize_with = "deserialize_opt_u64")]
    pub per_page: Option<u64>,
}

impl PaginationParams {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1)
    }

    pub fn per_page(&self) -> u64 {
        self.per_page.unwrap_or(20).clamp(1, 100)
    }
}

/// Accepts either numeric or string query values to keep pagination parsing flexible.
fn deserialize_opt_u64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum U64OrString {
        Number(u64),
        Text(String),
    }

    let value = Option::<U64OrString>::deserialize(deserializer)?;

    match value {
        Some(U64OrString::Number(n)) => Ok(Some(n)),
        Some(U64OrString::Text(text)) => text
            .parse::<u64>()
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}
