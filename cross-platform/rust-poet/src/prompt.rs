use crate::provider::{Message, Role};
use crate::topic::Topic;

const SYSTEM_PROMPT: &str = "You are a poet. Write a short, evocative poem on the requested topic. \
Use vivid imagery and concrete sensory detail. Return only the poem itself — no preamble, no \
explanation, no title, no quotation marks around it.";

pub fn build(topic: &Topic) -> Vec<Message> {
    let mut user = String::new();
    user.push_str("Write a short poem about: ");
    user.push_str(&topic.seed);

    if let Some(ctx) = &topic.context {
        user.push_str("\n\nBackground (use as inspiration, do not quote):\n");
        user.push_str(ctx);
    }

    vec![
        Message { role: Role::System, content: SYSTEM_PROMPT.to_string() },
        Message { role: Role::User, content: user },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_system_then_user() {
        let topic = Topic { seed: "rain".into(), context: None };
        let msgs = build(&topic);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, Role::System);
        assert_eq!(msgs[1].role, Role::User);
        assert!(msgs[1].content.contains("rain"));
    }

    #[test]
    fn includes_context_when_present() {
        let topic = Topic {
            seed: "the moon landing".into(),
            context: Some("Apollo 11 landed in 1969.".into()),
        };
        let msgs = build(&topic);
        assert!(msgs[1].content.contains("Apollo 11"));
        assert!(msgs[1].content.contains("the moon landing"));
    }
}
