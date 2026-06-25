use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{File, OpenOptions, read_dir, read_link},
    io::{BufRead, BufReader},
    os::unix::fs::FileExt,
    path::PathBuf,
};

use bytemuck::Pod;
use nix::libc::{self, iovec, process_vm_readv};

use crate::constants::{cs2, elf};

#[derive(Debug)]
pub struct Process {
    pub pid: i32,
    file: File,
    path: PathBuf,
    pub min: u64,
    pub max: u64,
    string_cache: RefCell<HashMap<u64, String>>,
}

impl Process {
    pub fn new(pid: i32) -> Self {
        if pid == -1 {
            return Self {
                pid,
                path: PathBuf::from(format!("/proc/{pid}")),
                file: OpenOptions::new().read(true).open("/dev/null").unwrap(),
                min: u64::MAX,
                max: u64::MIN,
                string_cache: RefCell::new(HashMap::new()),
            };
        }

        let file = OpenOptions::new()
            .read(true)
            .open(format!("/proc/{pid}/mem"))
            .unwrap_or_else(|e| {
                utils::error!("failed to open /proc/{pid}/mem: {e}");
                OpenOptions::new().read(true).open("/dev/null").unwrap()
            });
        let mut ret = Self {
            pid,
            path: PathBuf::from(format!("/proc/{pid}")),
            file,
            min: u64::MAX,
            max: u64::MIN,
            string_cache: RefCell::new(HashMap::new()),
        };

        let libs: Vec<u64> = cs2::LIBS
            .iter()
            .filter_map(|&lib| ret.module_base_address(lib))
            .collect();
        let sizes: Vec<u64> = libs.iter().map(|lib| ret.module_size(*lib)).collect();

        for (lib, size) in libs.into_iter().zip(sizes) {
            let min = lib - 1_000_000;
            let max = lib + size + 1_000_000;
            if min < ret.min {
                ret.min = min;
            }
            if max > ret.max {
                ret.max = max;
            }
        }

        ret
    }

    pub fn is_valid(&self) -> bool {
        self.path.exists() && self.pid > 0
    }

    pub fn read<T: Pod + Default>(&self, address: u64) -> T {
        let mut t = T::default();
        let buffer = bytemuck::bytes_of_mut(&mut t);

        let local_iov = iovec {
            iov_base: buffer.as_mut_ptr() as *mut libc::c_void,
            iov_len: buffer.len(),
        };
        let remote_iov = iovec {
            iov_base: address as *mut libc::c_void,
            iov_len: buffer.len(),
        };

        unsafe {
            process_vm_readv(self.pid, &local_iov, 1, &remote_iov, 1, 0);
        }

        t
    }

    pub fn read_or_zeroed<T: Pod>(&self, address: u64) -> T {
        let mut t = T::zeroed();
        let buffer = bytemuck::bytes_of_mut(&mut t);

        let local_iov = iovec {
            iov_base: buffer.as_mut_ptr() as *mut libc::c_void,
            iov_len: buffer.len(),
        };
        let remote_iov = iovec {
            iov_base: address as *mut libc::c_void,
            iov_len: buffer.len(),
        };

        unsafe {
            process_vm_readv(self.pid, &local_iov, 1, &remote_iov, 1, 0);
        }

        t
    }

    pub fn read_vec(&self, address: u64, length: usize) -> Vec<u8> {
        let mut buffer = vec![0u8; length];

        let local_iov = iovec {
            iov_base: buffer.as_mut_ptr() as *mut libc::c_void,
            iov_len: buffer.len(),
        };
        let remote_iov = iovec {
            iov_base: address as *mut libc::c_void,
            iov_len: buffer.len(),
        };

        unsafe {
            process_vm_readv(self.pid, &local_iov, 1, &remote_iov, 1, 0);
        }

        buffer
    }

    pub fn read_typed_vec<T: Pod + Default>(
        &self,
        address: u64,
        stride: usize,
        count: usize,
    ) -> Vec<T> {
        let size = size_of::<T>();
        assert!(stride >= size);

        let mut buffer = vec![0u8; stride * count];

        let local_iov = iovec {
            iov_base: buffer.as_mut_ptr() as *mut libc::c_void,
            iov_len: buffer.len(),
        };
        let remote_iov = iovec {
            iov_base: address as *mut libc::c_void,
            iov_len: buffer.len(),
        };

        unsafe {
            process_vm_readv(self.pid, &local_iov, 1, &remote_iov, 1, 0);
        }

        let mut result = vec![T::default(); count];
        let result_ptr = result.as_mut_ptr() as *mut u8;

        for i in 0..count {
            unsafe {
                let src = buffer.as_ptr().add(i * stride);
                let dst = result_ptr.add(i * size);
                std::ptr::copy_nonoverlapping(src, dst, size);
            }
        }

        result
    }

    #[cfg(feature = "read-only")]
    pub fn write<T: Pod>(&self, _address: u64, _value: T) {}

