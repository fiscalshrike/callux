use crate::auth::AuthManager;
use crate::cache::EventCache;
use crate::config::Config;
use crate::error::{CalendarError, Result};
use crate::output::CalendarEvent;
use chrono::{DateTime, Local, TimeZone, Utc};
use google_calendar3::{CalendarHub, api::{CalendarListEntry, Event}};
use google_calendar3::hyper::client::HttpConnector;
use google_calendar3::hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};

pub struct CalendarClient {
    config: Config,
    auth_manager: AuthManager,
    cache: EventCache,
}

impl CalendarClient {
    pub fn new(config: Config) -> Self {
        let auth_manager = AuthManager::new(config.clone());
        let cache = EventCache::new(&config.cache);

        Self {
            config,
            auth_manager,
            cache,
        }
    }

    pub async fn get_events(&self, days_ahead: i64, limit: Option<usize>) -> Result<Vec<CalendarEvent>> {
        let enabled_calendars: Vec<_> = self.config.calendars
            .iter()
            .filter(|cal| cal.enabled)
            .collect();

        let calendar_ids: Vec<String> = enabled_calendars
            .iter()
            .map(|cal| cal.id.clone())
            .collect();

        let cache_key = self.cache.generate_key(&calendar_ids, days_ahead);
        
        if let Some(cached_events) = self.cache.get(&cache_key).await {
            return Ok(cached_events);
        }

        let events = self.fetch_events_from_api(&calendar_ids, days_ahead).await?;
        self.cache.set(cache_key, events.clone()).await;

        let limited_events = if let Some(limit) = limit {
            events.into_iter().take(limit).collect()
        } else {
            events
        };

        Ok(limited_events)
    }

    async fn fetch_events_from_api(&self, calendar_ids: &[String], days_ahead: i64) -> Result<Vec<CalendarEvent>> {
        let authenticator = self.auth_manager.get_authenticator().await?;
        
        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .map_err(|e| CalendarError::ApiError(format!("Failed to build HTTPS connector: {}", e)))?
            .https_or_http()
            .enable_http1()
            .build();
        
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        let hub = CalendarHub::new(client, authenticator);

        let now = Utc::now();
        let end_time = now + chrono::Duration::days(days_ahead);

        let mut all_events = Vec::new();

        for calendar_id in calendar_ids {
            match self.fetch_calendar_events(&hub, calendar_id, &now, &end_time).await {
                Ok(events) => all_events.extend(events),
                Err(e) => {
                    eprintln!("Warning: Failed to fetch events from calendar {}: {}", calendar_id, e);
                }
            }
        }

        all_events.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        Ok(all_events)
    }

    async fn fetch_calendar_events(
        &self,
        hub: &CalendarHub<HttpsConnector<HttpConnector>>,
        calendar_id: &str,
        start_time: &DateTime<Utc>,
        end_time: &DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>> {
        let result = hub
            .events()
            .list(calendar_id)
            .time_min(*start_time)
            .time_max(*end_time)
            .single_events(true)
            .order_by("startTime")
            .max_results(250)
            .doit()
            .await
            .map_err(|e| CalendarError::ApiError(format!("Failed to fetch events: {}", e)))?;

        let calendar_config = self.config.calendars
            .iter()
            .find(|cal| cal.id == calendar_id)
            .ok_or_else(|| CalendarError::ConfigError(format!("Calendar config not found for ID: {}", calendar_id)))?;

        let events = result.1.items.unwrap_or_default();
        let mut calendar_events = Vec::new();

        for event in events {
            if let Some(cal_event) = self.convert_event(event, calendar_config)? {
                calendar_events.push(cal_event);
            }
        }

        Ok(calendar_events)
    }

    fn convert_event(&self, event: Event, calendar_config: &crate::config::CalendarConfig) -> Result<Option<CalendarEvent>> {
        let id = event.id.unwrap_or_default();
        let title = event.summary.unwrap_or_else(|| "Untitled Event".to_string());
        let description = event.description;

        let (start_time, end_time, all_day) = if let Some(start) = event.start {
            if let Some(date_time) = &start.date_time {
                let start_dt = date_time.with_timezone(&Local);
                
                let end_dt = if let Some(end) = event.end {
                    if let Some(end_date_time) = &end.date_time {
                        end_date_time.with_timezone(&Local)
                    } else {
                        start_dt + chrono::Duration::hours(1)
                    }
                } else {
                    start_dt + chrono::Duration::hours(1)
                };
                
                (start_dt, end_dt, false)
            } else if let Some(date) = &start.date {
                let start_dt = Local.from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap()).unwrap();
                let end_dt = start_dt + chrono::Duration::days(1);
                
                (start_dt, end_dt, true)
            } else {
                return Ok(None);
            }
        } else {
            return Ok(None);
        };

        Ok(Some(CalendarEvent {
            id,
            title,
            description,
            start_time,
            end_time,
            calendar_name: calendar_config.name.clone(),
            calendar_color: calendar_config.color.clone(),
            all_day,
        }))
    }

    pub async fn list_calendars(&self) -> Result<Vec<CalendarListEntry>> {
        let authenticator = self.auth_manager.get_authenticator().await?;
        
        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .map_err(|e| CalendarError::ApiError(format!("Failed to build HTTPS connector: {}", e)))?
            .https_or_http()
            .enable_http1()
            .build();
        
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        let hub = CalendarHub::new(client, authenticator);

        let result = hub
            .calendar_list()
            .list()
            .doit()
            .await
            .map_err(|e| CalendarError::ApiError(format!("Failed to list calendars: {}", e)))?;

        Ok(result.1.items.unwrap_or_default())
    }

    pub async fn clear_cache(&self) {
        self.cache.clear().await;
    }
}