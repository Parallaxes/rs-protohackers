pub enum Request {
    Insert { key: Vec<u8>, value: Vec<u8> },
    Retrieve { key: Vec<u8> },
}

#[derive(Debug)]
pub enum ProtocolError {
    TooLong,
    InvalidKey,
}

pub const MAX_PACKET_SIZE: usize = 1000;

pub fn parse_request(packet: &[u8]) -> Result<Request, ProtocolError> {
    if packet.len() > MAX_PACKET_SIZE {
        return Err(ProtocolError::TooLong);
    }

    if let Some(eq_pos) = packet.iter().position(|&b| b == b'=') {
        let key = &packet[..eq_pos];
        let value = &packet[eq_pos + 1..];
        if key.iter().any(|&b| b == b'=') {
            return Err(ProtocolError::InvalidKey);
        }

        Ok(Request::Insert {
            key: key.to_vec(),
            value: value.to_vec(),
        })
    } else {
        Ok(Request::Retrieve {
            key: packet.to_vec(),
        })
    }
}

pub fn format_response(key: &[u8], value: &[u8]) -> Vec<u8> {
    let mut resp = Vec::with_capacity(key.len() + 1 + value.len());
    resp.extend_from_slice(key);
    resp.push(b'=');
    resp.extend_from_slice(value);
    resp
}
