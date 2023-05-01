use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use serde::Deserialize;
use glob::glob;
use local_ip_address::local_ip;
use portpicker::pick_unused_port;
use uuid::Uuid;

const SCREENSHOT_PATH: &str = ".steam/steam/userdata/*/*/remote/*/screenshots/*.jpg";
const JSON_API_APPLIST: &str = "applist.json";
const DEFAULT_SCALE_FACTOR: f64 = 0.3;

#[derive(Debug, Deserialize)]
struct AppListRoute {
    applist: AppListApps,
}

#[derive(Debug, Deserialize)]
struct AppListApps {
    apps: Vec<AppListApp>,
}

#[derive(Debug, Deserialize)]
struct AppListApp {
    appid: u32,
    name: String,
}

pub struct App {
    images: HashMap<String, HashMap<String, Screenshot>>,
    steam_apps: HashMap<String, SteamApp>,
    app_ids: HashMap<String, String>,

    server_ip: String,
    base_url: String,
    home_dir: String,
    cache_dir: String,
    ss_search_path: String,

    pub scale_factor: f64,
    pub server_port: u16,
    pub applist_json_path: String,
    pub shared_image: Arc<Mutex<SharedScreenshot>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SteamApp {
    pub id: String,
    pub title: String,
}

impl std::fmt::Display for SteamApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.id.as_str() == "0" {
            write!(f, "ALL")
        } else {
            write!(f, "{} ({})", self.title, self.id)
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SharedScreenshot {
    pub filepath: Option<String>,
    pub filename: Option<String>,
    pub uuid: Option<String>,
}

impl SharedScreenshot {
    pub fn set_values(&mut self, filepath: String, filename: String, uuid: String) {
        self.filepath = Some(filepath);
        self.filename = Some(filename);
        self.uuid = Some(uuid);
    }

    pub fn clear(&mut self) {
        self.filepath = None;
        self.filename = None;
        self.uuid = None;
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Screenshot {
    pub filepath: String,
    pub filename: String,
    pub app_id: String,
}

impl std::fmt::Display for Screenshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.filename)
    }
}

pub struct Screenshots {
    pub sorted_by_app: HashMap<String, Vec<Screenshot>>,
    pub sorted_all: Vec<Screenshot>,
}

fn get_app_id(filepath: &String) -> String {
    Path::new(filepath)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap()
}

fn get_filename(filepath: &String) -> String {
    Path::new(filepath)
        .file_name()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap()
}

impl App {
    pub fn new() -> Self {
        let shared_image = Arc::new(
            Mutex::new(SharedScreenshot::default()));
        Self {
            images: HashMap::new(),
            steam_apps: HashMap::new(),
            app_ids: HashMap::new(),
            scale_factor: DEFAULT_SCALE_FACTOR,

            server_ip: String::from(""),
            server_port: 0,
            base_url: String::from(""),
            shared_image,

            home_dir: String::from(""),
            cache_dir: String::from(""),
            ss_search_path: String::from(""),
            applist_json_path: String::from(""),
        }
    }

    pub fn check_env(&mut self) {
        let mut home = String::from("");
        let mut scale_factor_text = String::from("");

        dotenv::dotenv().ok();
        let envs: Vec<(String, String)> = dotenv::vars().collect();
        for (key, value) in envs {
            match key.as_str() {
                "HOMEDIR" => self.home_dir = value,
                "CACHEDIR" => self.cache_dir = value,
                "SCALE_FACTOR" => scale_factor_text = value,
                "HOME" => home = value,
                _ => {},
            }
        }

        if !scale_factor_text.is_empty() {
            let scale_factor = scale_factor_text.parse::<f64>();
            if scale_factor.is_ok() {
                self.scale_factor = scale_factor.unwrap();
            }
        }

        if self.home_dir.is_empty() {
            self.home_dir = home;
        }

        if self.home_dir.is_empty() {
            panic!("HOME not defined.");
        }

        let home_dir_path = Path::new(&self.home_dir);

        if !home_dir_path.is_dir() {
            panic!("HOMEDIR is not directory.");
        }

        if self.cache_dir.is_empty() {
            let cache_dir_path = home_dir_path
                .join(".cache")
                .join("sharess");

            if !cache_dir_path.is_dir() {
                fs::create_dir_all(cache_dir_path.clone())
                    .expect("Failed to create CACHEDIR.");
            }

            self.cache_dir = cache_dir_path
                .into_os_string()
                .into_string()
                .unwrap();
        }

        println!("HOMEDIR: {}", self.home_dir);
        println!("CACHEDIR: {}", self.cache_dir);
    }

