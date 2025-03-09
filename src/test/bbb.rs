use crate::*;

#[test]
fn bbb() {
    const ENCODED: &[u8] = &[
        0x00, 0x00, 0x00, 0x1C, 0x66, 0x74, 0x79, 0x70, 0x69, 0x73, 0x6F, 0x36, 0x00, 0x00, 0x02,
        0x00, 0x69, 0x73, 0x6F, 0x36, 0x63, 0x6D, 0x66, 0x63, 0x6D, 0x70, 0x34, 0x31, 0x00, 0x00,
        0x05, 0x03, 0x6D, 0x6F, 0x6F, 0x76, 0x00, 0x00, 0x00, 0x6C, 0x6D, 0x76, 0x68, 0x64, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xE8,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x02, 0x15, 0x74, 0x72,
        0x61, 0x6B, 0x00, 0x00, 0x00, 0x5C, 0x74, 0x6B, 0x68, 0x64, 0x00, 0x00, 0x00, 0x03, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
        0x02, 0xD0, 0x00, 0x00, 0x00, 0x00, 0x01, 0xB1, 0x6D, 0x64, 0x69, 0x61, 0x00, 0x00, 0x00,
        0x20, 0x6D, 0x64, 0x68, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x5D, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x55, 0xC4, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x42, 0x68, 0x64, 0x6C, 0x72, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x76, 0x69, 0x64, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x28, 0x43, 0x29, 0x20, 0x32, 0x30, 0x30, 0x37, 0x20, 0x47, 0x6F, 0x6F, 0x67, 0x6C,
        0x65, 0x20, 0x49, 0x6E, 0x63, 0x2E, 0x20, 0x76, 0x30, 0x38, 0x2E, 0x31, 0x33, 0x2E, 0x32,
        0x30, 0x30, 0x37, 0x2E, 0x00, 0x00, 0x00, 0x01, 0x47, 0x6D, 0x69, 0x6E, 0x66, 0x00, 0x00,
        0x00, 0x14, 0x76, 0x6D, 0x68, 0x64, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x24, 0x64, 0x69, 0x6E, 0x66, 0x00, 0x00, 0x00, 0x1C,
        0x64, 0x72, 0x65, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x0C, 0x75, 0x72, 0x6C, 0x20, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0x07, 0x73, 0x74,
        0x62, 0x6C, 0x00, 0x00, 0x00, 0xBB, 0x73, 0x74, 0x73, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xAB, 0x61, 0x76, 0x63, 0x31, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x02, 0xD0, 0x00, 0x48, 0x00, 0x00, 0x00, 0x48,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0xFF, 0xFF, 0x00,
        0x00, 0x00, 0x31, 0x61, 0x76, 0x63, 0x43, 0x01, 0x64, 0x00, 0x1F, 0xFF, 0xE1, 0x00, 0x19,
        0x67, 0x64, 0x00, 0x1F, 0xAC, 0x24, 0x84, 0x01, 0x40, 0x16, 0xEC, 0x04, 0x40, 0x00, 0x00,
        0x03, 0x00, 0x40, 0x00, 0x00, 0x0C, 0x23, 0xC6, 0x0C, 0x92, 0x01, 0x00, 0x05, 0x68, 0xEE,
        0x32, 0xC8, 0xB0, 0x00, 0x00, 0x00, 0x10, 0x70, 0x61, 0x73, 0x70, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x14, 0x62, 0x74, 0x72, 0x74, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x1E, 0x62, 0x77, 0x00, 0x1E, 0x62, 0x77, 0x00, 0x00, 0x00, 0x10, 0x73, 0x74,
        0x74, 0x73, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x73,
        0x74, 0x73, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x14,
        0x73, 0x74, 0x73, 0x7A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x10, 0x73, 0x74, 0x63, 0x6F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x01, 0xD1, 0x74, 0x72, 0x61, 0x6B, 0x00, 0x00, 0x00, 0x5C, 0x74,
        0x6B, 0x68, 0x64, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x6D, 0x6D, 0x64, 0x69, 0x61, 0x00, 0x00, 0x00, 0x20, 0x6D, 0x64, 0x68, 0x64, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xAC, 0x44, 0x00,
        0x00, 0x00, 0x00, 0x55, 0xC4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x42, 0x68, 0x64, 0x6C, 0x72,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x73, 0x6F, 0x75, 0x6E, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x28, 0x43, 0x29, 0x20, 0x32, 0x30,
        0x30, 0x37, 0x20, 0x47, 0x6F, 0x6F, 0x67, 0x6C, 0x65, 0x20, 0x49, 0x6E, 0x63, 0x2E, 0x20,
        0x76, 0x30, 0x38, 0x2E, 0x31, 0x33, 0x2E, 0x32, 0x30, 0x30, 0x37, 0x2E, 0x00, 0x00, 0x00,
        0x01, 0x03, 0x6D, 0x69, 0x6E, 0x66, 0x00, 0x00, 0x00, 0x10, 0x73, 0x6D, 0x68, 0x64, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x24, 0x64, 0x69, 0x6E, 0x66,
        0x00, 0x00, 0x00, 0x1C, 0x64, 0x72, 0x65, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x0C, 0x75, 0x72, 0x6C, 0x20, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x00, 0xC7, 0x73, 0x74, 0x62, 0x6C, 0x00, 0x00, 0x00, 0x7B, 0x73, 0x74, 0x73, 0x64, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x6B, 0x6D, 0x70, 0x34, 0x61,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x02, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0xAC, 0x44, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x33, 0x65, 0x73, 0x64, 0x73, 0x00, 0x00, 0x00, 0x00, 0x03, 0x80, 0x80, 0x80, 0x22,
        0x00, 0x02, 0x00, 0x04, 0x80, 0x80, 0x80, 0x14, 0x40, 0x15, 0x00, 0x00, 0x00, 0x00, 0x01,
        0xEA, 0x93, 0x00, 0x01, 0xEA, 0x93, 0x05, 0x80, 0x80, 0x80, 0x02, 0x12, 0x10, 0x06, 0x80,
        0x80, 0x80, 0x01, 0x02, 0x00, 0x00, 0x00, 0x14, 0x62, 0x74, 0x72, 0x74, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x01, 0xEA, 0x93, 0x00, 0x01, 0xEA, 0x93, 0x00, 0x00, 0x00, 0x10, 0x73, 0x74,
        0x74, 0x73, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x73,
        0x74, 0x73, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x14,
        0x73, 0x74, 0x73, 0x7A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x10, 0x73, 0x74, 0x63, 0x6F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x48, 0x6D, 0x76, 0x65, 0x78, 0x00, 0x00, 0x00, 0x20, 0x74,
        0x72, 0x65, 0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x20, 0x74, 0x72, 0x65, 0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00,
        0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x61, 0x75, 0x64, 0x74, 0x61, 0x00, 0x00, 0x00, 0x59, 0x6D, 0x65, 0x74, 0x61,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0x68, 0x64, 0x6C, 0x72, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x6D, 0x64, 0x69, 0x72, 0x61, 0x70, 0x70, 0x6C, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2C, 0x69, 0x6C, 0x73, 0x74,
        0x00, 0x00, 0x00, 0x24, 0xA9, 0x74, 0x6F, 0x6F, 0x00, 0x00, 0x00, 0x1C, 0x64, 0x61, 0x74,
        0x61, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x4C, 0x61, 0x76, 0x66, 0x36, 0x31,
        0x2E, 0x31, 0x2E, 0x31, 0x30, 0x30, 0x00, 0x00, 0x00, 0x6C, 0x6D, 0x6F, 0x6F, 0x66, 0x00,
        0x00, 0x00, 0x10, 0x6D, 0x66, 0x68, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x54, 0x74, 0x72, 0x61, 0x66, 0x00, 0x00, 0x00, 0x20, 0x74, 0x66, 0x68,
        0x64, 0x00, 0x02, 0x00, 0x3A, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x03, 0xE8, 0x00, 0x00, 0x00, 0xD7, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x14, 0x74,
        0x66, 0x64, 0x74, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x18, 0x74, 0x72, 0x75, 0x6E, 0x01, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x74, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xDF, 0x6D, 0x64,
        0x61, 0x74, 0x00, 0x00, 0x00, 0xD3, 0x65, 0x88, 0x80, 0x80, 0x03, 0x3F, 0xFE, 0xF5, 0xF8,
        0x45, 0x4F, 0x32, 0xCB, 0x1B, 0xB4, 0x20, 0x3F, 0x85, 0x4D, 0xD6, 0x9B, 0xC2, 0xCA, 0x91,
        0xB2, 0xBC, 0xE1, 0xFB, 0x35, 0x27, 0x44, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00,
        0x03, 0x00, 0x00, 0x03, 0x00, 0x50, 0x99, 0x98, 0x41, 0xD1, 0xAF, 0xD3, 0x24, 0xAE, 0xA0,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x0F, 0x60, 0x00, 0x11, 0xC0, 0x00, 0x1B, 0x40, 0x00, 0x4E,
        0x40, 0x01, 0x1F, 0x00, 0x03, 0xB8, 0x00, 0x10, 0x80, 0x00, 0x59, 0x00, 0x02, 0x38, 0x00,
        0x0B, 0xE0, 0x00, 0x5E, 0x00, 0x02, 0x20, 0x00, 0x11, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x40, 0x41, 0x00, 0x00, 0x00, 0x68, 0x6D, 0x6F, 0x6F, 0x66,
        0x00, 0x00, 0x00, 0x10, 0x6D, 0x66, 0x68, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x02, 0x00, 0x00, 0x00, 0x50, 0x74, 0x72, 0x61, 0x66, 0x00, 0x00, 0x00, 0x20, 0x74, 0x66,
        0x68, 0x64, 0x00, 0x02, 0x00, 0x3A, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00,
        0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x09, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x14,
        0x74, 0x66, 0x64, 0x74, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x14, 0x74, 0x72, 0x75, 0x6E, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00,
        0x00, 0x01, 0x00, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 0x11, 0x6D, 0x64, 0x61, 0x74, 0x21,
        0x00, 0x49, 0x90, 0x02, 0x19, 0x00, 0x23, 0x80, 0x00, 0x00, 0x00, 0x68, 0x6D, 0x6F, 0x6F,
        0x66, 0x00, 0x00, 0x00, 0x10, 0x6D, 0x66, 0x68, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x03, 0x00, 0x00, 0x00, 0x50, 0x74, 0x72, 0x61, 0x66, 0x00, 0x00, 0x00, 0x20, 0x74,
        0x66, 0x68, 0x64, 0x00, 0x02, 0x00, 0x3A, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x09, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x14, 0x74, 0x66, 0x64, 0x74, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x04, 0x00, 0x00, 0x00, 0x00, 0x14, 0x74, 0x72, 0x75, 0x6E, 0x01, 0x00, 0x00, 0x01, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 0x11, 0x6D, 0x64, 0x61, 0x74,
        0x21, 0x20, 0x49, 0x90, 0x02, 0x19, 0x00, 0x23, 0x80, 0x00, 0x00, 0x00, 0x68, 0x6D, 0x6F,
        0x6F, 0x66, 0x00, 0x00, 0x00, 0x10, 0x6D, 0x66, 0x68, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x50, 0x74, 0x72, 0x61, 0x66, 0x00, 0x00, 0x00, 0x20,
        0x74, 0x66, 0x68, 0x64, 0x00, 0x02, 0x00, 0x3A, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x03, 0xE8, 0x00, 0x00, 0x00, 0x27, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x14, 0x74, 0x66, 0x64, 0x74, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x03, 0xE8, 0x00, 0x00, 0x00, 0x14, 0x74, 0x72, 0x75, 0x6E, 0x01, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 0x2F, 0x6D, 0x64, 0x61,
        0x74, 0x00, 0x00, 0x00, 0x23, 0x41, 0x9A, 0x04, 0x0E, 0x43, 0x3F, 0xFD, 0xF1, 0x00, 0x00,
        0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00,
        0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x06, 0xF4, 0x00, 0x00, 0x00, 0x68, 0x6D,
        0x6F, 0x6F, 0x66, 0x00, 0x00, 0x00, 0x10, 0x6D, 0x66, 0x68, 0x64, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x50, 0x74, 0x72, 0x61, 0x66, 0x00, 0x00, 0x00,
        0x20, 0x74, 0x66, 0x68, 0x64, 0x00, 0x02, 0x00, 0x3A, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00,
        0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x02, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x14, 0x74, 0x66, 0x64, 0x74, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x14, 0x74, 0x72, 0x75, 0x6E, 0x01, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 0x6C, 0x6D, 0x6F,
        0x6F, 0x66, 0x00, 0x00, 0x00, 0x10, 0x6D, 0x66, 0x68, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x54, 0x74, 0x72, 0x61, 0x66, 0x00, 0x00, 0x00, 0x20,
        0x74, 0x66, 0x68, 0x64, 0x00, 0x02, 0x00, 0x3A, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x03, 0xE8, 0x00, 0x00, 0x00, 0xD7, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x14, 0x74, 0x66, 0x64, 0x74, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x74, 0x72, 0x75, 0x6E, 0x01, 0x00, 0x00, 0x05,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x74, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xDF, 0x6D, 0x64, 0x61, 0x74, 0x00, 0x00, 0x00, 0xD3, 0x65, 0x88, 0x80, 0x80, 0x03, 0x3F,
        0xFE, 0xF5, 0xF8, 0x45, 0x4F, 0x32, 0xCB, 0x1B, 0xB4, 0x20, 0x3F, 0x85, 0x4D, 0xD6, 0x9B,
        0xC2, 0xCA, 0x91, 0xB2, 0xBC, 0xE1, 0xFB, 0x35, 0x27, 0x44, 0x00, 0x00, 0x03, 0x00, 0x00,
        0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x50, 0x99, 0x98, 0x41, 0xD1, 0xAF, 0xD3,
        0x24, 0xAE, 0xA0, 0x00, 0x00, 0x03, 0x00, 0x00, 0x0F, 0x60, 0x00, 0x11, 0xC0, 0x00, 0x1B,
        0x40, 0x00, 0x4E, 0x40, 0x01, 0x1F, 0x00, 0x03, 0xB8, 0x00, 0x10, 0x80, 0x00, 0x59, 0x00,
        0x02, 0x38, 0x00, 0x0B, 0xE0, 0x00, 0x5E, 0x00, 0x02, 0x20, 0x00, 0x11, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
        0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x40, 0x41,
    ];

    let mut buf = &mut std::io::Cursor::new(&ENCODED);

    let ftyp = Ftyp::decode(buf).expect("failed to decode ftyp");

    assert_eq!(
        ftyp,
        Ftyp {
            major_brand: b"iso6".into(),
            minor_version: 512,
            compatible_brands: vec![b"iso6".into(), b"cmfc".into(), b"mp41".into()],
        }
    );

    let moov = Moov::decode(buf).expect("failed to decode moov");
    assert_eq!(
        moov,
        Moov {
            mvhd: Mvhd {
                timescale: 1000,
                rate: 1.into(),
                volume: 1.into(),
                next_track_id: 2,
                ..Default::default()
            },
            mvex: Some(Mvex {
                trex: vec![
                    Trex {
                        track_id: 1,
                        default_sample_description_index: 1,
                        ..Default::default()
                    },
                    Trex {
                        track_id: 2,
                        default_sample_description_index: 1,
                        ..Default::default()
                    }
                ],
                ..Default::default()
            }),
            trak: vec![Trak {
                tkhd: Tkhd {
                    track_id: 1,
                    enabled: true,
                    width: 1280.into(),
                    height: 720.into(),
                    ..Default::default()
                },
                mdia: Mdia {
                    mdhd: Mdhd {
                        timescale: 24000,
                        language: "und".into(),
                        ..Default::default()
                    },
                    hdlr: Hdlr {
                        handler: b"vide".into(),
                        name: "(C) 2007 Google Inc. v08.13.2007.".into(),
                    },
                    minf: Minf {
                        smhd: None,
                        vmhd: Vmhd {
                            ..Default::default()
                        }
                        .into(),
                        dinf: Dinf {
                            dref: Dref {
                                urls: vec![Url {
                                    location: "".into(),
                                }],
                            },
                        },
                        stbl: Stbl {
                            stsd: Stsd {
                                codecs: vec![Avc1 {
                                    visual: Visual {
                                    data_reference_index: 1,
                                    width: 1280,
                                    height: 720,
                                    horizresolution: 72.into(),
                                    vertresolution: 72.into(),
                                    frame_count: 1,
                                    compressor: "".into(),
                                    depth: 24,
                                    },
                                    avcc: Avcc {
                                        configuration_version: 1,
                                        avc_profile_indication: 100,
                                        profile_compatibility: 0,
                                        avc_level_indication: 31,
                                        length_size: 4,
                                        sequence_parameter_sets: vec![b"gd\0\x1f\xac$\x84\x01@\x16\xec\x04@\0\0\x03\0@\0\0\x0c#\xc6\x0c\x92".into()],
                                        picture_parameter_sets:  vec![b"h\xee2\xc8\xb0".into()],
                                        ext: None,
                                    },
                                }
                                .into()],
                            },
                            stts: Stts {
                                ..Default::default()
                            },
                            stsc: Stsc {
                                ..Default::default()
                            },
                            stsz: Stsz {
                                ..Default::default()
                            },
                            stco: Some(Stco { ..Default::default() }),
                            ..Default::default()
                        },
                    },
                },
                ..Default::default()
            },
            Trak {
                tkhd: Tkhd {
                    track_id: 2,
                    alternate_group: 1,
                    enabled: true,
                    volume: 1.into(),
                    ..Default::default()
                },
                mdia: Mdia {
                    mdhd: Mdhd {
                        timescale: 44100,
                        language: "und".into(),
                        ..Default::default()
                    },
                    hdlr: Hdlr {
                        handler: b"soun".into(),
                        name: "(C) 2007 Google Inc. v08.13.2007.".into(),
                    },
                    minf: Minf {
                        smhd: Some(Smhd {
                            ..Default::default()
                        }),
                        dinf: Dinf {
                            dref: Dref {
                                urls: vec![Url {
                                    location: "".into(),
                                }],
                            },
                        },
                        stbl: Stbl {
                            stsd: Stsd {
                                codecs: vec![Mp4a {
                                    data_reference_index: 1,
                                    channelcount: 2,
                                    samplesize: 16,
                                    samplerate: 44100.into(),
                                    esds: Some(Esds {
                                        es_desc: esds::EsDescriptor {
                                            es_id: 2,
                                            dec_config: esds::DecoderConfig{
                                                object_type_indication: 64,
                                                stream_type: 5,
                                                max_bitrate: 125587,
                                                avg_bitrate: 125587,
                                                dec_specific: esds::DecoderSpecific {
                                                    profile: 2,
                                                    freq_index: 4,
                                                    chan_conf: 2,
                                                },
                                                ..Default::default()
                                            },
                                            sl_config: esds::SLConfig{},
                                        },
                                    }),
                                }
                                .into()],
                            },
                            stts: Stts {
                                ..Default::default()
                            },
                            stsc: Stsc {
                                ..Default::default()
                            },
                            stsz: Stsz {
                                ..Default::default()
                            },
                            stco: Some(Stco { ..Default::default() }),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                ..Default::default()
            }],
            udta: Some(Udta {
                meta: Some(Meta::Mdir {
                    ilst: Some(Ilst {
                        ..Default::default()
                    }),
                }),
                ..Default::default()
            }),

            ..Default::default()
        },
    );

    let moof = Moof::decode(&mut buf).expect("failed to decode moof");
    assert_eq!(
        moof,
        Moof {
            mfhd: Mfhd { sequence_number: 1 },
            traf: vec![Traf {
                tfhd: Tfhd {
                    track_id: 1,
                    sample_description_index: 1.into(),
                    default_sample_duration: 1000.into(),
                    default_sample_flags: 0x1010000.into(),
                    default_sample_size: 215.into(),
                    ..Default::default()
                },
                tfdt: Some(Tfdt {
                    ..Default::default()
                }),
                trun: Some(Trun {
                    data_offset: 116.into(),
                    entries: vec![TrunEntry {
                        flags: Some(33554432),
                        ..Default::default()
                    }],
                }),
            }],
        },
    );

    let mdat = Mdat::decode(&mut buf).expect("failed to decode mdat");
    assert_eq!(
        mdat,
        Mdat {
            data: vec![
                0x00, 0x00, 0x00, 0xD3, 0x65, 0x88, 0x80, 0x80, 0x03, 0x3F, 0xFE, 0xF5, 0xF8, 0x45,
                0x4F, 0x32, 0xCB, 0x1B, 0xB4, 0x20, 0x3F, 0x85, 0x4D, 0xD6, 0x9B, 0xC2, 0xCA, 0x91,
                0xB2, 0xBC, 0xE1, 0xFB, 0x35, 0x27, 0x44, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00,
                0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x50, 0x99, 0x98, 0x41, 0xD1, 0xAF, 0xD3, 0x24,
                0xAE, 0xA0, 0x00, 0x00, 0x03, 0x00, 0x00, 0x0F, 0x60, 0x00, 0x11, 0xC0, 0x00, 0x1B,
                0x40, 0x00, 0x4E, 0x40, 0x01, 0x1F, 0x00, 0x03, 0xB8, 0x00, 0x10, 0x80, 0x00, 0x59,
                0x00, 0x02, 0x38, 0x00, 0x0B, 0xE0, 0x00, 0x5E, 0x00, 0x02, 0x20, 0x00, 0x11, 0x00,
                0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
                0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00,
                0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00,
                0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
                0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00,
                0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00,
                0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03,
                0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00, 0x03, 0x00, 0x00,
                0x03, 0x00, 0x00, 0x40, 0x41,
            ],
        }
    );

    let moof = Moof::decode(&mut buf).expect("failed to decode moof");
    assert_eq!(
        moof,
        Moof {
            mfhd: Mfhd { sequence_number: 2 },
            traf: vec![Traf {
                tfhd: Tfhd {
                    track_id: 2,
                    sample_description_index: 1.into(),
                    default_sample_duration: 1024.into(),
                    default_sample_flags: 0x2000000.into(),
                    default_sample_size: 9.into(),
                    ..Default::default()
                },
                tfdt: Some(Tfdt {
                    ..Default::default()
                }),
                trun: Some(Trun {
                    data_offset: 112.into(),
                    entries: vec![Default::default()],
                }),
            }],
        },
    );

    let mdat = Mdat::decode(&mut buf).expect("failed to decode mdat");
    assert_eq!(
        mdat,
        Mdat {
            data: vec![0x21, 0x00, 0x49, 0x90, 0x02, 0x19, 0x00, 0x23, 0x80],
        }
    );

    let mut buf = Vec::new();

    ftyp.encode(&mut buf).expect("failed to encode ftyp");
    moov.encode(&mut buf).expect("failed to encode moov");
    moof.encode(&mut buf).expect("failed to encode moof");
    mdat.encode(&mut buf).expect("failed to encode mdat");

    // One day:
    // assert_eq!(buf, ENCODED);
}
