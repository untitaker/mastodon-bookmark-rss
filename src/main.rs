use axum::{
    error_handling::HandleErrorLayer,
    extract::{Query, Host},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    BoxError, Router,
};
use chrono::DateTime;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::{net::SocketAddr, time::Duration};

use tower::ServiceBuilder;
use tower_governor::{errors::display_error, governor::GovernorConfigBuilder, GovernorLayer};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Allow bursts with up to five requests per IP address
    // and replenishes one element every two seconds
    // We Box it because Axum 0.6 requires all Layers to be Clone
    // and thus we need a static reference to it
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .finish()
            .unwrap(),
    );

    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("index.html")) }))
        // If this line is failing compilation, you need to run 'yarn install && yarn build' to get your CSS bundle.
        .route("/bundle.css", get(|| async { ([("Content-Type", "text/css")], include_str!("bundle.css")) }))
        .route("/feed", get(show_feed))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|e: BoxError| async move {
                    display_error(e)
                }))
                .layer(GovernorLayer {
                    config: Box::leak(governor_conf),
                }),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("request to your mastodon instance failed: {0}")]
    UpstreamIO(#[from] reqwest::Error),
    #[error("parsing a datetime from mastodon failed")]
    UpstreamChrono(#[from] chrono::ParseError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

async fn show_feed(Query(params): Query<ShowFeed>, Host(host): Host) -> Result<Response, Error> {
    let url = format!("https://{}/api/v1/bookmarks", params.host);

    static HTTP_CLIENT: OnceCell<reqwest::Client> = OnceCell::new();
    let upstream_response: Vec<UpstreamBookmark> = HTTP_CLIENT
        .get_or_init(reqwest::Client::new)
        .get(url)
        .header("Authorization", format!("Bearer {}", params.token))

        // axum::extract::Host can be forged, but it is the best thing that works out of the box
        // without extra work, and forgery is not really part of any threat model for us anyway.
        .header("User-Agent", format!("mastodon-bookmark-rss/{} (+https://{})", env!("CARGO_PKG_VERSION"), host))
        .timeout(Duration::from_secs(5))
        .send()
        .await?
        // Perhaps limit the response body size here? But how.
        .error_for_status()?
        .json()
        .await?;

    // Why can't I use a generator here? Oh god.
    let mut body = String::new();
    body.push_str(
        r#"<?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0"
            xmlns:content="http://purl.org/rss/1.0/modules/content/"
            xmlns:wfw="http://wellformedweb.org/CommentAPI/"
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            xmlns:atom="http://www.w3.org/2005/Atom"
            xmlns:sy="http://purl.org/rss/1.0/modules/syndication/"
            xmlns:slash="http://purl.org/rss/1.0/modules/slash/">
        <channel>
        <title>Mastodon Bookmarks</title>
        <description></description>
        <link><![CDATA[https://"#,
    );

    body.push_str(&params.host);
    body.push_str("]]></link>");

    for bookmark in upstream_response {
        body.push_str("<item>");
        body.push_str("<link><![CDATA[");
        body.push_str(&bookmark.url);
        body.push_str("]]></link>");
        body.push_str("<pubDate>");
        let parsed = DateTime::parse_from_rfc3339(&bookmark.created_at)?;
        body.push_str(&parsed.to_rfc2822());
        body.push_str("</pubDate>");
        body.push_str("<guid><![CDATA[");
        body.push_str(&bookmark.url);
        body.push_str("]]></guid>");
        if let Some(ref card) = bookmark.card {
            body.push_str("<title><![CDATA[");
            body.push_str(&card.title);
            body.push_str("]]></title>");

            body.push_str("<description><![CDATA[");
            body.push_str(&escape_for_cdata(&card.description));
            body.push_str("]]></description>");
        } else {
            body.push_str("<title><![CDATA[");
            body.push_str(&bookmark.url);
            body.push_str("]]></title>");
        }

        body.push_str("<content:encoded><![CDATA[");
        body.push_str(&escape_for_cdata(&bookmark.content));
        body.push_str("]]></content:encoded>");
        body.push_str("</item>\n");
    }

    body.push_str("</channel></rss>");

    Ok((
        StatusCode::OK,
        // Use this nonstandard content type so that Firefox does not download the feed.
        [("Content-Type", "text/xml; charset=\"utf-8\"")],
        body,
    )
        .into_response())
}

fn escape_for_cdata(input: &str) -> String {
    // There do not appear to be any decent standalone crates for this.
    input
        .replace("&", "&amp;")
        .replace("]]>", "")
}

#[derive(Deserialize)]
struct ShowFeed {
    host: String,
    token: String,
}

#[derive(Deserialize)]
struct UpstreamBookmark {
    #[serde(default)]
    card: Option<UpstreamCard>,
    url: String,
    created_at: String,
    content: String,
}

#[derive(Deserialize)]
struct UpstreamCard {
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
}