    fn prepare_server(&mut self) {
        self.server_ip = local_ip().unwrap().to_string();
        self.server_port = pick_unused_port().unwrap();
        self.base_url = format!("http://{}:{}/", self.server_ip, self.server_port);
    }

    pub fn post_fetch(&mut self) {
        self.load_app_ids();
        self.gather_images();
        self.gather_apps();
    }

    pub fn setup(&mut self) {
        self.ss_search_path = Path::new(&self.home_dir)
            .join(SCREENSHOT_PATH)
            .into_os_string()
            .into_string()
            .unwrap();

        self.applist_json_path = Path::new(&self.cache_dir)
            .join(JSON_API_APPLIST)
            .into_os_string()
            .into_string()
            .unwrap();

        self.prepare_server();
    }

    fn load_app_ids(&mut self) {
        let path = Path::new(&self.applist_json_path);

        let raw_json = fs::read_to_string(path)
            .expect("Unable to read file");

        let data: AppListRoute = serde_json::from_str(&raw_json).unwrap();

        for item in data.applist.apps.iter() {
            self.app_ids.insert(item.appid.to_string(), item.name.clone());
        }
    }

    fn gather_images(&mut self) {
        for item in glob(&self.ss_search_path)
                .expect("Failed to search screenshots") {
            let filepath = item
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap();

            let app_id = get_app_id(&filepath);
            let filename = get_filename(&filepath);
            let image = Screenshot {
                filepath,
                filename: filename.clone(),
                app_id: app_id.clone(),
            };

            if !self.images.contains_key(&app_id) {
                let images: HashMap<String, Screenshot> = HashMap::new();
                self.images.insert(app_id.clone(), images);
            }

            let images = self.images.get_mut(&app_id).unwrap();
            images.insert(filename, image);
        }
    }

    fn gather_apps(&mut self) {
        for app_id in self.images.keys() {
            let title = if self.app_ids.contains_key(app_id) {
                self.app_ids.get(app_id).clone().unwrap().clone()
            } else {
                String::from("None")
            };

            let app = SteamApp {
                id: app_id.clone(),
                title: title.clone(),
            };

            self.steam_apps.insert(app_id.clone(), app);
        }
    }

    pub fn get_images(&self) -> Screenshots {
        let mut sorted_by_app: HashMap<String, Vec<Screenshot>> = HashMap::new();
        let mut sorted_all = Vec::new();

        for (app_id, images_by_name) in self.images.iter() {
            let mut images: Vec<Screenshot> = images_by_name
                .values()
                .cloned()
                .collect();
            sorted_all.extend(images.clone());
            images.sort_by_key(|image| Reverse(image.filename.clone()));
            sorted_by_app.insert(app_id.clone(), images);
        }

        sorted_all.sort_by_key(|image| Reverse(image.filename.clone()));

        Screenshots {
            sorted_by_app,
            sorted_all,
        }
    }

    pub fn get_steam_apps(&self) -> Vec<SteamApp> {
        let mut apps: Vec<SteamApp> = self.steam_apps
            .values()
            .cloned()
            .collect();
        apps.sort_by(|a, b| a.to_string()
            .partial_cmp(&b.to_string())
            .unwrap());
        apps
    }

    pub fn share(&mut self, screenshot: Screenshot) -> String {
        let filepath = screenshot.filepath.clone();
        let filename = screenshot.filename.clone();
        let image_uuid = Uuid::new_v4().to_string();
        self.shared_image.lock().unwrap().set_values(
            filepath,
            filename,
            image_uuid.clone());
        format!("{}{}", self.base_url, image_uuid)
    }

    pub fn stop_share(&mut self) {
        self.shared_image.lock().unwrap().clear();
    }
}