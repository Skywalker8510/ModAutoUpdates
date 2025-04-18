use futures_util::StreamExt;
use reqwest::{Client, get};
use serde_json::Value;
use std::fs::{File, read_dir};
use std::io::{Read, Write};
use std::path::Path;

#[tokio::main]
async fn main() {
    //TODO Change this to a TOML file instead of using JSON.
    //**********************************************************************
    let mut file = File::open("./src/default.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let v: Value = serde_json::from_str(&data).unwrap();
    let folder_path = v["target_path"].as_str().unwrap();
    //**********************************************************************

    for file in read_dir(folder_path).unwrap() {
        let jar_path = file.unwrap().path();
        let fabricmod_id = get_fabric_id(jar_path.clone()).expect("No Id Found");

        let client = Client::new();
        let api_search_result = get_api_search_result(client.clone(), fabricmod_id).await.unwrap();
        let api_project_result = get_api_project_result(client.clone(), api_search_result).await.unwrap();
        let api_version_result = get_api_version_result(client, api_project_result).await.unwrap();

        let download_url = api_version_result["files"][0]["url"].as_str().unwrap();
        let download_path = format!(
            "./{}{}",
            folder_path,
            api_version_result["files"][0]["filename"].as_str().unwrap()
        );

        download_files(download_url, &download_path).await.unwrap();
    }
}

fn get_fabric_id<P: AsRef<Path>>(path: P) -> Option<String> {
    let file = File::open(path).unwrap();

    let mut jar = zip::ZipArchive::new(file).unwrap();

    let file = jar.by_name("fabric.mod.json").ok()?;
    let fabricmod_json: Value = serde_json::from_reader(file).unwrap();

    Some(fabricmod_json["id"].to_string())
}

async fn download_files(url: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    let mut stream = get(url).await?.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk)?;
    }

    file.flush()?;
    Ok(())
}

async fn get_api_search_result(
    client: Client,
    fabricmod_id: String,
) -> Result<Value, Box<dyn std::error::Error>> {
    let search_result = client.get("https://api.modrinth.com/v2/search").query(&[("query", fabricmod_id)]).send().await?.text().await?;
    let api_search_results: Value = serde_json::from_str(&search_result)?;
    
    Ok(api_search_results)
}

async fn get_api_project_result(
    client: Client,
    api_search_results: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let project_search = client
        .get(format!(
            "https://api.modrinth.com/v2/project/{}",
            api_search_results["hits"][0]["project_id"]
                .as_str()
                .unwrap()
        ))
        .send()
        .await?
        .text()
        .await?;
    let api_project_results: Value = serde_json::from_str(&project_search)?;
    
    Ok(api_project_results)
}

async fn get_api_version_result(
    client: Client,
    api_project_results: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let version_id_array = api_project_results["versions"].as_array().unwrap();
    let version_search = client.get(format!("https://api.modrinth.com/v2/version/{}", version_id_array[version_id_array.len() - 1].as_str().unwrap())).send().await?.text().await?;
    let api_version_results: Value = serde_json::from_str(&version_search)?;
    
    Ok(api_version_results)
}