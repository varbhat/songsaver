use std::{cmp::min, fmt, path::Path};

use anyhow::Context;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use tabled::Tabled;
use tokio::io::AsyncWriteExt;
use url::Url;

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct Performer {
    name: String,
    id: i64,
}

impl fmt::Display for Performer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Serialize, Deserialize, Tabled)]
pub struct TrackItem {
    pub title: String,
    pub id: i64,
    pub isrc: String,
    pub performer: Performer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TracksResp {
    pub items: Vec<TrackItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlavartSearchResp {
    pub query: String,
    pub tracks: TracksResp,
}

pub async fn slavart_search(query: &str) -> anyhow::Result<SlavartSearchResp> {
    let mut slavart_search_uri = Url::parse("https://slavart.gamesdrive.net/api/search")?;
    slavart_search_uri.query_pairs_mut().append_pair("q", query);
    let slavart_search_uri = slavart_search_uri.to_string();
    let resp = reqwest::get(slavart_search_uri).await?;
    let resp = resp.json::<SlavartSearchResp>().await?;
    Ok(resp)
}

pub async fn slavart_fetch_track(track_id: i64, file_path: &Path) -> anyhow::Result<()> {
    let mut slavart_track_dl_uri =
        Url::parse("https://slavart-api.gamesdrive.net/api/download/track")?;
    slavart_track_dl_uri
        .query_pairs_mut()
        .append_pair("id", track_id.to_string().as_str());
    let slavart_track_dl_uri = slavart_track_dl_uri.to_string();
    let slavart_track_dl_uri: &str = &slavart_track_dl_uri;

    let mut targurl = "https://slavart.gamesdrive.net/api/search/q=".to_string();
    targurl.push_str(track_id.to_string().as_str());

    let reqwest_client = reqwest::Client::new();

    let resp = reqwest_client.get(slavart_track_dl_uri).send().await?;
    let total_size = resp.content_length().context(format!(
        "Failed to get content length from: {}",
        slavart_track_dl_uri
    ))?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar().template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?);
    pb.set_message(format!("Downloading {} to {:?}", slavart_track_dl_uri, file_path.to_str()));

    let mut target_file = tokio::fs::File::create(file_path).await?;
    let mut downloaded = 0;
    let mut dl_stream = resp.bytes_stream();

    while let Some(dl_item) = dl_stream.next().await {
        let dl_chunk = dl_item?;
        target_file.write_all(&dl_chunk).await?;
        downloaded = min(downloaded + (dl_chunk.len() as u64), total_size);
        pb.set_position(downloaded);
    }

    pb.finish_with_message(format!(
        "Downloaded {} to {:?} ",
        slavart_track_dl_uri,
        file_path.to_str()
    ));

    Ok(())
}
