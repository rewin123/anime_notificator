
pub mod anime_search_engine;

use std::sync::Arc;

use indicatif::{ProgressBar};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use teloxide::types::ChatId;
use tokio::sync::Mutex;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BigAnime {
    pub name : String,
    pub desc : String,
    pub url : String,
    pub episode : i32
}

impl PartialEq for BigAnime {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub id : ChatId,
    pub all_anime_updates : bool,
    pub animu_sub : Vec<String>,
    pub last_finded : String
}



#[derive(Clone)]
pub struct Server {
    pub anime_ongoings : Arc<Mutex<anime_search_engine::AnimeSearchEngine>>,
    pub db : Arc<Mutex<pickledb::PickleDb>>
}

impl Default for Server {
    fn default() -> Self {
        let db;
        let path = "db.db";
        match pickledb::PickleDb::load_json(path, pickledb::PickleDbDumpPolicy::AutoDump)
        {
            Ok(db_loaded) => {
                db = db_loaded;
            }
            _ => {
                db = pickledb::PickleDb::new_json(path, pickledb::PickleDbDumpPolicy::AutoDump);
            }
        }

        Self { 
            anime_ongoings : Arc::new(Mutex::new(anime_search_engine::AnimeSearchEngine::default())),
            db : Arc::new(Mutex::new(db))
        }
    }
}

impl Server {
    pub async fn load_vec<T>(
            &self, 
            key : &str) 
            -> Vec<T> where T : DeserializeOwned {

        let db = self.db.lock().await;
        match db.get(key.to_string().as_str()) {
            Some(res) => {
                res
            }
            None => {
                vec![]
            }
        }
    }

    pub async fn save_vec<T>(
        &self, 
        key : &str,
        v : &Vec<T>)
        where T : Serialize {

        let mut db = self.db.lock().await;
        db.set(key.to_string().as_str(), v).unwrap()
    }

    pub async fn get_user(&self, id : ChatId) -> User {
        let users = self.load_vec::<User>("users").await;
        match users.iter().position(|r| r.id == id) {
            Some(user) => {users[user].clone()}
            None => {User { id: id, all_anime_updates: false, animu_sub: vec![], last_finded: "".to_string() } }
        }
    }

    pub async fn set_user(&self, user : &User) {
        let mut users = self.load_vec::<User>("users").await;
        match users.iter().position(|r| r.id == user.id) {
            Some(user_idx) => {users[user_idx] = user.clone();}
            None => {
                users.push(user.clone());
            }
        }
        self.save_vec("users", &users).await;
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl BigAnime {
    pub fn from_page(url : &String, page_text : &String) -> BigAnime {
        let desc_re = regex::Regex::new(r#"full-text clearfix"><p>(.+)<.p>"#).unwrap();
        let name_re = regex::Regex::new(r#"anime__title">\n\s+<h1>(.+)<.h1>"#).unwrap();

        let page_str = page_text.as_str();
        let name = name_re.captures(page_str).unwrap()[1].to_string();
        let desc = desc_re.captures(page_str).unwrap()[1].to_string();

        Self {
            name,
            desc,
            url : url.clone(),
            episode : 0
        }
    }
}

async fn load_urls(url_vec : &Vec<String>) -> Vec<String> {
    let mut res = vec![];
    
    let mut pb = ProgressBar::new(url_vec.len() as u64);

    let mut load_tasks = vec![];

    for url in url_vec {
        
        let url_clone = url.clone();
        load_tasks.push(tokio::spawn(async move {
            reqwest::get(url_clone).await.unwrap().text().await.unwrap()
        }));
    }

    for task in load_tasks {
        res.push(task.await.unwrap());
    }

    res
}

async fn parse_ongoing_urls() -> Vec<String> {
    let result = reqwest::get("https://yummyanime.org/ongoing/").await.unwrap().text().await.unwrap();
    
    let page_regex = regex::Regex::new(r#"page.(\d+)"#).unwrap();

    let mut max_page = 0;

    for pat in page_regex.captures_iter(result.as_str()) {
        let page_num = *(&pat[1].parse::<i32>().unwrap());
        if page_num > max_page {
            max_page = page_num;
        }
    }

    let mut res = vec!["https://yummyanime.org/ongoing/".to_string()];
    for i in 2..(max_page + 1) {
        res.push(
            format!("https://yummyanime.org/ongoing/page/{}", i)
        );
    }

    res
}

pub fn parse_anime_names_from_page(text : &String) -> Vec<String> {
    let mut res = vec![];

    let re = 
        regex::Regex::new(r#""poster grid-item d-flex fd-column has-overlay".+href="(.+)">"#)
        .unwrap();

    for cap in re.captures_iter(text.as_str()) {
        res.push(cap[1].to_string());
    }

    res
}

pub async fn parse_ongoings() -> Vec<BigAnime> {
    let ongoing_pages_url = parse_ongoing_urls().await;
    let ongoung_pages = load_urls(&ongoing_pages_url).await;

    let mut anime_urls = vec![];
    for page in &ongoung_pages {
        anime_urls.append(&mut parse_anime_names_from_page(page));
    }

    let pb = ProgressBar::new(anime_urls.len() as u64);

    let mut animes = vec![];
    let mut animes_task = vec![];

    

    for url in anime_urls {
        animes_task.push(tokio::spawn(async move {
            let page = reqwest::get(url.clone()).await.unwrap().text().await.unwrap();
            let anime = BigAnime::from_page(&url, &page);
            anime
        }));
    }

    for task in animes_task {
        let anime = task.await.unwrap();
        pb.println(format!("Anime: {}", anime.name));
        animes.push(anime);
        pb.inc(1);
    }

    animes
}

pub async fn update_serialized_ongoings() {
    let animes = parse_ongoings().await;
    let animes_serialized = serde_json::to_string(&animes).unwrap();
    std::fs::write("loaded_ongoings.json", animes_serialized).expect("Cannot save serialized ongoings");
}

pub async fn load_last_anime() -> Vec<BigAnime> {
    let page = reqwest::get("https://yummyanime.org/").await.unwrap()
        .text().await.unwrap();

    let re = regex::Regex::new(r#"<a href="(.+)"> .+\n.+\n.+\n.+\n.+\n.+\n\s+(.+)<br>.+\n.+\n\s+<span class="cell cell-2">(\d+)"#).unwrap();

    let mut res = vec![];

    for cap in re.captures_iter(page.as_str()) {
        res.push(
            BigAnime {
                name : cap[2].to_string(),
                desc : "".to_string(),
                url : format!("https://yummyanime.org{}",cap[1].to_string()).to_string(),
                episode : cap[3].parse().unwrap()
            }
        )
    }

    res
}