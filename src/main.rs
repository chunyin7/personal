use askama::Template;
use axum::{Router, response::Html, routing::get};
use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;
use tower_http::services::ServeDir;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    age: u32,
    work: &'a [Work],
    projects: &'a [Project],
}

#[derive(Deserialize)]
struct ProjectsData {
    project: Vec<Project>,
}

#[derive(Deserialize, Clone)]
struct Project {
    name: String,
    description: String,
    link: String,
}

#[derive(Deserialize)]
struct Experience {
    experience: Vec<Work>,
}

#[derive(Deserialize, Clone)]
struct Work {
    title: String,
    company: String,
    start: String,
    end: String,
    description: String,
}

fn load_work_experience() -> Vec<Work> {
    let content = std::fs::read_to_string("data/work.toml").unwrap_or("".to_string());
    let experience: Experience = toml::from_str(&content).unwrap();
    experience.experience
}

fn load_projects() -> Vec<Project> {
    let content = std::fs::read_to_string("data/projects.toml").unwrap_or("".to_string());
    let projects: ProjectsData = toml::from_str(&content).unwrap();
    projects.project
}

async fn index() -> Html<String> {
    let template = IndexTemplate {
        age: Utc::now()
            .years_since(Utc.with_ymd_and_hms(2005, 4, 24, 0, 0, 0).unwrap())
            .unwrap()
            .into(),
        work: &load_work_experience()[..3],
        projects: &load_projects()[..3],
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
