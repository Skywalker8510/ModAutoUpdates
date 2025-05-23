mod api_calls;

use crate::api_calls::{get_api_project_result, get_api_search_result, get_api_version_result};
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

    let config: Value = serde_json::from_str(&data).unwrap();
    let folder_path = config["target_path"].as_str().unwrap();
    //**********************************************************************

    for file in read_dir(folder_path).unwrap() {
        let jar_path = file.unwrap().path();
        let fabricmod_id = get_fabric_id(jar_path.clone()).expect("No Id Found");

        let client = Client::new();

        let project_id =
            match get_api_search_result(client.clone(), fabricmod_id, config.clone()).await {
                Ok(search_result) => {
                    if is_compatible(
                        config["loader_version"].clone(),
                        config["server_version"].clone(),
                        search_result["game_versions"].as_array().unwrap(),
                        search_result["loaders"].as_array().unwrap(),
                    ) {
                        search_result["hits"][0]["project_id"].to_string()
                    } else {
                        continue; //ToDo add information to console log
                    }
                }
                Err(_) => continue,
            };

        let version_id_array = match get_api_project_result(client.clone(), project_id).await {
            Ok(project_result) => match project_result["versions"].as_array() {
                Some(versions) => versions.to_vec(),
                None => continue,
            },
            Err(_) => continue,
        };

        let mut api_version_result_option = None;
        for result in version_id_array {
            api_version_result_option = match get_api_version_result(client.clone(), &result).await
            {
                Ok(version_result) => {
                    if is_compatible(
                        config["loader_version"].clone(),
                        config["server_version"].clone(),
                        version_result["game_versions"].as_array().unwrap(),
                        version_result["loaders"].as_array().unwrap(),
                    ) {
                        Option::from(version_result)
                    } else {
                        continue;
                    }
                }
                Err(_) => continue,
            };
            break;
        }

        if api_version_result_option.is_none() {
            continue;
        }

        let api_version_result = match api_version_result_option {
            Some(version_result) => version_result,
            None => continue,
        };

        let download_url = api_version_result["files"][0]["url"].as_str().unwrap();
        let download_path = format!(
            "./{}{}",
            folder_path,
            api_version_result["files"][0]["filename"].as_str().unwrap()
        );

        download_files(download_url, &download_path).await.unwrap();
    }
}

fn is_compatible(
    loader_version: Value,
    server_version: Value,
    game_version_array: &[Value],
    loader_version_array: &[Value],
) -> bool {
    loader_version_array.contains(&loader_version) && game_version_array.contains(&server_version)
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
