use std::{
    collections::HashMap,
    path::PathBuf,
    sync::OnceLock,
};

use serde::Deserialize;

use crate::{
    constants::cs2,
    cs2::{
        offsets::{DirectOffsets, LibraryOffsets},
        schema::Schema,
    },
    os::process::Process,
};

const EMBEDDED: &str = include_str!("signatures.json");

static MANIFEST: OnceLock<Manifest> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct Manifest {
    signatures: Vec<Signature>,
}

#[derive(Debug, Deserialize)]
struct Signature {
    name: String,
    module: String,
    #[serde(default = "default_true")]
    required: bool,
    variants: Vec<Variant>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct Variant {
    pattern: String,
    #[serde(default)]
    match_offset: u64,
    operations: Vec<Operation>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Operation {
    Rip {
        #[serde(default = "default_rip_offset")]
        offset: u8,
        #[serde(default = "default_rip_len")]
        len: u8,
    },
    Add {
        value: u64,
    },
    Slice {
        start: u8,
        end: u8,
    },
    Read,
}

fn default_rip_offset() -> u8 {
    3
}

fn default_rip_len() -> u8 {
    7
}

#[derive(Debug, Default)]
pub struct Globals {
    pub direct: DirectOffsets,
    pub entity_system: u64,
    pub sensitivity: u64,
}

pub struct Resolved {
    pub globals: Globals,
    pub schema: Schema,
}

/// Resolve global pointers and schema field offsets. Runs once during CS2::setup.
pub fn resolve(process: &Process, libraries: &LibraryOffsets) -> Option<Resolved> {
    let manifest = manifest();
    let mut modules = ModuleCache::new(process);

    let schema_system = {
        let sig = manifest.signatures.iter().find(|s| s.name == "schema_system")?;
        let base = module_base(libraries, &sig.module)?;
        resolve_validated(process, base, sig, &mut modules, always_valid)?
    };

    let schema = Schema::from_system(process, schema_system)?;
    let client = schema.get_library(cs2::CLIENT_LIB)?;
    let controller_name = client.get("CBasePlayerController", "m_iszPlayerName")?;

    let mut globals = Globals::default();

    for sig in &manifest.signatures {
        if sig.name == "schema_system" {
            continue;
        }

        let base = module_base(libraries, &sig.module)?;
        let resolved = match sig.name.as_str() {
            "local_player" => resolve_local_player(
                process,
                base,
                sig,
                &mut modules,
                controller_name,
            )?,
            "view_matrix" => resolve_validated(
                process,
                base,
                sig,
                &mut modules,
                validate_view_matrix,
            )?,
            _ if sig.required => {
                resolve_validated(process, base, sig, &mut modules, always_valid)?
            }
            _ => resolve_signature(process, base, sig, &mut modules).unwrap_or(0),
        };

        match sig.name.as_str() {
            "local_player" => globals.direct.local_player = resolved,
            "view_matrix" => globals.direct.view_matrix = resolved,
            "global_vars" => {
                globals.direct.global_vars = resolved;
                globals.direct.global_vars_map_name = detect_map_name_offset(process, resolved);
            }
            "planted_c4" => globals.direct.planted_c4 = resolved,
            "entity_system" => globals.entity_system = resolved,
            "vphys_world" => globals.direct.vphys_world = resolved,
            "sensitivity" => globals.sensitivity = resolved,
            _ => {}
        }
    }

    Some(Resolved { globals, schema })
}

fn resolve_local_player(
    process: &Process,
    module_base: u64,
    sig: &Signature,
    modules: &mut ModuleCache,
    name_offset: u64,
) -> Option<u64> {
    for (i, variant) in sig.variants.iter().enumerate() {
        let Some(match_addr) = modules.scan(&variant.pattern, module_base) else {
            continue;
        };
        let instruction = match_addr.wrapping_add(variant.match_offset);
        let Some(resolved) = apply_ops(process, instruction, &variant.operations) else {
            continue;
        };
        if !validate_local_player(process, resolved, name_offset) {
            continue;
        }
        utils::debug!("signature {}: variant {i} matched", sig.name);
        return Some(resolved);
    }
    utils::warn!("signature {}: all patterns failed", sig.name);
    None
}

fn resolve_validated(
    process: &Process,
    module_base: u64,
    sig: &Signature,
    modules: &mut ModuleCache,
    validate: fn(&Process, u64) -> bool,
) -> Option<u64> {
    for (i, variant) in sig.variants.iter().enumerate() {
        let Some(match_addr) = modules.scan(&variant.pattern, module_base) else {
            continue;
        };
        let instruction = match_addr.wrapping_add(variant.match_offset);
        let Some(resolved) = apply_ops(process, instruction, &variant.operations) else {
            continue;
        };
        if !validate(process, resolved) {
            continue;
        }
        utils::debug!("signature {}: variant {i} matched", sig.name);
        return Some(resolved);
    }
    utils::warn!("signature {}: all patterns failed", sig.name);
    None
}

fn resolve_signature(
    process: &Process,
    module_base: u64,
    sig: &Signature,
    modules: &mut ModuleCache,
) -> Option<u64> {
    for (i, variant) in sig.variants.iter().enumerate() {
        let Some(match_addr) = modules.scan(&variant.pattern, module_base) else {
            continue;
        };
        let instruction = match_addr.wrapping_add(variant.match_offset);
        if let Some(resolved) = apply_ops(process, instruction, &variant.operations) {
            utils::debug!("signature {}: variant {i} matched", sig.name);
            return Some(resolved);
        }
    }

    if sig.required {
        utils::warn!("signature {}: all patterns failed", sig.name);
    }
    None
}

fn apply_ops(process: &Process, mut addr: u64, ops: &[Operation]) -> Option<u64> {
    for op in ops {
        addr = match op {
            Operation::Rip { offset, len } => {
                process.get_relative_address(addr, *offset as u64, *len as u64)
            }
            Operation::Add { value } => addr.wrapping_add(*value),
            Operation::Slice { start, end } => {
                let bytes = process.read_bytes(addr + *start as u64, (*end - *start) as u64);
                let mut value = 0u64;
                for (i, byte) in bytes.iter().enumerate() {
                    value |= (*byte as u64) << (i * 8);
                }
                value
            }
            Operation::Read => process.read(addr),
        };
    }
    Some(addr)
}

fn always_valid(_: &Process, _: u64) -> bool {
    true
}

fn validate_view_matrix(process: &Process, addr: u64) -> bool {
    let value: f32 = process.read(addr);
    value.is_finite() && value.abs() < 100.0
}

fn validate_local_player(process: &Process, global_addr: u64, name_offset: u64) -> bool {
    let controller: u64 = process.read(global_addr);
    if controller == 0 {
        return true;
    }
    let name_ptr: u64 = process.read(controller + name_offset);
    if name_ptr == 0 {
        return true;
    }
    let name = process.read_string_uncached(name_ptr);
    if name.is_empty() {
        return true;
    }
    name.len() < 64 && name.chars().all(|c| c.is_ascii_graphic() || c == ' ')
}

fn detect_map_name_offset(process: &Process, global_vars_global: u64) -> u64 {
    const CANDIDATES: [u64; 2] = [0x1C8, 0x198];
    let global_vars: u64 = process.read(global_vars_global);
    if global_vars == 0 {
        return CANDIDATES[0];
    }

    for offset in CANDIDATES {
        let name_ptr: u64 = process.read(global_vars + offset);
        if name_ptr == 0 {
            continue;
        }
        let name = process.read_string_uncached(name_ptr);
        if name.starts_with("de_") || name.starts_with("cs_") || name.starts_with("ar_") {
            return offset;
        }
    }
    CANDIDATES[0]
}

fn module_base(libraries: &LibraryOffsets, module: &str) -> Option<u64> {
    match module {
        cs2::CLIENT_LIB => Some(libraries.client),
        cs2::SCHEMA_LIB => Some(libraries.schema),
        _ => None,
    }
}

fn manifest() -> &'static Manifest {
    MANIFEST.get_or_init(|| {
        let mut manifest: Manifest =
            serde_json::from_str(EMBEDDED).expect("embedded signatures.json");

        if let Ok(path) = override_path() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(overlay) = serde_json::from_str::<Manifest>(&content) {
                    merge_manifest(&mut manifest, overlay);
                }
            }
        }

        manifest
    })
}

