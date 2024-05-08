use crate::enums::Collection;
use log::info;
use resvg::tiny_skia;
use resvg::usvg::{self, fontdb};
use std::collections::HashMap;
use std::error::Error;
use std::io::copy;
use viuer::{print_from_file, Config};

use std::fs::{create_dir_all, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::{path::PathBuf, process::exit};

use enums::IconCollection;
use home::home_dir;
use log::error;

mod enums;

pub fn get_icon_xml(icon_identifier: &str) -> Result<(usize, usize, String), Box<dyn Error>> {
    let Some((collection_id, icon_identifier)) = icon_identifier.split_once(':') else {
        todo!();
    };

    let collection = get_collection(collection_id)?;
    if let Some(icon) = collection.icons.get(icon_identifier) {
        Ok((
            collection.width.unwrap_or(16),
            collection.height.unwrap_or(16),
            icon.body.clone(),
        ))
    } else {
        Err("Could not find icon.".into())
    }
}

pub fn preview(icon_identifier: &str) -> Result<(), Box<dyn Error>> {
    let mut file = Vec::new();

    let (width, height, xml) = get_icon_xml(icon_identifier)?;

    let header = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="96" height="96" color="white" viewBox="0 0 {} {}">"#,
        width, height
    );
    let footer = r#"</svg>"#;

    let xml = xml.replace("stroke=\"#000\"", "stroke=\"#fff\"");

    let _ = file.write_all(header.as_bytes());
    let _ = file.write_all(xml.as_bytes());
    let _ = file.write_all(footer.as_bytes());

    let in_file = file.as_slice();
    let out_file = "/tmp/icon-rs-preview.png";

    let tree = {
        let opt = usvg::Options::default();
        let mut fontdb = fontdb::Database::new();
        fontdb.load_system_fonts();
        usvg::Tree::from_data(&in_file, &opt, &fontdb).unwrap()
    };

    let pixmap_size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    pixmap.save_png(out_file).unwrap();

    let conf = Config {
        absolute_offset: false,
        x: 0,
        y: 0,
        width: Some(6),
        restore_cursor: false,
        ..Default::default()
    };

    print_from_file("/tmp/icon-rs-preview.png", &conf).expect("Image printing failed.");

    Ok(())
}

pub fn get_home_dir() -> PathBuf {
    if let Some(path) = home_dir() {
        path
    } else {
        error!("ERROR: Unable to get home dir.");
        exit(1)
    }
}

pub fn fetch_icons_in_collection(collection_id: &str) -> Result<IconCollection, Box<dyn Error>> {
    // If we've already got this collection's icons locally, just return it.
    if let Ok(collection) = get_collection(collection_id) {
        return Ok(collection);
    }

    let url = format!(
        "https://raw.githubusercontent.com/iconify/icon-sets/master/json/{}.json",
        collection_id
    );

    let response = reqwest::blocking::get(url)?.text()?;

    let home_dir = get_home_dir();
    let path = home_dir.join(".local/share/icon-rs/cache/collections");

    create_dir_all(&path.as_path())?;

    let filename = format!("{}.json", collection_id);
    let full_path = path.join(filename);
    let mut dest = File::create(&full_path)?;
    copy(&mut response.as_bytes(), &mut dest)?;

    info!("  {}", full_path.display());

    let result: IconCollection = serde_json::from_str(&response)?;

    Ok(result)
}

pub fn write_bytes_to_file_in_home_dir(
    path: &str,
    filename: &str,
    data: &[u8],
) -> Result<(), Box<dyn Error>> {
    let home_dir = get_home_dir();
    let full_path = home_dir.join(path);

    create_dir_all(full_path.clone())?;

    let file_path = full_path.join(filename);

    let mut dest = File::create(&file_path)?;
    let result = dest.write_all(data)?;

    Ok(result)
}

