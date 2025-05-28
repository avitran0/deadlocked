use std::{
    fs::{read_dir, read_link, File, OpenOptions},
    io::{BufRead, BufReader},
    os::{fd::AsRawFd, unix::fs::FileExt},
    path::{Path, PathBuf},
};

use bytemuck::{AnyBitPattern, NoUninit};
use libc::{iovec, process_vm_readv, process_vm_writev};
use log::{debug, warn};
use nix::{ioctl_readwrite, libc};

use crate::constants::elf;

#[derive(Debug, PartialEq)]
enum AccessMode {
    Syscall,
    KernelModule,
}

#[allow(unused)]
#[repr(C)]
struct MemoryParams {
    pid: libc::pid_t,
    addr: libc::c_ulong,
    size: libc::size_t,
    buf: *mut libc::c_void,
}

impl MemoryParams {
    pub fn new(pid: i32, addr: u64, size: usize, buf: *mut libc::c_void) -> Self {
        Self {
            pid,
            addr,
            size,
            buf,
        }
    }
}

ioctl_readwrite!(ioctl_read_mem, 0xBC, 1, MemoryParams);
ioctl_readwrite!(ioctl_write_mem, 0xBD, 1, MemoryParams);

#[derive(Debug)]
pub struct Process {
    pub pid: i32,
    file: File,
    path: PathBuf,
    mode: AccessMode,
}

