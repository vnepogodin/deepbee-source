use std::cmp::min;
use std::fs;
use std::io::Write;
use std::path::Path;

use reqwest::Client;
use indicatif::{ProgressBar, ProgressStyle};
use futures_util::StreamExt;
use anyhow::{Context, Result};

const URLS_PATH: &str = "../resources/images_urls.csv";
const OUTPUT_PATH: &str = "../../DeepBee/original_images";

struct FileEntry {
    filename: String,
    file_url: String,
}

async fn download_file(client: &Client, url: &str, path: &str) -> Result<()> {
    // Reqwest setup
    let res = client
        .get(url)
        .send()
        .await.with_context(|| format!("Failed to GET from '{}'", &url))?;
    let total_size = res
        .content_length().with_context(|| format!("Failed to get content length from '{}'", &url))?;
    
    // Indicatif setup
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
        .progress_chars("#>-"));
    pb.set_message(format!("Downloading {}", url));

    // download chunks
    let mut file = fs::File::create(path).with_context(|| format!("Failed to create file '{}'", path))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.expect("Error while downloading file");
        file.write_all(&chunk).expect("Error while writing to file");
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(format!("Downloaded {} to {}", url, path));
    return Ok(());
}

#[tokio::main]
async fn main() -> Result<()> {
    if !Path::new(OUTPUT_PATH).exists() {
        fs::create_dir_all(OUTPUT_PATH)?;
    }
    //let client = reqwest::Client::builder().build()?;

    let mut file_entries: Vec<FileEntry> = vec![];

    {
        let mut rdr = csv::Reader::from_path(URLS_PATH)?;
        for result in rdr.records() {
            let record = result?;

            let filename = &record[0];
            let file_url = &record[1];
            //println!("{:?}", record);
            file_entries.push(FileEntry { filename: filename.to_owned(), file_url: file_url.to_owned() });
        }
    }

    let fetches = futures::stream::iter(
    file_entries.into_iter().map(|file_entry| {
        async move {
            let client = reqwest::Client::builder().build().unwrap();
            let file_path = format!("{}/{}", OUTPUT_PATH, file_entry.filename);
            match download_file(&client, &file_entry.file_url, &file_path).await {
                Err(err) => println!("Failed to download: {:?}", err),
                Ok(_) => {},
            }
        }
    })).buffer_unordered(8).collect::<Vec<()>>();

    println!("Waiting...");
    fetches.await;

    Ok(())
}
