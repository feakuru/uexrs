use tokio::io::AsyncReadExt;

pub enum FrameType {
    AMQP = 0x00,
    SASL = 0x05,
}

pub struct Frame {
    pub size: u32,
    pub doff: u8,
    pub frame_type: FrameType,
    pub type_specific: [u8; 2],
    pub extended_header: Vec<u8>,
    pub frame_body: Vec<u8>,
}

impl Frame {
    pub async fn new(buf_reader: &mut (impl AsyncReadExt + Unpin)) -> Result<Self, &'static str> {
        let mut buffer = [0u8; 4];
        buf_reader
            .read_exact(&mut buffer)
            .await
            .unwrap_or_else(|_| 0);
        let frame_size = u32::from_be_bytes(buffer);
        let mut buffer = [0u8; 1];
        buf_reader
            .read_exact(&mut buffer)
            .await
            .unwrap_or_else(|_| 0);
        let doff = buffer[0];
        let mut buffer = [0u8; 1];
        buf_reader
            .read_exact(&mut buffer)
            .await
            .unwrap_or_else(|_| 0);
        let frame_type = match buffer[0] {
            0x00 => FrameType::AMQP,
            0x05 => FrameType::SASL,
            _ => return Err("Unexpected frame type"),
        };
        let mut buffer = [0u8; 2];
        buf_reader
            .read_exact(&mut buffer)
            .await
            .unwrap_or_else(|_| 0);
        let mut frame = Frame {
            size: frame_size,
            doff,
            frame_type,
            type_specific: buffer,
            extended_header: vec![],
            frame_body: vec![],
        };
        for _ in 8..frame.doff * 4 {
            let mut buffer = [0u8; 1];
            buf_reader
                .read_exact(&mut buffer)
                .await
                .unwrap_or_else(|_| 0);
            frame.extended_header.push(buffer[0]);
        }
        for _ in (frame.doff as u32) * 4..frame.size {
            let mut buffer = [0u8; 1];
            buf_reader
                .read_exact(&mut buffer)
                .await
                .unwrap_or_else(|_| 0);
            frame.frame_body.push(buffer[0]);
        }
        Ok(frame)
    }
}
