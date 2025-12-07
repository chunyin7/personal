use askama::Template;
use axum::{Router, response::Html, routing::get};
use chrono::{DateTime, TimeZone, Utc};
use tower_http::services::ServeDir;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    age: u32,
}

async fn index() -> Html<String> {
    let template = IndexTemplate {
        age: Utc::now()
            .years_since(Utc.with_ymd_and_hms(2005, 4, 24, 0, 0, 0).unwrap())
            .unwrap()
            .into(),
    };
    Html(template.render().unwrap())
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/assets", ServeDir::new("assets"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
