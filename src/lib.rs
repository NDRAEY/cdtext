use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

/// Main parser structure.
pub struct CDText<'data> {
    _length: usize,
    data: &'data [u8],
}

/// The pack type
#[derive(Debug, FromPrimitive, PartialEq, Clone, Copy)]
pub enum CDTextPackType {
    Title = 0x80,
    Performers = 0x81,
    Songwriters = 0x82,
    Composers = 0x83,
    Arrangers = 0x84,
    Message = 0x85,
    DiscID = 0x86,
    Genre = 0x87,
    TOC = 0x88,
    AdditionalTOC = 0x89,
    ClosedInfo = 0x8d,
    Code = 0x8e,
    BlockSizeInfo = 0x8f,
}

/// Track number entry referring to.
/// Entry can refer to whole album or on separate track in it.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CDTextTrackNumber {
    WholeAlbum,
    Track(u8),
}

/// A pack itself.
#[derive(Debug, Clone)]
pub struct CDTextPack {
    pub pack_type: CDTextPackType,
    pub track_number: CDTextTrackNumber,
    pub seq_counter: u8,
    pub character_position: u8,
    pub block_number: u8,
    pub is_double_byte_characters: bool,

    pub payload: [u8; 12],
    pub crc: u16,
}

/// Data can be represented as string or raw data.
#[derive(Debug, Clone)]
pub enum CDTextEntryDataType {
    String(String),
    Data(Vec<u8>),
}

/// The processed entry.
#[derive(Debug, Clone)]
pub struct CDTextEntry {
    pub track_number: CDTextTrackNumber,
    pub entry_type: CDTextPackType,
    pub data: CDTextEntryDataType,
}

impl<'data> CDText<'data> {
    /// Creates a parser from data, assuming that first 4 bytes are used for service info.
    /// First two bytes are the data length minus two.
    pub fn from_data_with_length(data: &'data [u8]) -> Self {
        Self {
            _length: (((data[0] as usize) << 8) | (data[1] as usize)) - 2,
            data: &data[4..],
        }
    }

    /// Creates a parser from data.
    pub fn from_data(data: &'data [u8]) -> Self {
        Self {
            _length: data.len(),
            data,
        }
    }

    /// Internal method. Parses a separate pack from data.
    /// Data (sub)slice must be 18 bytes long.
    fn parse_pack(&self, subdata: &[u8]) -> Option<CDTextPack> {
        debug_assert!(subdata.len() == 18);

        // The first byte of each pack contains the pack type.
        let pack_type = CDTextPackType::from_u8(subdata[0])?;

        // The second byte often gives the track number of the pack.
        let track_number = match subdata[1] {
            // However, a zero track value indicates that the information pertains to the whole album.
            0 => CDTextTrackNumber::WholeAlbum,
            n => CDTextTrackNumber::Track(n),
        };

        // The third byte is a sequential counter.
        let seq_counter = subdata[2];

        // bits 0-3: Character position.
        let character_position = subdata[3] & 0b1111;

        // bits 4-6: Block number
        let block_nr = (subdata[3] >> 4) & 0b111;

        // bit 7: Is 0 if single byte characters, 1 if double-byte characters.
        let is_double_byte_chars = ((subdata[3] >> 7) & 1) != 0;

        let payload = &subdata[4..16];

        let crc = u16::from_be_bytes(subdata[16..18].try_into().unwrap());

        Some(CDTextPack {
            pack_type,
            track_number,
            seq_counter,
            character_position,
            block_number: block_nr,
            is_double_byte_characters: is_double_byte_chars,
            payload: payload.try_into().unwrap(),
            crc,
        })
    }

    /// Wrapper method.
    pub fn iter_pack_chunks(&self) -> impl Iterator<Item = Option<CDTextPack>> {
        // Each pack consists of a 4-byte header, 12 bytes of payload, and 2 bytes of CRC.
        // 4 + 12 + 2 = 18
        self.data.chunks(18).map(|x| self.parse_pack(x))
    }

