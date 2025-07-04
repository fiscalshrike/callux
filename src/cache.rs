use crate::config::CacheConfig;
use crate::output::CalendarEvent;
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

pub struct EventCache {
    cache: Cache<String, Vec<CalendarEvent>>,
}

impl EventCache {
    pub fn new(config: &CacheConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.max_entries)
            .time_to_live(Duration::from_secs(config.ttl_seconds))
            .build();

        Self { cache }
    }

    pub async fn get(&self, key: &str) -> Option<Vec<CalendarEvent>> {
        self.cache.get(key).await
    }

    pub async fn set(&self, key: String, events: Vec<CalendarEvent>) {
        self.cache.insert(key, events).await;
    }

    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(key).await;
    }

    pub async fn clear(&self) {
        self.cache.invalidate_all();
    }

    pub fn generate_key(&self, calendar_ids: &[String], days_ahead: i64) -> String {
        let mut key = calendar_ids.join(",");
        key.push_str(&format!(":{}", days_ahead));
        key
    }
}