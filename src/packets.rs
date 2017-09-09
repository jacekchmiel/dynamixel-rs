use std;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    PacketTooShort,
    MalformedPacket,
    InvalidCrc
}

#[derive(Debug)]
pub enum Request {
    Ping { id: u8 },
    Read { id: u8, addr: u8, len: u8 },
    Write { id: u8, addr: u8, data: Vec<u8> },
}

#[derive(Debug, PartialEq)]
pub struct Status {
    pub id: u8,
    pub error: u8,
    pub data: Vec<u8>
}

impl Status {
    pub fn from_bytes(serialized: &[u8]) -> Result<Status> {
        if !Status::is_constructible_from(serialized) {
            return Err(Error::PacketTooShort);
        }

        let len = serialized.len();
        let actual_crc = crc(&serialized[0..len - 1]);
        let declared_crc = serialized[len - 1];
        if declared_crc != actual_crc {
            return Err(Error::InvalidCrc);
        }

        let d = match len {
            len if len > 4 => serialized[5..len - 1].to_vec(),
            _ => vec![],
        };
        Ok(Status { id: serialized[2], error: serialized[4], data: d })
    }

    pub fn is_constructible_from(serialized: &[u8]) -> bool {
        let len = serialized.len();
        match Status::extract_declared_length(serialized) {
            Some(declared_len) => {
                len == declared_len
            }
            None => {
                false
            }
        }
    }

    fn extract_declared_length(serialized: &[u8]) -> Option<usize> {
        let len = serialized.len();
        if len < 6 {
            None
        } else {
            Some((serialized[3] + 4) as usize)
        }
    }
}

impl Request {
    fn id_byte(&self) -> u8 {
        match *self {
            Request::Ping { id } | Request::Write { id, .. } | Request::Read { id, .. } => id,
        }
    }

    fn instruction_byte(&self) -> u8 {
        match *self {
            Request::Ping { .. } => 0x01,
            Request::Read { .. } => 0x02,
            Request::Write { .. } => 0x03,
        }
    }

    fn len_byte(&self) -> u8 {
        match *self {
            Request::Ping { .. } => 2,
            Request::Read { .. } => 4,
            Request::Write { ref data, .. } => (data.len() + 3) as u8,
        }
    }

    pub fn serialized(&self) -> Vec<u8> {
        let mut v = vec![0xff, 0xff];
        v.push(self.id_byte());
        v.push(self.len_byte());
        v.push(self.instruction_byte());
        match *self {
            Request::Write { addr, ref data, .. } => {
                v.push(addr);
                v.extend(data);
            }
            Request::Read { addr, len, .. } => {
                v.push(addr);
                v.push(len);
            }
            _ => {}
        }
        let crc = crc(&v);
        v.push(crc);
        v
    }
}

fn crc(serialized: &[u8]) -> u8 {
    crc_data(&serialized[2..])
}

fn crc_data(data: &[u8]) -> u8 {
    let mut sum: u8 = 0;
    for b in data {
        sum = sum.wrapping_add(*b);
    }
    return !sum;
}

#[cfg(test)]
mod tests {
    use super::{crc_data, Request, Status, Error};

    #[test]
    fn crc_is_calculated_correctly_from_primitive_data() {
        let actual = crc_data(&[0x00]);
        assert_eq!(actual, 0xff)
    }

    #[test]
    fn crc_is_calculated_correctly_from_longer_data_array() {
        let actual = crc_data(&[0x01, 0x01, 0x01]);
        assert_eq!(actual, 0xfc)
    }

    #[test]
    fn ping_is_serialized_correctly_up_to_crc() {
        //from doc
        let p = Request::Ping { id: 0x01 };
        let expected: Vec<u8> = vec![0xff, 0xff, 0x01, 0x02, 0x01];
        assert_eq!(expected[0..5], p.serialized()[0..5])
    }

    #[test]
    fn ping_is_serialized_correctly() {
        //from doc
        let p = Request::Ping { id: 0x03 };
        let expected: Vec<u8> = vec![0xff, 0xff, 0x03, 0x02, 0x01, 0xf9];
        assert_eq!(expected[0..5], p.serialized()[0..5])
    }

    #[test]
    fn write_is_serialized_correctly() {
        //from doc
        let expected: Vec<u8> = vec![0xff, 0xff, 0xfe, 0x04, 0x03, 0x03, 0x01, 0xf6];
        assert_eq!(expected, Request::Write { id: 0x0fe, addr: 0x03, data: vec![0x01] }.serialized())
    }

    #[test]
    fn read_is_serialized_correctly() {
        //from doc
        let expected: Vec<u8> = vec![0xff, 0xff, 0x01, 0x04, 0x02, 0x2b, 0x01, 0xcc];
        assert_eq!(expected, Request::Read { id: 0x01, addr: 0x2b, len: 0x01 }.serialized())
    }

    #[test]
    fn status_is_deserialized_correctly() {
        //from doc
        let input: Vec<u8> = vec![0xff, 0xff, 0x01, 0x02, 0x24, 0xd8];
        let status = Status::from_bytes(&input).unwrap();
        assert_eq!(status, Status { id: 0x01, error: 0x24, data: vec![] })
    }

    #[test]
    fn status_with_data_is_deserialized_correctly() {
        let input: Vec<u8> = vec![0xff, 0xff, 0x01, 0x06, 0x24, 0x00, 0x00, 0x00, 0x00, 0xd4];
        let status = Status::from_bytes(&input).unwrap();
        assert_eq!(status, Status { id: 0x01, error: 0x24, data: vec![0x00; 4] })
    }

    #[test]
    fn status_from_bytes_returns_too_short_error_when_too_few_bytes_provided() {
        let input: Vec<u8> = vec![0xff, 0xff, 0x01];
        assert_eq!(Status::from_bytes(&input).err(), Some(Error::PacketTooShort))
    }

    #[test]
    fn status_from_bytes_returns_too_short_error_when_invalid_length_is_provided() {
        let input: Vec<u8> = vec![0xff, 0xff, 0x01, 0x06, 0x24, 0xd8];
        assert_eq!(Status::from_bytes(&input).err(), Some(Error::PacketTooShort))
    }

    #[test]
    fn status_from_bytes_returns_invalid_crc_for_corrupt_packet() {
        let input: Vec<u8> = vec![0xff, 0xff, 0x01, 0x02, 0x24, 0xff];
        assert_eq!(Status::from_bytes(&input).err(), Some(Error::InvalidCrc))
    }
}