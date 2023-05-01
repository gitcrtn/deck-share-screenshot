use std::path::Path;

const URL_GET_APP_LIST: &str = "https://api.steampowered.com/ISteamApps/GetAppList/v2/";

pub async fn fetch_applist(filepath: String) {
    let applist_path = Path::new(&filepath);

    if applist_path.is_file() {
        return;
    }

    let res = reqwest::get(URL_GET_APP_LIST).await;

    if res.is_err() {
        return;
    }

    let body = res.unwrap().text().await;

    if body.is_err() {
        return;
    }

    std::fs::write(applist_path, body.unwrap()).unwrap();
}