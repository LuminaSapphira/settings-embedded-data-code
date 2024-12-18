#!/usr/bin/env rust-script
//!
//! ```cargo
//! [dependencies]
//! reqwest = { version = "0.12", features = ["blocking", "json"]}
//! serde = { version = "1", features = ["derive"]}
//! serde_json = "1"
//! toml = { version = "0.8" }
//! anyhow = "1"
//! pretty_env_logger = "0.5"
//! log = "0.4"
//! jiff = "0.1"
//!
use std::{
    cell::OnceCell,
    collections::HashSet,
    env,
    fs::{self, File},
    io::BufReader,
    usize,
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use serde_json::Value as JsonValue;

use jiff::Timestamp;

#[derive(Deserialize)]
struct ModPortalPage {
    pagination: ModPortalPagination,
    results: Vec<ModPortalResultItem>,
}

#[derive(Deserialize)]
struct ModPortalPagination {
    page_count: usize,
    page: usize,
}

#[derive(Deserialize)]
struct ModPortalResultItem {
    name: String,
}

#[derive(Deserialize)]
struct Properties {
    ignore: HashSet<String>,
    version: u16,
}

#[derive(Serialize)]
struct ModPortalInfoFinal {
    #[serde(flatten)]
    base: JsonValue,
    dependencies: Vec<String>,
    version: String,
}

#[derive(Serialize, Deserialize)]
struct CachedModNames {
    mods: Vec<String>,
}

fn scrape_mod_portal() -> anyhow::Result<Vec<String>> {
    log::info!("Scraping mod portal for mods");
    let mut mod_names = Vec::new();

    let max_pages = OnceCell::new();
    let mut current_page = 1usize;
    loop {
        let res = reqwest::blocking::get(format!(
            "https://mods.factorio.com/api/mods?page={current_page}"
        ))
        .context("Getting mod portal page")?;

        log::info!("Analyzing mod portal page {current_page}");

        let page = res
            .json::<ModPortalPage>()
            .context("Deserializing response")?;
        assert_eq!(
            page.pagination.page, current_page,
            "Current page is corrupted"
        );

        max_pages.get_or_init(|| page.pagination.page_count);
        for item in page.results {
            mod_names.push(item.name);
        }
        if current_page == *max_pages.get().unwrap() {
            break;
        } else {
            current_page += 1;
        }
    }

    if env::var("SEDC_CREATE_CACHE").is_ok() {
        create_mod_name_cache(mod_names.clone()).context("Creating mod name cache")?;
    }

    Ok(mod_names)
}

fn cached_mod_names() -> anyhow::Result<Vec<String>> {
    log::info!("Using cached mods list");
    let cached = toml::from_str::<CachedModNames>(
        &fs::read_to_string("cached-mods.toml").context("Reading cached-mods.toml")?,
    )
    .context("Deserializing TOML cached-mods.toml")?;
    Ok(cached.mods)
}

fn create_mod_name_cache(mods: Vec<String>) -> anyhow::Result<()> {
    log::info!("Creating mod cache");
    let string = toml::to_string(&CachedModNames { mods }).context("Serializing TOML")?;
    fs::write("cached-mods.toml", &string).context("Writing cached.mods.toml")?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let properties = toml::from_str::<Properties>(
        &fs::read_to_string("properties.toml").context("Reading properties.toml")?,
    )
    .context("Deserializing TOML deps-ignore.toml")?;

    let mod_names = match env::var("SEDC_CACHED") {
        Ok(_) => cached_mod_names().context("Loading cached mods")?,
        Err(_) => scrape_mod_portal().context("Scraping mod portal")?,
    };

    let dependencies = mod_names
        .into_iter()
        .filter(|s| !properties.ignore.contains(s))
        .map(|mut s| {
            s.insert_str(0, "? ");
            s
        })
        .collect::<Vec<_>>();

    let json_base = serde_json::from_reader::<_, JsonValue>(BufReader::new(
        File::open("info-base.json").context("Opening info-base.json")?,
    ))
    .context("Deserializing JSON info-base.json ")?;

    let finalized = ModPortalInfoFinal {
        base: json_base,
        dependencies,
        version: format!(
            "{}.{}",
            properties.version,
            Timestamp::now().strftime("%y0%m.%d")
        ),
    };

    serde_json::to_writer_pretty(
        File::create("info.json").context("Creating info.json")?,
        &finalized,
    )
    .context("Serializing JSON info.json")?;

    Ok(())
}