fn override_path() -> Result<PathBuf, std::env::VarError> {
    let config = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        format!("{home}/.config")
    });
    Ok(PathBuf::from(config).join("deadlocked/signatures.json"))
}

fn merge_manifest(base: &mut Manifest, overlay: Manifest) {
    for overlay_sig in overlay.signatures {
        let Some(local) = base.signatures.iter_mut().find(|s| s.name == overlay_sig.name)
        else {
            base.signatures.push(overlay_sig);
            continue;
        };
        for variant in overlay_sig.variants.into_iter().rev() {
            local.variants.insert(0, variant);
        }
    }
}

struct ModuleCache<'a> {
    process: &'a Process,
    modules: HashMap<u64, Vec<u8>>,
}

impl<'a> ModuleCache<'a> {
    fn new(process: &'a Process) -> Self {
        Self {
            process,
            modules: HashMap::new(),
        }
    }

    fn scan(&mut self, pattern: &str, base_address: u64) -> Option<u64> {
        let module = self
            .modules
            .entry(base_address)
            .or_insert_with(|| self.process.dump_module(base_address));

        let (bytes, mask) = parse_pattern(pattern)?;
        if module.len() < bytes.len() {
            return None;
        }

        let scan = if bytes.len() <= 32 && is_x86_feature_detected!("avx2") {
            crate::os::process::scan_simd
        } else {
            crate::os::process::scan_normal
        };

        scan(&bytes, &mask, module).map(|offset| base_address + offset)
    }
}

fn parse_pattern(pattern: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let mut bytes = Vec::new();
    let mut mask = Vec::new();

    for token in pattern.split_whitespace() {
        if token == "?" || token == "??" {
            bytes.push(0x00);
            mask.push(0x00);
        } else if token.len() == 2 {
            bytes.push(u8::from_str_radix(token, 16).ok()?);
            mask.push(0xFF);
        }
    }

    if bytes.is_empty() { None } else { Some((bytes, mask)) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{constants::cs2, cs2::offsets::LibraryOffsets, os::process::Process};

    #[test]
    fn resolve_against_live_cs2() {
        let Some(process) = Process::open(cs2::PROCESS_NAME) else {
            return;
        };

        let libraries = LibraryOffsets {
            client: process.module_base_address(cs2::CLIENT_LIB).unwrap(),
            engine: process.module_base_address(cs2::ENGINE_LIB).unwrap(),
            tier0: process.module_base_address(cs2::TIER0_LIB).unwrap(),
            input: process.module_base_address(cs2::INPUT_LIB).unwrap(),
            sdl: process.module_base_address(cs2::SDL_LIB).unwrap(),
            schema: process.module_base_address(cs2::SCHEMA_LIB).unwrap(),
        };

        let resolved = resolve(&process, &libraries).expect("offset resolution failed");
        assert_ne!(resolved.globals.direct.local_player, 0);
        assert_ne!(resolved.globals.direct.view_matrix, 0);
        assert_ne!(resolved.globals.entity_system, 0);
        assert!(resolved.schema.get_library(cs2::CLIENT_LIB).is_some());
    }
}
