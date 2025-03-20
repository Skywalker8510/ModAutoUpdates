use std::fs::{read_dir, File};
use std::{fs, io};
use std::io::Read;
use std::path::{Path, PathBuf};
use reqwest::{get, Response};
use serde_json::Value;

#[tokio::main]
async fn main() {

    let mut file = File::open("./src/default.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    
    let v: Value = serde_json::from_str(&data).unwrap();
    let folder_path = v["target_path"].as_str();
    
    for file in read_dir(folder_path.unwrap()).unwrap() {
        let jar_path = file.unwrap().path();
        let fabricmod_id = match get_id(jar_path.clone()) {
            Some(id) => id,
            None => panic!("No ID found"),
        };

        let client = reqwest::Client::new();
        let search_result = client.get("https://api.modrinth.com/v2/search").query(&[("query", fabricmod_id)]).send().await.unwrap().text().await.unwrap();

        let v: Value = serde_json::from_str(&search_result).unwrap();

        let project_result = client.get(format!("https://api.modrinth.com/v2/project/{}", v["hits"][0]["project_id"].as_str().unwrap())).send().await.unwrap().text().await.unwrap();

        let v: Value = serde_json::from_str(&project_result).unwrap();

        let version_id_array = v["versions"].as_array().unwrap();

        let version_results = client.get(format!("https://api.modrinth.com/v2/version/{}", version_id_array[version_id_array.len() - 1].as_str().unwrap())).send().await.unwrap().text().await.unwrap();

        let v: Value = serde_json::from_str(&version_results).unwrap();
        
        download_and_replace(folder_path, get(v["files"][0]["url"].as_str().unwrap()).await.unwrap(), v, jar_path).await;
    }
}

fn get_id<P: AsRef<Path>>(path: P) -> Option<String> {
    let file = File::open(path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let fabricmodjson_index = archive.index_for_name("fabric.mod.json");
    let mut file = archive.by_index(fabricmodjson_index?).unwrap();
    let mut fabric_json_content = String::new();
    file.read_to_string(&mut fabric_json_content).unwrap();
    
    let v: Value = serde_json::from_str(&fabric_json_content).unwrap();
    let final_string = Option::from(v["id"].to_string());
    
    final_string
}

async fn download_and_replace(folder_path: Option<&str>, url: Response, v: Value, jar_path: PathBuf) {
    let mut out = File::create(format!("./{}/{}",folder_path.unwrap(), v["files"][0]["filename"].as_str().unwrap())).unwrap();
    let body = url.text().await.expect("body invalid");
    io::copy(&mut body.as_bytes(), &mut out).expect("failed to copy content");
    fs::remove_file(jar_path).unwrap();
}