use async_trait::async_trait;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use std::sync::Mutex;

use crate::topic::{Topic, TopicError, TopicSource};

const SEEDS: &[&str] = &[
    "the sea",
    "old machinery",
    "morning light",
    "loneliness",
    "snowfall in a small town",
    "an empty train station",
    "a cat asleep in the sun",
    "the smell of rain on dry stone",
    "stars over a city skyline",
    "footprints on a beach at dusk",
    "a half-finished letter",
    "the hum of a refrigerator at night",
];

pub struct RandomTopic {
    rng: Mutex<StdRng>,
}

impl RandomTopic {
    /// Construct with an OS-seeded RNG (via the thread-local default).
    pub fn new() -> Self {
        Self { rng: Mutex::new(StdRng::from_rng(&mut rand::rng())) }
    }

    /// Construct with a deterministic seed (used by tests).
    pub fn from_seed(seed: u64) -> Self {
        Self { rng: Mutex::new(StdRng::seed_from_u64(seed)) }
    }
}

impl Default for RandomTopic {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TopicSource for RandomTopic {
    fn name(&self) -> &'static str {
        "random"
    }

    async fn next_topic(&self) -> Result<Topic, TopicError> {
        let mut rng = self.rng.lock().expect("rng poisoned");
        let seed = SEEDS
            .choose(&mut *rng)
            .expect("SEEDS is non-empty")
            .to_string();
        Ok(Topic { seed, context: None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn picks_from_seed_list() {
        let s = RandomTopic::from_seed(42);
        let t = s.next_topic().await.unwrap();
        assert!(SEEDS.contains(&t.seed.as_str()), "got unknown seed: {}", t.seed);
    }

    #[tokio::test]
    async fn deterministic_with_fixed_seed() {
        let a = RandomTopic::from_seed(7).next_topic().await.unwrap();
        let b = RandomTopic::from_seed(7).next_topic().await.unwrap();
        assert_eq!(a.seed, b.seed);
    }
}
