use crate::cli::OutputFormat;
use chrono::{DateTime, Local};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub calendar_name: String,
    pub calendar_color: String,
    pub all_day: bool,
}

#[derive(Debug, Serialize)]
pub struct WaybarOutput {
    pub text: String,
    pub tooltip: String,
    pub class: String,
    pub percentage: u8,
}

pub struct OutputFormatter {
    format: OutputFormat,
    date_format: String,
    max_events: usize,
}

impl OutputFormatter {
    pub fn new(format: OutputFormat, date_format: String, max_events: usize) -> Self {
        Self {
            format,
            date_format,
            max_events,
        }
    }

    pub fn format_events(&self, events: &[CalendarEvent]) -> String {
        let limited_events: Vec<&CalendarEvent> = events.iter().take(self.max_events).collect();

        match self.format {
            OutputFormat::Json => self.format_json(&limited_events),
            OutputFormat::Human => self.format_human(&limited_events),
            OutputFormat::Colored => self.format_colored(&limited_events),
        }
    }

    fn format_json(&self, events: &[&CalendarEvent]) -> String {
        let waybar_output = if events.is_empty() {
            WaybarOutput {
                text: "No events".to_string(),
                tooltip: "No upcoming events".to_string(),
                class: "calendar-empty".to_string(),
                percentage: 0,
            }
        } else {
            let next_event = events[0];
            let text = if next_event.all_day {
                format!("{}", next_event.title)
            } else {
                format!(
                    "{} {}",
                    next_event.start_time.format("%H:%M"),
                    next_event.title
                )
            };

            let tooltip = self.create_tooltip(events);
            let class = if events.len() > 1 {
                "calendar-multiple".to_string()
            } else {
                "calendar-single".to_string()
            };

            WaybarOutput {
                text,
                tooltip,
                class,
                percentage: std::cmp::min(events.len() * 10, 100) as u8,
            }
        };

        serde_json::to_string(&waybar_output).unwrap_or_else(|_| "{}".to_string())
    }

    fn format_human(&self, events: &[&CalendarEvent]) -> String {
        if events.is_empty() {
            return "No upcoming events".to_string();
        }

        let mut output = String::new();
        let mut current_date = String::new();

        for event in events {
            let event_date = event.start_time.format("%Y-%m-%d").to_string();
            if event_date != current_date {
                if !current_date.is_empty() {
                    output.push('\n');
                }
                output.push_str(&format!("{}\n", event.start_time.format("%A, %B %d, %Y")));
                current_date = event_date;
            }

            if event.all_day {
                output.push_str(&format!("  All day: {}\n", event.title));
            } else {
                output.push_str(&format!(
                    "  {}: {}\n",
                    event.start_time.format(&self.date_format),
                    event.title
                ));
            }
        }

        output.trim_end().to_string()
    }

    fn format_colored(&self, events: &[&CalendarEvent]) -> String {
        if events.is_empty() {
            return "No upcoming events".bright_yellow().to_string();
        }

        let mut output = String::new();
        let mut current_date = String::new();

        for event in events {
            let event_date = event.start_time.format("%Y-%m-%d").to_string();
            if event_date != current_date {
                if !current_date.is_empty() {
                    output.push('\n');
                }
                output.push_str(&format!(
                    "{}\n",
                    event
                        .start_time
                        .format("%A, %B %d, %Y")
                        .to_string()
                        .bright_blue()
                        .bold()
                ));
                current_date = event_date;
            }

            if event.all_day {
                output.push_str(&format!(
                    "  {}: {}\n",
                    "All day".bright_green(),
                    event.title.white()
                ));
            } else {
                output.push_str(&format!(
                    "  {}: {}\n",
                    event
                        .start_time
                        .format(&self.date_format)
                        .to_string()
                        .bright_green(),
                    event.title.white()
                ));
            }
        }

        output.trim_end().to_string()
    }

    fn create_tooltip(&self, events: &[&CalendarEvent]) -> String {
        let mut tooltip = String::new();
        let mut events_by_date: HashMap<String, Vec<&CalendarEvent>> = HashMap::new();

        for event in events {
            let date_key = event.start_time.format("%Y-%m-%d").to_string();
            events_by_date
                .entry(date_key)
                .or_insert_with(Vec::new)
                .push(event);
        }

        let mut sorted_dates: Vec<String> = events_by_date.keys().cloned().collect();
        sorted_dates.sort();

        for (i, date) in sorted_dates.iter().enumerate() {
            if i > 0 {
                tooltip.push_str("\n\n");
            }

            let events_on_date = &events_by_date[date];
            let parsed_date = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
            let formatted_date = parsed_date.format("%A, %B %d");

            tooltip.push_str(&format!("{}:\n", formatted_date));

            for event in events_on_date {
                if event.all_day {
                    tooltip.push_str(&format!("• All day: {}\n", event.title));
                } else {
                    tooltip.push_str(&format!(
                        "• {}: {}\n",
                        event.start_time.format("%H:%M"),
                        event.title
                    ));
                }
            }
        }

        tooltip.trim_end().to_string()
    }
}
