use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::Mutex;

// Global error counter for sites
static SITE_ERROR_COUNTER: Lazy<Mutex<HashMap<i64, u32>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Register one error for a site and return the new error count.
pub async fn register_site_error(site_id: i64) -> u32 {
    let mut map = SITE_ERROR_COUNTER.lock().await;
    let counter = map.entry(site_id).or_insert(0);
    *counter += 1;
    *counter
}

/// Reset error count for a site (e.g. on successful run).
pub async fn reset_site_error(site_id: i64) {
    let mut map = SITE_ERROR_COUNTER.lock().await;
    map.remove(&site_id);
}
