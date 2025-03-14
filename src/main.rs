use std::fs::{read_dir, File};
use std::io::Read;
use std::path::Path;
use serde_json::Value;

#[tokio::main]
async fn main() {

    let mut file = File::open("./src/default.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    
    let v: Value = serde_json::from_str(&data).unwrap();
    let folder_path = Some(v["target_path"].as_str());
    
    for file in read_dir(folder_path.unwrap().unwrap()).unwrap() {
        let jar_path = file.unwrap().path();
        let fabricmod_id = match get_id(jar_path) {
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

        println!("{}", v["files"][0]["url"].as_str().unwrap());
    }
}

fn get_id<P: AsRef<Path>>(path: P) -> Option<String> {
    let file = File::open(path).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();
    
    let mut final_string = None;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if file.name() != "fabric.mod.json" {
            continue;
        }
        let mut fabric_json_content = String::new();
        file.read_to_string(&mut fabric_json_content).unwrap();

        let v: Value = serde_json::from_str(&fabric_json_content).unwrap();
        
        final_string = Some(v["id"].to_string());
    }
    final_string
}
