use askama::Template;
use axum::{Router, response::Html, routing::get};
use chrono::{DateTime, Duration, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use tower_http::services::ServeDir;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    age: u32,
    work: Vec<Work>,
    projects: Vec<Project>,
    recent_tracks: Vec<DisplayTrack>,
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

#[derive(Clone)]
struct DisplayTrack {
    artist: String,
    image_url: String,
    time_ago: String,
    album: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LastFmResponse {
    recenttracks: RecentTracks,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct RecentTracks {
    #[serde(rename = "track")]
    tracks: Vec<Track>,
    #[serde(rename = "@attr")]
    attr: TrackAttributes,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TrackAttributes {
    user: String,
    totalPages: String,
    total: String,
    perPage: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Track {
    artist: Artist,
    streamable: String,
    #[serde(rename = "image")]
    images: Vec<Image>,
    mbid: String,
    album: Album,
    name: String,
    url: String,
    #[serde(default)]
    date: Option<Date>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Artist {
    mbid: String,
    #[serde(rename = "#text")]
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Image {
    size: String,
    #[serde(rename = "#text")]
    url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Album {
    mbid: String,
    #[serde(rename = "#text")]
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Date {
    uts: String,
    #[serde(rename = "#text")]
    formatted: String,
}

pub fn format_time_ago(uts: &str) -> String {
    let timestamp_seconds: i64 = match uts.parse() {
        Ok(t) => t,
        Err(_) => return "Invalid timestamp".to_string(),
    };

    let past_time: DateTime<Utc> = match Utc.timestamp_opt(timestamp_seconds, 0).single() {
        Some(dt) => dt,
        None => return "Invalid timestamp".to_string(),
    };

    let now: DateTime<Utc> = Utc::now();
    let duration: Duration = now.signed_duration_since(past_time);

    if duration.num_minutes() < 1 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{} mins ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hrs ago", duration.num_hours())
    } else {
        format!("{} days ago", duration.num_days())
    }
}

async fn get_listening() -> Option<Vec<DisplayTrack>> {
    dotenv::dotenv().ok();
    let api_key = match std::env::var("LAST_FM_KEY") {
        Ok(key) => key,
        Err(_) => return None,
    };

    let url = format!(
        "http://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user={}&api_key={}&format=json&limit=10",
        "kkyowa", &api_key
    );

    let client = reqwest::Client::new();
    let response = match client.get(&url).send().await {
        Ok(response) => response,
        Err(_) => return None,
    };

    if response.status().is_success() {
        let text = match response.text().await {
            Ok(text) => text,
            Err(_) => return None,
        };

        let parsed_data: LastFmResponse = match from_str(&text) {
            Ok(data) => data,
            Err(_) => return None,
        };

        let ret = parsed_data.recenttracks.tracks.iter().map(|track| {
            let album = track.album.name.clone();
            let artist = track.artist.name.clone();
            let image_url = track
                .images
                .iter()
                .find(|image| image.size == "small")
                .unwrap()
                .url
                .clone();
            let time_ago = format_time_ago(track.date.as_ref().unwrap().uts.as_str());
            let name = track.name.clone();

            DisplayTrack {
                artist,
                image_url,
                time_ago,
                album,
                name,
            }
        });

        Some(ret.collect())
    } else {
        None
    }
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
        work: load_work_experience(),
        projects: load_projects(),
        recent_tracks: get_listening().await.unwrap_or(vec![]),
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
