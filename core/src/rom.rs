use log::*;

#[derive(Debug, Clone)]
pub struct Rom {
    header: Header,
    payload: Vec<u8>,
}

impl Rom {
    pub fn new(rom: &[u8]) -> Self {
        let header = Header::new(rom);
        let offset = if header.trainer { 16 + 512 } else { 16 };
        let payload = &rom[offset..];

        assert!(
            payload.len() >= header.prg_len() + header.chr_len(),
            "Rom file is too short"
        );

        trace!("Header: {:#?}", header);

        Self {
            header,
            payload: payload.to_vec(),
        }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn prg(&self) -> &[u8] {
        &self.payload[0..self.header.prg_len()]
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    prg_len: usize,
    chr_len: usize,
    trainer: bool,
    // TODO: Flags
}

impl Header {
    pub fn new(rom: &[u8]) -> Self {
        assert_eq!(rom[0..4], [0x4e, 0x45, 0x53, 0x1a], "Invalid header");

        Self {
            prg_len: rom[4] as usize * 16 * 1024,
            chr_len: rom[5] as usize * 8 * 1024,
            trainer: (rom[6] & 0x04) != 0,
        }
    }

    pub fn prg_len(&self) -> usize {
        self.prg_len
    }

    pub fn chr_len(&self) -> usize {
        self.chr_len
    }
}
