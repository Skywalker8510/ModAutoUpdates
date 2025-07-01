use reqwest::Client;
use serde_json::Value;

pub async fn get_api_search_result(
    client: Client,
    fabricmod_id: String,
    loader_version: String,
    server_version: String,
) -> Result<Value, Box<dyn std::error::Error>> {
    let search_result = client
        .get("https://api.modrinth.com/v2/search")
        .query(&[
            ("query", fabricmod_id),
            ("facets", format!("[[\"categories:{loader_version}\"],[\"versions:{server_version}\"]]"))])
        .send()
        .await?
        .text()
        .await?;
    let api_search_results: Value = serde_json::from_str(&search_result)?;

    Ok(api_search_results)
}

pub async fn get_api_project_result(
    client: Client,
    project_id: String,
) -> Result<Value, Box<dyn std::error::Error>> {
    let project_search = client
        .get(format!("https://api.modrinth.com/v2/project/{project_id}"))
        .send()
        .await?
        .text()
        .await?;
    let api_project_results: Value = serde_json::from_str(&project_search)?;

    Ok(api_project_results)
}

pub async fn get_api_version_result(
    client: Client,
    version_id: String,
) -> Result<Value, Box<dyn std::error::Error>> {
    let version_search = client
        .get(format!("https://api.modrinth.com/v2/version/{version_id}"))
        .send()
        .await?
        .text()
        .await?;
    let api_version_results: Value = serde_json::from_str(&version_search)?;

    Ok(api_version_results)
}
