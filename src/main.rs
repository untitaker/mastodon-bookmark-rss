use axum::{
    error_handling::HandleErrorLayer,
    extract::{Host, Query},
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
use tower_governor::{
    errors::display_error, governor::GovernorConfigBuilder, key_extractor::KeyExtractor,
    GovernorError, GovernorLayer,
};

const MAX_UPSTREAM_RESPONSE_SIZE: usize = 5 * 1024 * 1024; // response from mastodon must be less than 5 MB

#[derive(Clone)]
struct ShowFeedExtractor;

impl KeyExtractor for ShowFeedExtractor {
    type Key = ShowFeed;

    fn extract<B>(
        &self,
        req: &axum::http::Request<B>,
    ) -> Result<ShowFeed, tower_governor::GovernorError> {
        let query = req.uri().query().unwrap_or_default();
        serde_urlencoded::from_str(query).map_err(|_| GovernorError::UnableToExtractKey)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Allow bursts with up to five requests per IP address
    // and replenishes one element every two seconds>
    let per_ip_governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .finish()
            .unwrap(),
    );

    // Allow bursts with up to 10 requests per feed (=(host, token))
    // and replenishes one element every 10 minutes.
    let per_feed_governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .key_extractor(ShowFeedExtractor)
            .per_second(600)
            .burst_size(10)
            .finish()
            .unwrap(),
    );

    let rate_limit_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            tracing::error!("error while serving request: {}", e.to_string());
            display_error(e)
        }))
        // XXX: enforcing two governors one after the other is not really correct in case of
        // partial success, see https://github.com/antifuchs/governor/issues/167
        //
        // Layers are sorted by how large their retry-after header is: The longer rate limits
        // (larger `per_second` value) go first.
        .layer(GovernorLayer {
            config: Box::leak(per_feed_governor_conf),
        })
        .layer(GovernorLayer {
            config: Box::leak(per_ip_governor_conf),
        });

    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("index.html")) }))
        // If this line is failing compilation, you need to run 'yarn install && yarn build' to get your CSS bundle.
        .route(
            "/bundle.css",
            get(|| async {
                (
                    [("Content-Type", "text/css")],
                    include_str!("../build/bundle.css"),
                )
            }),
        )
        .route(
            "/bundle.js",
            get(|| async {
                (
                    [("Content-Type", "application/javascript")],
                    include_str!("../build/bundle.js"),
                )
            }),
        )
        .route("/feed", get(show_feed).route_layer(rate_limit_layer));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("request to mastodon instance failed: {0}")]
    UpstreamIO(#[from] reqwest::Error),
    #[error("parsing a datetime from mastodon failed: {0}")]
    UpstreamChrono(#[from] chrono::ParseError),
    #[error("failed to parse JSON response: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("mastodon response too large")]
    ResponseTooLarge,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let s = self.to_string();
        tracing::error!("error while serving request: {}", s);
        (StatusCode::INTERNAL_SERVER_ERROR, s).into_response()
    }
}

async fn show_feed(Query(params): Query<ShowFeed>, Host(host): Host) -> Result<Response, Error> {
    let url = format!("https://{}/api/v1/bookmarks", params.host);

    static HTTP_CLIENT: OnceCell<reqwest::Client> = OnceCell::new();
    let mut upstream_response = HTTP_CLIENT
        .get_or_init(reqwest::Client::new)
        .get(url)
        .header("Authorization", format!("Bearer {}", params.token))
        // axum::extract::Host can be forged, but it is the best thing that works out of the box
        // without extra work, and forgery is not really part of any threat model for us anyway.
        .header(
            "User-Agent",
            format!(
                "mastodon-bookmark-rss/{} (+https://{})",
                env!("CARGO_PKG_VERSION"),
                host
            ),
        )
        .timeout(Duration::from_secs(5))
        .send()
        .await?
        .error_for_status()?;

    let mut upstream_response_body = Vec::new();

    while let Some(chunk) = upstream_response.chunk().await? {
        if upstream_response_body.len() + chunk.len() > MAX_UPSTREAM_RESPONSE_SIZE {
            return Err(Error::ResponseTooLarge);
        }

        upstream_response_body.extend(chunk);
    }

    let upstream_response_body_parsed: Vec<UpstreamBookmark> =
        serde_json::from_slice(&upstream_response_body)?;

    let mut body = String::new();
    body.push_str(
        r#"<?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:dc="http://purl.org/dc/elements/1.1/">
        <channel>
        <title>Mastodon Bookmarks</title>
        <description></description>
        <link><![CDATA[https://"#,
    );

    body.push_str(&params.host);
    body.push_str("]]></link>");

    for bookmark in upstream_response_body_parsed {
        body.push_str("<item>");
        body.push_str("<pubDate>");
        let parsed = DateTime::parse_from_rfc3339(&bookmark.created_at)?;
        body.push_str(&parsed.to_rfc2822());
        body.push_str("</pubDate>");
        body.push_str("<guid><![CDATA[");
        body.push_str(&bookmark.url);
        body.push_str("]]></guid>");
        if let Some(ref card) = bookmark.card {
            body.push_str("<link><![CDATA[");
            body.push_str(&card.url);
            body.push_str("]]></link>");
            body.push_str("<title><![CDATA[");
            body.push_str(&card.title);
            body.push_str("]]></title>");
        } else {
            body.push_str("<link><![CDATA[");
            body.push_str(&bookmark.url);
            body.push_str("]]></link>");
            body.push_str("<title><![CDATA[");
            body.push_str(&bookmark.url);
            body.push_str("]]></title>");
        }
        if let Some(ref account) = bookmark.account {
            body.push_str("<dc:creator><![CDATA[@");
            body.push_str(&escape_for_cdata(&account.username));
            body.push_str("]]></dc:creator>");
        }

        body.push_str("<content:encoded><![CDATA[");
        body.push_str("<p><a href=\"");
        body.push_str(&bookmark.url);
        body.push_str("\">Original Mastodon Post</a></p>");
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
    input.replace("&", "&amp;").replace("]]>", "")
}

#[derive(Deserialize, Hash, Eq, PartialEq, Debug, Clone)]
struct ShowFeed {
    host: String,
    token: String,
}

#[derive(Deserialize)]
struct UpstreamBookmark {
    #[serde(default)]
    card: Option<UpstreamCard>,
    account: Option<UpstreamAccount>,
    url: String,
    created_at: String,
    content: String,
}

#[derive(Deserialize)]
struct UpstreamAccount {
    #[serde(default)]
    username: String,
}

#[derive(Deserialize)]
struct UpstreamCard {
    #[serde(default)]
    title: String,
    #[serde(default)]
    url: String,
}
