use std::{
    fs::{read_dir, read_link, File},
    path::Path,
};

use bytemuck::{Pod, Zeroable};

use crate::process::Process;

pub fn get_pid(process_name: &str) -> Option<u64> {
    for dir in read_dir("/proc").unwrap() {
        let entry = dir.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }

        let pid_osstr = entry.file_name();
        let pid = pid_osstr.to_str().unwrap();

        if !pid.chars().all(|char| char.is_numeric()) {
            continue;
        }

        let Ok(exe_path) = read_link(format!("/proc/{}/exe", pid)) else {
            continue;
        };

        let (_, exe_name) = exe_path.to_str().unwrap().rsplit_once('/').unwrap();

        if exe_name == process_name {
            return Some(pid.parse::<u64>().unwrap());
        }
    }
    None
}

pub fn validate_pid(pid: u64) -> bool {
    Path::new(&format!("/proc/{}", pid)).exists()
}

pub fn open_process(pid: u64) -> Option<Process> {
    if !validate_pid(pid) {
        return None;
    }

    let memory = File::open(format!("/proc/{pid}/mem"));
    match memory {
        Ok(mem) => Some(Process::new(pid, mem)),
        _ => None,
    }
}

pub fn read_vec<T: Pod + Zeroable + Default>(data: &[u8], address: u64) -> T {
    let size = std::mem::size_of::<T>();
    if address as usize + size > data.len() {
        return T::default();
    }

    let slice = &data[address as usize..address as usize + size];
    bytemuck::try_from_bytes(slice).copied().unwrap_or_default()
}

pub fn read_string_vec(data: &[u8], address: u64) -> String {
    let mut string = String::new();
    let mut i = address;
    loop {
        let c = data[i as usize];
        if c == 0 {
            break;
        }
        string.push(c as char);
        i += 1;
    }
    string
}