impl Process {
    pub fn new(pid: i32) -> Self {
        let mode = if Path::new("/dev/stealthmem").exists() {
            AccessMode::KernelModule
        } else {
            AccessMode::Syscall
        };
        Self {
            pid,
            path: PathBuf::from(format!("/proc/{pid}")),
            file: match mode {
                AccessMode::Syscall => OpenOptions::new()
                    .write(true)
                    .open(format!("/proc/{pid}/mem"))
                    .unwrap_or_else(|_| OpenOptions::new().write(true).open("/dev/null").unwrap()),
                AccessMode::KernelModule => OpenOptions::new()
                    .write(true)
                    .open("/dev/stealthmem")
                    .unwrap(),
            },
            mode,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.path.exists() && self.pid > 0
    }

    pub fn read<T: AnyBitPattern + Default>(&self, address: u64) -> T {
        let mut buffer = vec![0u8; std::mem::size_of::<T>()];

        if self.mode == AccessMode::KernelModule {
            let mut params = MemoryParams::new(
                self.pid,
                address,
                std::mem::size_of::<T>(),
                buffer.as_mut_ptr() as *mut libc::c_void,
            );
            unsafe {
                ioctl_read_mem(self.file.as_raw_fd(), &mut params as *mut MemoryParams).unwrap()
            };
        } else {
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
        }

        bytemuck::try_from_bytes(&buffer)
            .copied()
            .unwrap_or_default()
    }

    pub fn write<T: NoUninit>(&self, address: u64, value: T) {
        let mut buffer = bytemuck::bytes_of(&value).to_vec();

        if self.mode == AccessMode::KernelModule {
            let mut params = MemoryParams::new(
                self.pid,
                address,
                std::mem::size_of::<T>(),
                buffer.as_mut_ptr() as *mut libc::c_void,
            );
            unsafe {
                ioctl_write_mem(self.file.as_raw_fd(), &mut params as *mut MemoryParams).unwrap()
            };
        } else {
            let local_iov = iovec {
                iov_base: buffer.as_mut_ptr() as *mut libc::c_void,
                iov_len: buffer.len(),
            };
            let remote_iov = iovec {
                iov_base: address as *mut libc::c_void,
                iov_len: buffer.len(),
            };

            unsafe { process_vm_writev(self.pid, &local_iov, 1, &remote_iov, 1, 0) };
        }
    }

    pub fn write_file<T: NoUninit>(&self, address: u64, value: T) {
        let mut buffer = bytemuck::bytes_of(&value).to_vec();
        if self.mode == AccessMode::KernelModule {
            let mut params = MemoryParams::new(
                self.pid,
                address,
                std::mem::size_of::<T>(),
                buffer.as_mut_ptr() as *mut libc::c_void,
            );
            unsafe {
                ioctl_write_mem(self.file.as_raw_fd(), &mut params as *mut MemoryParams).unwrap()
            };
        } else {
            self.file.write_at(&buffer, address).unwrap();
        }
    }

    pub fn read_string(&self, address: u64) -> String {
        let mut string = String::with_capacity(8);
        let mut i = address;
        loop {
            let c = self.read::<u8>(i);
            if c == 0 {
                break;
            }
            string.push(c as char);
            i += 1;
        }
        string
    }

    pub fn read_bytes(&self, address: u64, count: u64) -> Vec<u8> {
        let mut buffer = vec![0u8; count as usize];
        if self.mode == AccessMode::KernelModule {
            let mut params = MemoryParams::new(
                self.pid,
                address,
                count as usize,
                buffer.as_mut_ptr() as *mut libc::c_void,
            );
            unsafe {
                ioctl_read_mem(self.file.as_raw_fd(), &mut params as *mut MemoryParams).unwrap()
            };
        } else {
            self.file.read_at(&mut buffer, address).unwrap_or(0);
        }
        buffer
    }

    pub fn module_base_address(&self, module_name: &str) -> Option<u64> {
        let maps = File::open(format!("/proc/{}/maps", self.pid)).unwrap();
        for line in BufReader::new(maps).lines() {
            if line.is_err() {
                continue;
            }
            let line = line.unwrap();
            if !line.contains(module_name) {
                continue;
            }
            let (address, _) = line.split_once('-').unwrap();
            let address = u64::from_str_radix(address, 16).unwrap();
            debug!("found module {module_name} at {address}");
            return Some(address);
        }
        warn!("module {module_name} not found");
        None
    }

    pub fn dump_module(&self, address: u64) -> Vec<u8> {
        let module_size = self.module_size(address);
        self.read_bytes(address, module_size)
    }

    pub fn scan_pattern(&self, pattern: &[u8], mask: &[u8], base_address: u64) -> Option<u64> {
        assert!(pattern.len() == mask.len(), "pattern length mismatch");

        let module = self.dump_module(base_address);
        if module.len() < 500 {
            return None;
        }

        let pattern_length = pattern.len();
        let stop_index = module.len() - pattern_length;
        'outer: for i in 0..stop_index {
            for j in 0..pattern_length {
                if mask[j] == b'x' && module[i + j] != pattern[j] {
                    continue 'outer;
                }
            }
            let address = base_address + i as u64;
            debug!("found pattern {pattern:?} at {address}");
            return Some(address);
        }
        debug!("pattern {pattern:?} not found, might be outdated");
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
        let export_address = self.get_relative_address(create_interface, 0x01, 0x05) + 0x10;

        let mut interface_entry =
            self.read(export_address + 0x07 + self.read::<u32>(export_address + 0x03) as u64);

        loop {
            let entry_name_address = self.read(interface_entry + 8);
            let entry_name = self.read_string(entry_name_address);
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
            let name = self.read_string(string_table + st_name as u64);
            if name == export_name {
                return Some(self.read::<u64>(symbol_table + 0x08) + base_address);
            }
            symbol_table += add;
        }
        warn!("export {} could not be found", export_name);
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
        warn!("did not find tag {} in dynamic section", tag);
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
        warn!("did not find dynamic section in program header table");
        None
    }

    pub fn get_convar(&self, convar_interface: u64, convar_name: &str) -> Option<u64> {
        if convar_interface == 0 {
            return None;
        }

        let objects = self.read::<u64>(convar_interface + 64);
        for i in 0..self.read::<u32>(convar_interface + 160) as u64 {
            let object = self.read(objects + i * 16);
            if object == 0 {
                break;
            }

            let name_address = self.read(object);
            let name = self.read_string(name_address);
            if name == convar_name {
                return Some(object);
            }
        }
        warn!("did not find convar {convar_name}");
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

    pub fn read_vec<T: AnyBitPattern + Default>(&self, data: &[u8], address: u64) -> T {
        let size = std::mem::size_of::<T>();
        if address as usize + size > data.len() {
            return T::default();
        }

        let slice = &data[address as usize..address as usize + size];
        bytemuck::try_from_bytes(slice).copied().unwrap_or_default()
    }

    pub fn read_string_vec(&self, data: &[u8], address: u64) -> String {
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

    fn get_pid(process_name: &str) -> Option<i32> {
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
