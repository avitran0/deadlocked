use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Write as _},
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
};

pub mod bvh;
pub mod dmx;
pub mod binary;
pub mod steam;

use crate::{
    os::crash::{self, report_error},
    parser::{bvh::{Bvh, Triangle}, steam::{find_game_dir, find_maps_dir, exe_path}, dmx::parse_dmx, dmx::Attribute},
};

pub fn parse_maps(
    bvh: Arc<Mutex<HashMap<String, Bvh>>>,
    mut force_reparse: bool,
    use_system_binary: bool,
) {
    crash::info();
    let source2viewer = exe_path().join("source2viewer/Source2Viewer-CLI");
    if !source2viewer.exists() && !use_system_binary {
        log::warn!("could not find source2viewer binary");
        return;
    }

    let game_dir = match find_game_dir() {
        Ok(dir) => dir,
        Err(err) => {
            log::warn!("could not find cs2 game directory: {err}");
            report_error(err);
            return;
        }
    };
    let build_file = game_dir.join("game/bin/built_from_cl.txt");
    let Ok(cs2_build_raw) = std::fs::read_to_string(&build_file) else {
        log::warn!("could not read cs2 build number");
        return;
    };
    let cs2_build: u64 = cs2_build_raw.trim().parse().unwrap_or_default();

    let maps_dir = match find_maps_dir() {
        Ok(dir) => dir,
        Err(err) => {
            log::error!("could not find cs2 maps directory: {err}");
            report_error(err);
            return;
        }
    };
    let parsed_build_file = maps_dir.join("parsed_build.txt");
    let parsed_build: u64 = std::fs::read_to_string(&parsed_build_file)
        .unwrap_or_default()
        .trim()
        .parse()
        .unwrap_or_default();

    if parsed_build != cs2_build {
        force_reparse = true;
    }

    if force_reparse {
        log::info!("reparsing map data");
    }

    let mut files = Vec::with_capacity(32);
    let maps_dir_iter = match std::fs::read_dir(&maps_dir) {
        Ok(dir) => dir,
        Err(err) => {
            log::error!("could not read cs2 maps dir: {err}");
            return;
        }
    };
    for file in maps_dir_iter {
        let Ok(file) = file else {
            continue;
        };

        if !file.file_type().map_or(true, |ft| !ft.is_file()) {
            continue;
        }

        let file_name = file.file_name();
        let file_name = file_name.to_str().unwrap();
        if file_name.contains("_vanity") {
            continue;
        }

        if !file_name.starts_with("ar_")
            && !file_name.starts_with("cs_")
            && !file_name.starts_with("de_")
        {
            continue;
        }

        if !file_name.ends_with(".vpk") {
            continue;
        }

        files.push(file_name.to_string());
    }

    let geom_dir = maps_dir.join("geometry");
    if force_reparse
        && geom_dir.exists()
        && let Err(err) = std::fs::remove_dir_all(&geom_dir)
    {
        log::error!("error removing geometry dir: {err}");
    }

    if !geom_dir.exists()
        && let Err(err) = std::fs::create_dir_all(geom_dir.join("maps"))
    {
        log::error!("error creating geometry dir: {err}");
    }
    for file in &files {
        let path = maps_dir.join(file);
        let map_name = file.replace(".vpk", "");

        if maps_dir.join("geometry/maps").join(&map_name).exists() && !force_reparse {
            continue;
        }

        let mut s2v_cmd = Command::new(if use_system_binary {
            std::ffi::OsStr::new("Source2Viewer-CLI")
        } else {
            source2viewer.as_os_str()
        });
        s2v_cmd.args([
            "-i",
            path.to_str().unwrap(),
            "-d",
            "-o",
            geom_dir.to_str().unwrap(),
            "-f",
            &format!("maps/{map_name}/world_physics.vmdl_c"),
        ]);
        if let Err(error) = s2v_cmd.output() {
            log::error!("source2viewer error:\n{error}");
        }
    }

    if !geom_dir.exists() {
        log::warn!("could not parse any map successfully");
        return;
    }

    let cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let batch_size = (cpus / 2).max(1);
    for chunk in files.chunks(batch_size) {
        let mut threads = Vec::with_capacity(batch_size);
        for map in chunk {
            let map = map.clone();
            let maps_dir = maps_dir.clone();
            let bvh_thread = bvh.clone();
            let thread = std::thread::spawn(move || {
                parse_map(&map, &maps_dir, bvh_thread, force_reparse);
            });
            threads.push(thread);
        }

        for thread in threads {
            let _ = thread.join();
        }
    }
    let mut parsed_build_file = match File::create(&parsed_build_file) {
        Ok(file) => file,
        Err(err) => {
            log::error!("could not open metadata file: {err}");
            return;
        }
    };
    if let Err(err) = parsed_build_file.write_all(format!("{cs2_build}").as_bytes()) {
        log::error!("could not write to metadata file: {err}");
    }
    log::info!("loaded map info");
}