pub fn write_iterator_to_file_in_home_dir<I>(
    path: &str,
    filename: &str,
    iterator: I,
) -> Result<(), Box<dyn Error>>
where
    I: IntoIterator<Item = String>,
{
    let home_dir = get_home_dir();
    let full_path = home_dir.join(path);

    create_dir_all(full_path.clone())?;

    let file_path = full_path.join(filename);

    let dest = File::create(&file_path)?;

    let mut buf = BufWriter::new(dest);

    for val in iterator.into_iter() {
        writeln!(buf, "{}", val)?;
    }

    Ok(())
}

pub fn fetch_collections(force: bool) -> Result<Vec<String>, Box<dyn Error>> {
    if let Ok(collection_ids) = get_collection_ids() {
        if force {
            info!("Force fetching collections..");
        } else {
            return Ok(collection_ids);
        }
    } else {
        info!("No cached collections found. Fetching collections..");
    };

    info!("Downloading collections..");
    let response = reqwest::blocking::get(
        "https://raw.githubusercontent.com/iconify/icon-sets/master/collections.json",
    )?
    .text()?;
    info!("Downloaded collections..");

    write_bytes_to_file_in_home_dir(
        ".local/share/icon-rs/cache",
        "collections.json",
        response.as_bytes(),
    )?;

    info!("Parsing collections..");
    let collections: HashMap<String, Collection> = serde_json::from_str(&response)?;
    info!("Parsed collections..");

    let collection_ids: Vec<String> = collections.iter().map(|(k, _)| k.to_string()).collect();

    info!("Writing collections file..");
    write_iterator_to_file_in_home_dir(
        ".local/share/icon-rs/cache",
        "collection_ids.txt",
        collection_ids.clone(),
    )?;
    info!("Wrote collections file..");

    Ok(collection_ids)
}

pub fn get_collection_ids() -> Result<Vec<String>, Box<dyn Error>> {
    let file_path = get_home_dir().join(".local/share/icon-rs/cache/collection_ids.txt");
    let reader = BufReader::new(File::open(file_path)?);
    let mut result = Vec::<String>::new();

    for line in reader.lines().flatten() {
        result.push(line);
    }

    Ok(result)
}

pub fn get_collection(collection_id: &str) -> Result<IconCollection, Box<dyn Error>> {
    let path = get_home_dir().join(format!(
        ".local/share/icon-rs/cache/collections/{}.json",
        collection_id
    ));

    let reader = BufReader::new(File::open(path)?);
    let result: IconCollection = serde_json::from_reader(reader)?;

    Ok(result)
}

pub fn get_cached_icons() -> Result<Vec<String>, Box<dyn Error>> {
    let path = get_home_dir().join(".local/share/icon-rs/cache/icons.txt");

    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        let mut result = Vec::<String>::new();

        for line in reader.lines().flatten() {
            result.push(line);
        }

        Ok(result)
    } else {
        generate_cached_icons()
    }
}

pub fn generate_cached_icons() -> Result<Vec<String>, Box<dyn Error>> {
    info!("Generating icons cache..");
    let collections = fetch_collections(false)?;

    let mut icons = Vec::new();
    for collection in collections {
        info!("  - {}", collection);
        let icons_in_collection = fetch_icons_in_collection(&collection)?;

        for (icon, _) in icons_in_collection.icons {
            icons.push(format!("{}:{}", collection, icon));
        }
    }

    write_iterator_to_file_in_home_dir(".local/share/icon-rs/cache", "icons.txt", icons)?;

    get_cached_icons()
}

pub fn query(
    query: &Option<String>,
    prefix: &Option<String>,
    preview_inline: bool,
) -> Result<Vec<String>, Box<dyn Error>> {
    let icons = get_cached_icons()?;

    let found: Vec<String> = icons
        .iter()
        .filter(|i| {
            let matching = if let Some(query) = query {
                i.contains(query)
            } else {
                true
            };

            if let Some(prefix) = &prefix {
                matching && i.starts_with(&format!("{}:", prefix))
            } else {
                matching
            }
        })
        .map(String::from)
        .collect();

    if preview_inline {
        for f in &found {
            preview(f)?;
            println!("{}", f);
            println!("");
        }
    }

    info!("Searched {} icons.", icons.len());

    Ok(found)
}