    #[cfg(not(feature = "read-only"))]
    pub fn write<T: Pod>(&self, address: u64, value: T) {
        let mut buffer = bytemuck::bytes_of(&value).to_vec();

        let local_iov = iovec {
            iov_base: buffer.as_mut_ptr() as *mut libc::c_void,
            iov_len: buffer.len(),
        };
        let remote_iov = iovec {
            iov_base: address as *mut libc::c_void,
            iov_len: buffer.len(),
        };

        unsafe { nix::libc::process_vm_writev(self.pid, &local_iov, 1, &remote_iov, 1, 0) };
    }

    pub fn read_string(&self, address: u64) -> String {
        if let Some(cached) = self.string_cache.borrow().get(&address).cloned() {
            return cached;
        }
        let string = self.read_string_uncached(address);
        self.string_cache
            .borrow_mut()
            .insert(address, string.clone());
        string
    }

    pub fn read_string_uncached(&self, address: u64) -> String {
        let mut bytes = Vec::with_capacity(8);
        let mut i = address;
        loop {
            let c = self.read::<u8>(i);
            if c == 0 {
                break;
            }
            bytes.push(c);
            i += 1;
        }

        String::from_utf8(bytes).unwrap_or_default()
    }

    pub fn read_bytes(&self, address: u64, count: u64) -> Vec<u8> {
        let mut buffer = vec![0u8; count as usize];
        self.file.read_at(&mut buffer, address).unwrap_or(0);
        buffer
    }

    pub fn module_base_address(&self, module_name: &str) -> Option<u64> {
        let Ok(maps) = File::open(format!("/proc/{}/maps", self.pid)) else {
            return None;
        };
        for line in BufReader::new(maps).lines() {
            let Ok(line) = line else {
                continue;
            };
            if !line.contains(module_name) {
                continue;
            }
            let Some((address, _)) = line.split_once('-') else {
                continue;
            };
            let Ok(address) = u64::from_str_radix(address, 16) else {
                continue;
            };
            utils::debug!("found module {module_name} at {address:X}");
            return Some(address);
        }
        utils::warn!("module {module_name} not found");
        None
    }

    pub fn dump_module(&self, address: u64) -> Vec<u8> {
        let module_size = self.module_size(address);
        self.read_bytes(address, module_size)
    }

    pub fn scan(&self, pattern: &str, base_address: u64) -> Option<u64> {
        let mut bytes = Vec::with_capacity(8);
        let mut mask = Vec::with_capacity(8);

        for token in pattern.split_whitespace() {
            if token == "?" || token == "??" {
                bytes.push(0x00);
                mask.push(0x00);
            } else if token.len() == 2 {
                match u8::from_str_radix(token, 16) {
                    Ok(b) => {
                        bytes.push(b);
                        mask.push(0xFF);
                    }
                    Err(_) => {
                        utils::warn!("unrecognized pattern token \"{token}\" in pattern {pattern}");
                    }
                }
            } else {
                utils::warn!("unrecognized pattern token \"{token}\" in pattern {pattern}");
            }
        }

        let module = self.dump_module(base_address);
        if module.len() < 500 {
            return None;
        }

        let scan_func = if bytes.len() <= 32 && is_x86_feature_detected!("avx2") {
            scan_simd
        } else {
            scan_normal
        };

        if let Some(address) =
            scan_func(&bytes, &mask, &module).map(|address| base_address + address)
        {
            return Some(address);
        }

        utils::info!("pattern {pattern} not found, might be outdated");
        None
    }

    pub fn get_relative_address(
        &self,
        instruction: u64,
        offset: u64,
        instruction_size: u64,
    ) -> u64 {
        // rip is instruction pointer
        let rip_address = self.read::<i32>(instruction + offset);
        instruction
            .wrapping_add(instruction_size)
            .wrapping_add(rip_address as u64)
    }

    pub fn get_interface_offset(&self, base_address: u64, interface_name: &str) -> Option<u64> {
        let create_interface = self.get_module_export(base_address, "CreateInterface")?;
        let export_address = create_interface + 0x10;

        let mut interface_entry =
            self.read(export_address + 0x07 + self.read::<u32>(export_address + 0x03) as u64);

        loop {
            let entry_name_address = self.read(interface_entry + 8);
            let entry_name = self.read_string_uncached(entry_name_address);
            if entry_name.starts_with(interface_name) {
                let vfunc_address = self.read::<u64>(interface_entry);
                return Some(self.read::<u32>(vfunc_address + 0x03) as u64 + vfunc_address + 0x07);
            }
            interface_entry = self.read(interface_entry + 0x10);
            if interface_entry == 0 {
                break;
            }
        }
        None
    }

    pub fn get_module_export(&self, base_address: u64, export_name: &str) -> Option<u64> {
        let add = 0x18;

        let string_table = self.get_address_from_dynamic_section(base_address, 0x05)?;
        let mut symbol_table = self.get_address_from_dynamic_section(base_address, 0x06)?;

        symbol_table += add;

        while self.read::<u32>(symbol_table) != 0 {
            let st_name = self.read::<u32>(symbol_table);
            let name = self.read_string_uncached(string_table + st_name as u64);
            if name == export_name {
                return Some(self.read::<u64>(symbol_table + 0x08) + base_address);
            }
            symbol_table += add;
        }
        utils::warn!("export {} could not be found", export_name);
        None
    }

