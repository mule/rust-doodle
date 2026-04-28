use async_trait::async_trait;
use chrono::{Datelike, NaiveDate};
use rand::{RngExt, SeedableRng};
use rand::rngs::StdRng;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Mutex;

use crate::topic::{Topic, TopicError, TopicSource};

const DEFAULT_BASE_URL: &str = "https://en.wikipedia.org/api/rest_v1/feed/onthisday/events";

pub struct WikipediaOnThisDay {
    http: Client,
    base_url: String,
    date: NaiveDate,
    rng: Mutex<StdRng>,
}

impl WikipediaOnThisDay {
    /// Production constructor: uses today's local date and a random seed.
    pub fn new() -> Self {
        Self::with_base_url_and_date(
            DEFAULT_BASE_URL.into(),
            chrono::Local::now().date_naive(),
            None,
        )
    }

    /// Test/internal constructor: caller controls base URL, date, and (optionally) RNG seed.
    pub fn with_base_url_and_date(
        base_url: String,
        date: NaiveDate,
        rng_seed: Option<u64>,
    ) -> Self {
        let rng = match rng_seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_rng(&mut rand::rng()),
        };
        Self { http: Client::new(), base_url, date, rng: Mutex::new(rng) }
    }
}

impl Default for WikipediaOnThisDay {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct Feed {
    events: Vec<Event>,
}

#[derive(Debug, Deserialize)]
struct Event {
    text: String,
    year: i32,
    #[serde(default)]
    pages: Vec<Page>,
}

#[derive(Debug, Deserialize)]
struct Page {
    #[serde(default)]
    extract: String,
}

#[async_trait]
impl TopicSource for WikipediaOnThisDay {
    fn name(&self) -> &'static str {
        "wikipedia"
    }

    async fn next_topic(&self) -> Result<Topic, TopicError> {
        let url = format!(
            "{}/{:02}/{:02}",
            self.base_url.trim_end_matches('/'),
            self.date.month(),
            self.date.day(),
        );

        let feed: Feed = self.http.get(&url).send().await?.error_for_status()?.json().await?;

        if feed.events.is_empty() {
            return Err(TopicError::NoEventsForDate);
        }

        let idx = {
            let mut rng = self.rng.lock().expect("rng poisoned");
            rng.random_range(0..feed.events.len())
        };
        let event = feed.events.into_iter().nth(idx).expect("idx in range");

        let seed = format!("In {}, {}", event.year, event.text);
        let context = if event.pages.is_empty() {
            None
        } else {
            let joined = event
                .pages
                .iter()
                .map(|p| p.extract.as_str())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n");
            (!joined.is_empty()).then_some(joined)
        };

        Ok(Topic { seed, context })
    }
}
