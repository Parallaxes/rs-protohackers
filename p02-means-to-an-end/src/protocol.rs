pub enum Message {
    Insert { timestamp: i32, price: i32 },
    Query { mintime: i32, maxtime: i32 },
}

impl Message {
    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() != 9 {
            return None;
        }

        match buf[0] {
            b'I' => Some(Self::Insert {
                timestamp: i32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]),
                price: i32::from_be_bytes([buf[5], buf[6], buf[7], buf[8]]),
            }),
            b'Q' => Some(Self::Query {
                mintime: i32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]),
                maxtime: i32::from_be_bytes([buf[5], buf[6], buf[7], buf[8]]),
            }),
            _ => None,
        }
    }
}

pub fn serialize_mean(mean: i32) -> [u8; 4] {
    mean.to_be_bytes()
}
