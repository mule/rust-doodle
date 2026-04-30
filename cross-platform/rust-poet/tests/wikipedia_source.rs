use rust_poet::topic::TopicSource;
use rust_poet::topic::wikipedia::WikipediaOnThisDay;
use wiremock::matchers::{header_regex, method, path};
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
    )
    .expect("client builds");
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
    )
    .expect("client builds");
    let err = source.next_topic().await.unwrap_err();
    assert!(matches!(err, rust_poet::topic::TopicError::NoEventsForDate));
}

#[tokio::test]
async fn sends_identifying_user_agent_header() {
    // Wikipedia 403s empty UAs. This test pins the format so a regression
    // in the UA constant or builder call is caught locally instead of in production.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/04/27"))
        .and(header_regex("user-agent", r"^rust-poet/\d+\.\d+\.\d+ \("))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "events": [{"text": "x", "year": 1, "pages": []}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let source = WikipediaOnThisDay::with_base_url_and_date(
        server.uri(),
        chrono::NaiveDate::from_ymd_opt(2026, 4, 27).unwrap(),
        Some(0),
    )
    .expect("client builds");
    source.next_topic().await.expect("request matched the UA matcher");
    // Mock's .expect(1) on drop verifies exactly one matching request was made.
}
