use reqwest::{Response, get};
use serde_json::Value;
use std::fs::{File, read_dir};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{fs, io};

#[tokio::main]
async fn main() {
    let mut file = File::open("./src/default.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let v: Value = serde_json::from_str(&data).unwrap();
    let folder_path = v["target_path"].as_str().unwrap();

    for file in read_dir(folder_path).unwrap() {
        let jar_path = file.unwrap().path();
        let fabricmod_id = get_id(jar_path.clone()).expect("No Id Found");

        let client = reqwest::Client::new();
        let search_result = client
            .get("https://api.modrinth.com/v2/search")
            .query(&[("query", fabricmod_id)])
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let v: Value = serde_json::from_str(&search_result).unwrap();

        let project_result = client
            .get(format!(
                "https://api.modrinth.com/v2/project/{}",
                v["hits"][0]["project_id"].as_str().unwrap()
            ))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let v: Value = serde_json::from_str(&project_result).unwrap();

        let version_id_array = v["versions"].as_array().unwrap();

        let version_results = client
            .get(format!(
                "https://api.modrinth.com/v2/version/{}",
                version_id_array[version_id_array.len() - 1]
                    .as_str()
                    .unwrap()
            ))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let v: Value = serde_json::from_str(&version_results).unwrap();

        println!("{}", v["files"][0]["url"].as_str().unwrap());

        download_and_replace(
            folder_path,
            get(v["files"][0]["url"].as_str().unwrap()).await.unwrap(),
            v,
            jar_path,
        )
        .await;
    }
}

fn get_id<P: AsRef<Path>>(path: P) -> Option<String> {
    let file = File::open(path).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();

    let file = archive.by_name("fabric.mod.json").ok()?;
    let v: Value = serde_json::from_reader(file).unwrap();

    Some(v["id"].to_string())
}

async fn download_and_replace(
    folder_path: &str,
    url: Response,
    v: Value,
    jar_path: PathBuf,
) {
    let mut out = File::create(format!(
        "./{}/{}",
        folder_path,
        v["files"][0]["filename"].as_str().unwrap()
    ))
    .unwrap();
    let body = url.text().await.expect("body invalid");
    io::copy(&mut body.as_bytes(), &mut out).expect("failed to copy content");
    fs::remove_file(jar_path).unwrap();
}
