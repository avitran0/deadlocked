use std::path::PathBuf;

pub fn find_game_dir() -> Result<PathBuf, String> {
    let Ok(home) = std::env::var("HOME") else {
        return Err("could not find home directory".into());
    };
    let steam_path = PathBuf::from(&home).join(".steam/steam");
    if !steam_path.exists() {
        return Err(format!(
            "could not locate steam directory ({home}/.steam/steam)"
        ));
    }

    let library_folders = steam_path.join("config/libraryfolders.vdf");
    let Ok(content) = std::fs::read_to_string(&library_folders) else {
        return Err(format!(
            "could not read steam library folders({home}/.steam/steam/config/libraryfolders.vdf)"
        ));
    };
    let libs: Vec<&str> = content
        .lines()
        .filter_map(|line| {
            if line.contains("\"path\"") {
                Some(line.rsplit('"').nth(1).unwrap())
            } else {
                None
            }
        })
        .collect();

    let game_dir = libs
        .iter()
        .find(|&&lib| {
            let dir = PathBuf::from(lib).join("steamapps/common/Counter-Strike Global Offensive");
            dir.exists()
        })
        .ok_or::<String>("could not locate cs2 files. is it installed?".into())?;
    Ok(PathBuf::from(game_dir).join("steamapps/common/Counter-Strike Global Offensive"))
}

pub fn find_maps_dir() -> Result<PathBuf, String> {
    let maps_dir = find_game_dir().map(|p| p.join("game/csgo/maps"))?;
    if maps_dir.exists() {
        Ok(maps_dir)
    } else {
        Err("could not locate csgo directory, but not maps directory".into())
    }
}

pub fn exe_path() -> PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}
