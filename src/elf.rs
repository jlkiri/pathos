use elf::{endian::LittleEndian, section::SectionHeader, ElfBytes};

pub fn parse_text(data: &[u8]) -> &[u8] {
    let file = ElfBytes::<LittleEndian>::minimal_parse(data).expect("Failed to parse ELF file");

    let text_section: SectionHeader = file
        .section_header_by_name(".text")
        .expect("Failed to find .text section")
        .expect("Failed to parse .text section");

    let (text, _) = file
        .section_data(&text_section)
        .expect("Failed to read .text section");

    text
}
