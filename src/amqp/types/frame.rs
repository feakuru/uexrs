use std::io::Read;

use super::constructor::Constructor;

pub struct Frame {
    header: [u8; 8],
    extended_header: Vec<u8>,
    frame_body: Vec<u8>,
}

impl Frame {
    pub fn size(&self) -> u32 {
        u32::from_be_bytes([
            self.header[0],
            self.header[1],
            self.header[2],
            self.header[3],
        ])
    }

    pub fn doff(&self) -> u8 {
        self.header[4]
    }

    pub fn frame_type(&self) -> u8 {
        self.header[5]
    }

    pub fn new(buf_reader: &mut impl Read) -> Result<Self, &'static str> {
        let mut frame = Frame {
            header: [0u8; 8],
            extended_header: vec![],
            frame_body: vec![],
        };
        buf_reader
            .read_exact(&mut frame.header)
            .unwrap_or_else(|_| {});
        let mut buffer = [0u8; 1];
        let ext_header_size = u32::from(frame.doff() * 4);
        for _ in 8..ext_header_size {
            buf_reader.read_exact(&mut buffer).unwrap_or_else(|_| {});
            frame.extended_header.push(buffer[0]);
        }
        for _ in ext_header_size..frame.size() {
            buf_reader.read_exact(&mut buffer).unwrap_or_else(|_| {});
            frame.frame_body.push(buffer[0]);
        }
        Ok(frame)
    }
}
