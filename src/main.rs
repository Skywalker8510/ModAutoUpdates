use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde_json::Value;

#[tokio::main]
async fn main() {

    let mut file = File::open("./src/default.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let v: Value = serde_json::from_str(&data).unwrap();
    let mod_jar = Some(v["target_path"].as_str());
    let mod_id = match get_id(mod_jar.unwrap().unwrap()) {
        Some(id) => id,
        None => panic!("No ID found"),
    };
    
    let client = reqwest::Client::new();
    let body = client.get("https://api.modrinth.com/v2/search").query(&[("query", mod_id)]).send().await.unwrap().text().await.unwrap();

    let v: Value = serde_json::from_str(&body).unwrap();

    println!("{}", v["hits"][0]["project_id"]);
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
