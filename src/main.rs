mod api_calls;
mod config;

use crate::api_calls::{get_api_project_result, get_api_search_result, get_api_version_result};
use crate::config::Config;
use futures_util::StreamExt;
use reqwest::{Client, get};
use serde_json::Value;
use serde_json::Value::Null;
use std::fs::{File, read_dir};
use std::io;
use std::io::Write;
use std::path::Path;

#[tokio::main]
async fn main() {
    let config = Config::open(&"./settings.toml").expect("Could not open settings file");
    let folder = read_dir(config.target_path.clone()).unwrap();

    for file in folder {
        let jar_path = file.unwrap().path();
        let fabricmod_id = match get_fabric_id(jar_path) {
            Ok(id) => id,
            Err(_) => continue,
        };

        let client = Client::new();

        let project_id = match get_api_search_result(
            client.clone(),
            fabricmod_id.clone(),
            config.loader_version.clone(),
            config.server_version.clone(),
        )
        .await
        {
            Ok(search_result) => {
                if search_result["hits"][0]["versions"] == Null {
                    println!("No versions found for fabric project {fabricmod_id}");
                    continue;
                } else if is_compatable(
                    Value::String(config.loader_version.clone()),
                    Value::String(config.server_version.clone()),
                    search_result["hits"][0]["versions"].as_array().unwrap(),
                    None,
                ) {
                    let title = &search_result["hits"][0]["title"];
                    println!(
                        "Fabric project {title} found from mod {fabricmod_id} do you want to download it? (Y/N)"
                    );
                    let mut input: String = String::new();
                    io::stdin()
                        .read_line(&mut input)
                        .expect("Unable to read Stdin");
                    if input.trim() == "Y" || input.trim() == "y" {
                        println!("Continuing with download for {title}");
                    } else {
                        println!("Skipping download for {title}");
                        continue;
                    }

                    search_result["hits"][0]["project_id"]
                        .as_str()
                        .unwrap()
                        .to_string()
                } else {
                    continue; //ToDo add information to console log
                }
            }
            Err(_) => continue,
        };

        let mut version_id_array = match get_api_project_result(client.clone(), project_id).await {
            Ok(project_result) => match project_result["versions"].as_array() {
                Some(versions) => versions.to_vec(),
                None => continue,
            },
            Err(_) => continue,
        };

        version_id_array.reverse();

        let mut api_version_result: Value = Null;
        for version_id in version_id_array {
            match get_api_version_result(client.clone(), version_id.as_str().unwrap().to_string())
                .await
            {
                Ok(version_result) => {
                    if is_compatable(
                        Value::String(config.loader_version.clone()),
                        Value::String(config.server_version.clone()),
                        version_result["game_versions"].as_array().unwrap(),
                        Some(version_result["loaders"].as_array().unwrap()),
                    ) {
                        api_version_result = version_result;
                        break;
                    } else {
                        continue;
                    }
                }
                Err(_) => continue,
            };
        }

        let download_url = match api_version_result["files"][0]["url"].as_str() {
            Some(url) => url,
            None => continue,
        };

        let filename = match api_version_result["files"][0]["filename"].as_str() {
            Some(filename) => filename,
            None => continue,
        };

        let download_path = format!("./{}/{}", config.target_path.clone().display(), filename);

        match download_files(download_url, &download_path).await {
            Ok(_) => println!("file {filename} downloaded successfully!"),
            Err(_) => continue,
        };
    }
}

fn is_compatable(
    loader_version: Value,
    server_version: Value,
    game_version_array: &[Value],
    loader_version_array: Option<&[Value]>,
) -> bool {
    match loader_version_array {
        Some(loader_version_array) => {
            loader_version_array.contains(&loader_version)
                && game_version_array.contains(&server_version)
        }
        None => game_version_array.contains(&server_version),
    }
}

fn get_fabric_id<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut jar = zip::ZipArchive::new(file)?;
    let file = jar.by_name("fabric.mod.json")?;
    let fabricmod_json: Value = serde_json::from_reader(file)?;

    Ok(fabricmod_json["id"].to_string())
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
