pub struct Constants;
impl Constants {
    pub const ELF_PROGRAM_HEADER_OFFSET: u64 = 0x20;
    pub const ELF_PROGRAM_HEADER_ENTRY_SIZE: u64 = 0x36;
    pub const ELF_PROGRAM_HEADER_NUM_ENTRIES: u64 = 0x38;

    pub const ELF_SECTION_HEADER_OFFSET: u64 = 0x28;
    pub const ELF_SECTION_HEADER_ENTRY_SIZE: u64 = 0x3A;
    pub const ELF_SECTION_HEADER_NUM_ENTRIES: u64 = 0x3C;

    pub const ELF_DYNAMIC_SECTION_PHT_TYPE: u64 = 0x02;
}
