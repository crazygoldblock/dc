use std::fs::{File, write};
use std::io::Write;

pub fn build(
    instructions: &mut Vec<u8>, 
    output_path: &str,
) {
    let buffer = Vec::new();

    let buffer = build_header(buffer, instructions.len() as u32);
    let buffer = build_idata_section(buffer);

    write(output_path, buffer).unwrap();

    while instructions.len() % 512 != 0 {
        instructions.push(0xC3);
    }

    File::options().append(true).open(output_path).unwrap().write(instructions).unwrap();
}
fn build_header(vec: Vec<u8>, code_size: u32) -> Vec<u8> {
    let mut builder = BinaryBuilder::with_vec(vec);
    let start_size = builder.len();

    // DOS header
    builder.push_u16_le(0x5A4D); // magic
    builder.push_slice(&[0; 58]);
    builder.push_u32_le(64); // PE header offset
    assert_eq!(builder.len(), 64);

    // PE header
    builder.push_u32_le(0x00_00_45_50); // magic
    builder.push_u16_le(0x8664); // machine
    builder.push_u16_le(2); // number of sections
    builder.push_u32_le(0); // timestamp
    builder.push_u32_le(0); // pointer to symbol table
    builder.push_u32_le(0); // number of symbols
    builder.push_u16_le(240); // size of optional header
    builder.push_u16_le(0x23); // characteristics
    assert_eq!(builder.len(), 64 + 24);

    // optional header
    builder.push_u16_le(0x20B); // magic PE32+
    builder.push_u8_le(0); // linker major version
    builder.push_u8_le(0); // linker minor version
    builder.push_u32_le(code_size.next_multiple_of(512)); // size of code section
    builder.push_u32_le(0x200); // size of data section
    builder.push_u32_le(0); // size of uninit data section
    builder.push_u32_le(0x2000); // address of entry point
    builder.push_u32_le(0x2000); // base of code
    assert_eq!(builder.len(), 64 + 24 + 24);

    // optional header - windows specific fields
    builder.push_u64_le(0x1_4000_0000); // image base
    builder.push_u32_le(0x1000); // section alignment
    builder.push_u32_le(0x200); // file alignment
    builder.push_u16_le(6); // major OS version
    builder.push_u16_le(0); // minor OS version
    builder.push_u16_le(0); // major image version
    builder.push_u16_le(0); // minor image version
    builder.push_u16_le(6); // major subsystem version
    builder.push_u16_le(0); // minor subsystem version
    builder.push_u32_le(0); // Win32version value, must be zero
    builder.push_u32_le(0x3000); // size of image -------------IMPORTANT
    builder.push_u32_le(0x200); // size of headers
    builder.push_u32_le(0); // checksum
    builder.push_u16_le(3); // subsystem
    builder.push_u16_le(0x100); // Dll characteristics
    builder.push_u64_le(0x10_0000); // stack reserve size
    builder.push_u64_le(0x1000); // stack commit size
    builder.push_u64_le(0x10_0000); // heap size reserve
    builder.push_u64_le(0x1000); // heap size commit
    builder.push_u32_le(0); // loader flags, must be zero
    builder.push_u32_le(16); // number of Rva and sizes
    assert_eq!(builder.len(), 64 + 24 + 112);

    // data directories
    for i in 0..16 {
        match i {
            1 => {
                // Import directory
                builder.push_u32_le(0x1000);
                builder.push_u32_le(40);
            }
            12 => {
                // Import Address Table
                builder.push_u32_le(0x1050);
                builder.push_u32_le(40);
            }
            _ => builder.push_u64_le(0),
        }
    }
    assert_eq!(builder.len(), 64 + 24 + 240);

    builder.push_slice(b".idata\0\0");
    builder.push_u32_le(0xC2);        // VirtualSize
    builder.push_u32_le(0x1000);      // VirtualAddress
    builder.push_u32_le(0x200);       // SizeOfRawData
    builder.push_u32_le(0x200);       // PointerToRawData
    builder.push_u32_le(0);
    builder.push_u32_le(0);
    builder.push_u16_le(0);
    builder.push_u16_le(0);
    builder.push_u32_le(0xC0000040);  // Initialized data, read, write
    assert_eq!(builder.len(), 64 + 24 + 240 + 40);

    // text section header
    builder.push_slice(b".text\0\0\0"); // name
    builder.push_u32_le(code_size); // virtual size
    builder.push_u32_le(0x2000); // virtual address
    builder.push_u32_le(code_size.next_multiple_of(512)); // size of raw data
    builder.push_u32_le(0x400); // pointer to raw data
    builder.push_u32_le(0); // pointer to relocations
    builder.push_u32_le(0); // pointer to line numbers
    builder.push_u16_le(0); // number of relocations
    builder.push_u16_le(0); // number of line numbers 
    builder.push_u32_le(0x60000020); // characteristics

    // alignment
    builder.push_slice(&[0; 104]);
    assert_eq!(builder.len(), start_size + 512);

    return builder.to_vec();
}
fn build_idata_section(buffer: Vec<u8>) -> Vec<u8> {
    let mut builder = BinaryBuilder::with_vec(buffer);
    let start_size = builder.len();

    // Import descriptor
    builder.push_u32_le(0x1028); // OriginalFirstThunk
    builder.push_u32_le(0);      // TimeDateStamp
    builder.push_u32_le(0);      // ForwarderChain
    builder.push_u32_le(0x1078); // Name
    builder.push_u32_le(0x1050); // FirstThunk

    // Null import descriptor
    builder.push_slice(&[0; 20]);
    
    // Import lookup table
    builder.push_u64_le(0x1086);
    builder.push_u64_le(0x1096);
    builder.push_u64_le(0x10A2);
    builder.push_u64_le(0x10B0);
    builder.push_u64_le(0);

    // 80
    // Import address table
    builder.push_u64_le(0x1086);
    builder.push_u64_le(0x1096);
    builder.push_u64_le(0x10A2);
    builder.push_u64_le(0x10B0);
    builder.push_u64_le(0);

    builder.push_slice(b"KERNEL32.dll\0");

    // Align to 2 bytes
    builder.push_u8_le(0);

    builder.push_u16_le(0);
    builder.push_slice(b"GetStdHandle\0");

    builder.push_u8_le(0);

    builder.push_u16_le(0);
    builder.push_slice(b"WriteFile\0");

    builder.push_u16_le(0);
    builder.push_slice(b"ExitProcess\0");

    builder.push_u16_le(0);
    builder.push_slice(b"GetCommandLineA\0");

    // Pad section to 512 bytes
    builder.push_slice(&[0; 0x200 - 0xC2]);
    assert_eq!(builder.len(), start_size + 512);

    return builder.to_vec();
}
struct BinaryBuilder {
    buffer: Vec<u8>,
}
impl BinaryBuilder {
    fn with_vec(vec: Vec<u8>) -> BinaryBuilder {
        BinaryBuilder { buffer: vec }
    }
    fn push_u8_le(&mut self, value: u8) {
        self.buffer.push(value);
    }
    fn push_u16_le(&mut self, value: u16) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }
    fn push_u32_le(&mut self, value: u32) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }
    fn push_u64_le(&mut self, value: u64) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }
    fn push_slice(&mut self, slice: &[u8]) {
        self.buffer.extend(slice);
    }
    fn len(&self) -> usize {
        self.buffer.len()
    }
    fn to_vec(self) -> Vec<u8> {
        self.buffer
    }
}