    pub fn get_address_from_dynamic_section(&self, base_address: u64, tag: u64) -> Option<u64> {
        let dynamic_section_offset =
            self.get_segment_from_pht(base_address, elf::DYNAMIC_SECTION_PHT_TYPE)?;

        let register_size = 8;
        let mut address =
            self.read::<u64>(dynamic_section_offset + 2 * register_size) + base_address;

        loop {
            let tag_address = address;
            let tag_value = self.read::<u64>(tag_address);

            if tag_value == 0 {
                break;
            }
            if tag_value == tag {
                return Some(self.read(tag_address + register_size));
            }

            address += register_size * 2;
        }
        utils::warn!("did not find tag {} in dynamic section", tag);
        None
    }

    pub fn get_segment_from_pht(&self, base_address: u64, tag: u64) -> Option<u64> {
        let first_entry =
            self.read::<u64>(base_address + elf::PROGRAM_HEADER_OFFSET) + base_address;
        let entry_size = self.read::<u16>(base_address + elf::PROGRAM_HEADER_ENTRY_SIZE) as u64;

        for i in 0..self.read::<u16>(base_address + elf::PROGRAM_HEADER_NUM_ENTRIES) {
            let entry = first_entry + i as u64 * entry_size;
            if self.read::<u32>(entry) as u64 == tag {
                return Some(entry);
            }
        }
        utils::warn!("did not find dynamic section in program header table");
        None
    }

    pub fn get_convar(&self, convar_interface: u64, convar_name: &str) -> Option<u64> {
        if convar_interface == 0 {
            return None;
        }

        let objects = self.read::<u64>(convar_interface + 0x48);
        for i in 0..self.read::<u32>(convar_interface + 160) as u64 {
            let object = self.read(objects + i * 16);
            if object == 0 {
                break;
            }

            let name_address = self.read(object);
            let name = self.read_string_uncached(name_address);
            if name == convar_name {
                return Some(object);
            }
        }
        utils::warn!("did not find convar {convar_name}");
        None
    }

    pub fn module_size(&self, address: u64) -> u64 {
        let section_header_offset = self.read::<u64>(address + elf::SECTION_HEADER_OFFSET);
        let section_header_entry_size =
            self.read::<u16>(address + elf::SECTION_HEADER_ENTRY_SIZE) as u64;
        let section_header_num_entries =
            self.read::<u16>(address + elf::SECTION_HEADER_NUM_ENTRIES) as u64;

        section_header_offset + section_header_entry_size * section_header_num_entries
    }

    pub fn get_interface_function(&self, interface_address: u64, index: u64) -> u64 {
        self.read(self.read::<u64>(interface_address) + (index * 8))
    }

    fn get_pid(process_name: &str) -> Option<i32> {
        for dir in read_dir("/proc").unwrap() {
            let entry = dir.unwrap();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_dir() {
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
                return Some(pid.parse::<i32>().unwrap());
            }
        }
        None
    }

    pub fn open(process_name: &str) -> Option<Process> {
        let pid = Self::get_pid(process_name)?;
        let process = Process::new(pid);
        if !process.is_valid() {
            None
        } else {
            Some(process)
        }
    }
}

fn scan_normal(bytes: &[u8], mask: &[u8], module: &[u8]) -> Option<u64> {
    let pattern_length = bytes.len();
    let stop_index = module.len() - pattern_length;
    'outer: for i in 0..stop_index {
        for j in 0..pattern_length {
            if mask[j] == 0xFF && module[i + j] != bytes[j] {
                continue 'outer;
            }
        }
        return Some(i as u64);
    }
    None
}

fn scan_simd(bytes: &[u8], mask: &[u8], module: &[u8]) -> Option<u64> {
    use std::arch::x86_64::{
        __m256i, _mm256_and_si256, _mm256_loadu_si256, _mm256_testz_si256, _mm256_xor_si256,
    };

    let pattern_length = bytes.len();
    assert!(pattern_length <= 32);
    assert_eq!(mask.len(), pattern_length);
    assert_eq!(mask[0], 0xFF);

    let stop_index = module.len() - 32;

    let mut pattern_padded = [0u8; 32];
    let mut mask_padded = [0u8; 32];
    pattern_padded[..pattern_length].copy_from_slice(bytes);
    mask_padded[..pattern_length].copy_from_slice(mask);

    let pattern = unsafe { _mm256_loadu_si256(pattern_padded.as_ptr().cast::<__m256i>()) };
    let mask = unsafe { _mm256_loadu_si256(mask_padded.as_ptr().cast::<__m256i>()) };

    for i in 0..stop_index {
        if module[i] != bytes[0] {
            continue;
        }
        let module_bytes = unsafe { _mm256_loadu_si256(module.as_ptr().add(i) as *const __m256i) };
        let pattern_xor = unsafe { _mm256_xor_si256(module_bytes, pattern) };
        let masked = unsafe { _mm256_and_si256(pattern_xor, mask) };

        if unsafe { _mm256_testz_si256(masked, masked) } == 1 {
            return Some(i as u64);
        }
    }

    None
}