    /// Parses all the entries from the data and returns a Vec with parsed entries.
    pub fn parse(&self) -> Vec<CDTextEntry> {
        let mut payload_buffer: Vec<u8> = Vec::with_capacity(16);
        let mut prev_pack = self.iter_pack_chunks().next().unwrap().unwrap();

        let mut parsed_data: Vec<CDTextEntry> = vec![];

        for pack in self.iter_pack_chunks().skip(1) {
            let pack = pack.as_ref().unwrap();

            // let index = if pack.character_position <= 12 {
            //     12 - pack.character_position
            // } else {
            //     0
            // } as usize;

            let index = 12u8.saturating_sub(pack.character_position) as usize;

            match pack.pack_type {
                CDTextPackType::Arrangers
                | CDTextPackType::Composers
                | CDTextPackType::Title
                | CDTextPackType::Performers
                | CDTextPackType::Songwriters => {
                    let mut track_number = prev_pack.track_number;
                    let mut before = &prev_pack.payload[..index];
                    let after = &prev_pack.payload[index..];

                    let is_terminal = before.ends_with(&[0]);

                    // I don't know why 2.
                    // More than one nul-terminated strings can be encountered in one entry (usually in short strings).
                    // So we need to handle it somehow.
                    if before.iter().filter(|&x| *x == 0).count() == 2 {
                        // println!("===== INCREMENT! {before:?}");

                        let position = before.iter().position(|&x| x == 0).unwrap();
                        payload_buffer.extend_from_slice(&before[..position]);

                        if !payload_buffer.is_empty() {
                            // println!("===== PAYLOAD: {payload_buffer:?}");

                            parsed_data.push(CDTextEntry {
                                track_number,
                                entry_type: prev_pack.pack_type,
                                data: CDTextEntryDataType::String(
                                    str::from_utf8(&payload_buffer).unwrap().to_owned(),
                                ),
                            });
                        } else {
                            parsed_data.push(CDTextEntry {
                                track_number,
                                entry_type: prev_pack.pack_type,
                                data: CDTextEntryDataType::String(
                                    str::from_utf8(&payload_buffer).unwrap().to_owned(),
                                ),
                            });
                        }

                        payload_buffer.clear();

                        before = &before[before.iter().position(|&x| x == 0).unwrap_or(0) + 1..];

                        if let CDTextTrackNumber::Track(nr) = track_number {
                            track_number = CDTextTrackNumber::Track(nr + 1);
                        } else if let CDTextTrackNumber::WholeAlbum = track_number {
                            track_number = CDTextTrackNumber::Track(1);
                        }
                    }

                    payload_buffer.extend_from_slice(if is_terminal {
                        let len = before.iter().rev().position(|x| *x != 0);

                        if let Some(ix) = len {
                            &before[..before.len() - ix]
                        } else {
                            before
                        }
                    } else {
                        before
                    });

                    // println!("Before: {before:?}");
                    // println!("After: {after:?}");

                    // println!("{:x?} ({:?} / {index})", pack, unsafe {
                    //     str::from_utf8_unchecked(&payload_buffer)
                    // });

                    if is_terminal {
                        parsed_data.push(CDTextEntry {
                            track_number,
                            entry_type: prev_pack.pack_type,
                            data: CDTextEntryDataType::String(
                                str::from_utf8(&payload_buffer).unwrap().trim_end_matches(|x| x as u32 == 0).to_owned(),
                            ),
                        });

                        payload_buffer.clear();
                    }

                    payload_buffer.extend_from_slice(after);
                }
                _ => {
                    break;
                },
            };

            prev_pack = pack.clone();
        }

        // println!("[{payload_buffer:?}]: Prev pack: {prev_pack:?}");

        payload_buffer.extend_from_slice(&prev_pack.payload[..prev_pack.payload.iter().position(|&x| x == 0).unwrap()]);

        parsed_data.push(CDTextEntry {
            track_number: prev_pack.track_number,
            entry_type: prev_pack.pack_type,
            data: CDTextEntryDataType::String(
                str::from_utf8(&payload_buffer).unwrap().to_owned(),
            ),
        });

        // println!("Length is: {}", self.length);

        // for i in parsed_data {
        //     println!("{:?} => {:?}", i.track_number, i.data);
        // }

        parsed_data
    }
}
