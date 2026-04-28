use rust_poet::topic::TopicSource;
use rust_poet::topic::wikipedia::WikipediaOnThisDay;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn parses_event_into_topic() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/04/27"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "events": [
                {
                    "text": "Magellan crossed the Mactan strait.",
                    "year": 1521,
                    "pages": [
                        {"title": "Battle of Mactan", "extract": "A naval battle..."}
                    ]
                }
            ]
        })))
        .mount(&server)
        .await;

    let source = WikipediaOnThisDay::with_base_url_and_date(
        server.uri(),
        chrono::NaiveDate::from_ymd_opt(2026, 4, 27).unwrap(),
        Some(123),
    );
    let topic = source.next_topic().await.unwrap();
    assert!(topic.seed.contains("1521"));
    assert!(topic.seed.contains("Magellan"));
    assert!(topic.context.is_some());
}

#[tokio::test]
async fn empty_events_is_no_events_for_date_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/04/27"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"events": []})))
        .mount(&server)
        .await;

    let source = WikipediaOnThisDay::with_base_url_and_date(
        server.uri(),
        chrono::NaiveDate::from_ymd_opt(2026, 4, 27).unwrap(),
        Some(0),
    );
    let err = source.next_topic().await.unwrap_err();
    assert!(matches!(err, rust_poet::topic::TopicError::NoEventsForDate));
}