fn parse_map(
    map: &str,
    maps_dir: &Path,
    bvh: Arc<Mutex<HashMap<String, Bvh>>>,
    force_reparse: bool,
) {
    let map_name = map.replace(".vpk", "");
    let bvh_name = format!("{map_name}.bvh");
    let bvh_path = maps_dir.join(bvh_name);

    if bvh_path.exists() && !force_reparse {
        if let Some(map_bvh) = load_cached_bvh(&bvh_path, &map_name) {
            store_bvh(bvh, map_name, map_bvh);
            return;
        }
    }

    let geom_dir = maps_dir.join("geometry/maps").join(&map_name);
    if !geom_dir.exists() {
        log::warn!("geometry directory doesn't exist...");
        return;
    }

    let mut map_bvh = Bvh::new();
    if let Err(err) = process_geometry_files(&geom_dir, &map_name, &mut map_bvh) {
        log::error!("Error processing geometry for {map_name}: {err}");
        return;
    }
    
    map_bvh.build();
    if let Err(err) = save_bvh(&map_bvh, &bvh_path, &map_name) {
        log::error!("Error saving BVH for {map_name}: {err}");
        return;
    }
    
    store_bvh(bvh, map_name, map_bvh);
}

fn load_cached_bvh(bvh_path: &Path, map_name: &str) -> Option<Bvh> {
    let mut bvh_file = File::open(bvh_path).ok()?;
    let map_bvh = Bvh::load(&mut bvh_file)?;
    log::debug!("loaded bvh for {map_name}");
    Some(map_bvh)
}

fn store_bvh(bvh: Arc<Mutex<HashMap<String, Bvh>>>, map_name: String, map_bvh: Bvh) {
    match bvh.lock() {
        Ok(mut lock) => {
            lock.insert(map_name, map_bvh);
        }
        Err(err) => {
            log::error!("failed to lock bvh mutex: {err}");
        }
    }
}

fn save_bvh(map_bvh: &Bvh, bvh_path: &Path, map_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut bvh_file = File::create(bvh_path)?;
    map_bvh.save(&mut bvh_file);
    log::info!("parsed bvh for {map_name}");
    Ok(())
}

fn process_geometry_files(geom_dir: &Path, _map_name: &str, map_bvh: &mut Bvh) -> Result<(), Box<dyn std::error::Error>> {
    let geom_dir_iter = std::fs::read_dir(geom_dir)?;
    for file in geom_dir_iter {
        let Ok(file) = file else {
            continue;
        };
        let file_name = file.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        let file_type = if file_name.contains("world_physics_hull") {
            FileType::Hull
        } else if file_name.contains("world_physics_phys") {
            FileType::Phys
        } else {
            continue;
        };
        let file = File::open(file.path())?;
        let mut reader = BufReader::new(file);
        let elements = parse_dmx(&mut reader);
        let vertex_element = elements.get("DmeVertexData_bind").unwrap();
        let Some(Attribute::Vec3Array(vertices)) = vertex_element.attributes.get("position$0")
        else {
            continue;
        };
        let vertex_indices: Vec<&[i32]> = if file_type == FileType::Hull {
            let Some(face_element) = elements.get("DmeFaceSet_hull faces") else {
                continue;
            };
            let Some(Attribute::IntegerArray(indices)) = face_element.attributes.get("faces")
            else {
                continue;
            };
            indices.split(|i| *i == -1).collect()
        } else {
            let Some(Attribute::IntegerArray(indices)) =
                vertex_element.attributes.get("position$0Indices")
            else {
                continue;
            };
            indices.chunks_exact(3).collect()
        };

        for face in vertex_indices {
            if face.len() < 3 || face.iter().any(|index| *index as usize >= vertices.len()) {
                continue;
            } else if face.len() == 3 {
                let v1 = vertices[face[0] as usize];
                let v2 = vertices[face[1] as usize];
                let v3 = vertices[face[2] as usize];
                let triangle = Triangle::new(v1, v2, v3);
                map_bvh.insert(triangle);
            } else {
                for i in 1..face.len() - 1 {
                    let v1 = vertices[face[0] as usize];
                    let v2 = vertices[face[i] as usize];
                    let v3 = vertices[face[i + 1] as usize];
                    let triangle = Triangle::new(v1, v2, v3);
                    map_bvh.insert(triangle);
                }
            }
        }
    }
    Ok(())
}

#[derive(PartialEq)]
enum FileType {
    Hull,
    Phys,
}

