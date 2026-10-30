#![allow(unused)]

use std::fs::write;

#[test]
fn test1() {
    build();
}

pub fn build() {
    let mut builder = BinaryBuilder::with_capacity(1024);

    // DOS header
    builder.push_u16_le(0x5A4D); // magic
    builder.push_slice(&[0; 58]);
    builder.push_u32_le(64); // PE header offset
    assert_eq!(builder.len(), 64);

    // PE header
    builder.push_u32_le(0x00_00_45_50); // magic
    builder.push_u16_le(0x8664); // machine
    builder.push_u16_le(1); // number of sections
    builder.push_u32_le(0); // timestamp
    builder.push_u32_le(0); // pointer to symbol table
    builder.push_u32_le(0); // number of symbols
    builder.push_u16_le(240); // size of optional header
    builder.push_u16_le(0x22); // characteristics
    assert_eq!(builder.len(), 64 + 24);

    // optional header
    builder.push_u16_le(0x20B); // magic PE32+
    builder.push_u8_le(0); // linker major version
    builder.push_u8_le(0); // linker minor version
    builder.push_u32_le(512); // size of code section
    builder.push_u32_le(0); // size of data section
    builder.push_u32_le(0); // size of uninit data section
    builder.push_u32_le(0x1000); // address of entry point
    builder.push_u32_le(0x1000); // base of code
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
    builder.push_u32_le(0x2000); // size of image
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
    builder.push_u64_le(0); // export table
    builder.push_u64_le(0); // import table
    builder.push_u64_le(0); // resource table
    builder.push_u64_le(0); // exception table
    builder.push_u64_le(0); // certificate table 
    builder.push_u64_le(0); // base relocation table
    builder.push_u64_le(0); // debug
    builder.push_u64_le(0); // architecture, must be zero
    builder.push_u64_le(0); // global ptr
    builder.push_u64_le(0); // thread local storage table
    builder.push_u64_le(0); // load config table
    builder.push_u64_le(0); // bound import table
    builder.push_u64_le(0); // import address table
    builder.push_u64_le(0); // delay import descriptor
    builder.push_u64_le(0); // CLR runtime header
    builder.push_u64_le(0); // must be zero
    assert_eq!(builder.len(), 64 + 24 + 240);

    // text section header
    builder.push_slice(b".text\0\0\0"); // name
    builder.push_u32_le(2); // virtual size
    builder.push_u32_le(0x1000); // virtual address
    builder.push_u32_le(0x200); // size of raw data
    builder.push_u32_le(0x200); // pointer to raw data
    builder.push_u32_le(0); // pointer to relocations
    builder.push_u32_le(0); // pointer to line numbers
    builder.push_u16_le(0); // number of relocations
    builder.push_u16_le(0); // number of line numbers 
    builder.push_u32_le(0x60000020); // characteristics
    assert_eq!(builder.len(), 64 + 24 + 240 + 40);

    // alignment
    builder.push_slice(&[0; 144]);
    assert_eq!(builder.len(), 512);

    // infinite loop instructions
    builder.push_slice(&[0xEB, 0xFE]);

    // alignment
    builder.push_slice(&[0; 510]);
    assert_eq!(builder.len(), 1024);

    write("./testexec.exe", builder.as_slice()).unwrap();
}
struct BinaryBuilder {
    buffer: Vec<u8>,
}
impl BinaryBuilder {
    fn with_capacity(capacity: usize) -> BinaryBuilder {
        BinaryBuilder { buffer: Vec::with_capacity(capacity) }
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
        self.buffer.extend_from_slice(slice);
    }
    fn len(&self) -> usize {
        self.buffer.len()
    }
    fn as_slice(&self) -> &[u8] {
        &self.buffer
    }
}