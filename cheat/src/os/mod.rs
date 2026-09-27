pub mod mouse;
pub mod process;

pub fn is_omarchy() -> bool {
    let contents = std::fs::read_to_string("/etc/os-release")
        .or_else(|_| std::fs::read_to_string("/usr/lib/os-release"));

    contents
        .map(|contents| is_omarchy_release(&contents))
        .unwrap_or(false)
}

fn is_omarchy_release(contents: &str) -> bool {
    contents.lines().any(|line| {
        let Some((key, value)) = line.split_once('=') else {
            return false;
        };

        let value = value.trim().trim_matches(['"', '\'']);
        match key.trim() {
            "ID" => value.eq_ignore_ascii_case("omarchy"),
            "ID_LIKE" => value
                .split_ascii_whitespace()
                .any(|id| id.eq_ignore_ascii_case("omarchy")),
            _ => false,
        }
    })
}
