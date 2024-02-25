// A single bookkeeping entry
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Record {
    pub taken: bool,
    pub last: bool,
}

impl Record {
    pub(crate) fn from_byte(byte: u8) -> [Record; 4] {
        let mut records = [Default::default(); 4];
        for (i, record) in records.iter_mut().enumerate() {
            let byte = byte >> (i * 2);
            let taken = 0b10 & byte != 0;
            let last = 0b01 & byte != 0;
            *record = Record { taken, last };
        }
        records
    }

    pub(crate) fn to_byte(records: [Record; 4]) -> u8 {
        let mut byte = 0;
        for (i, record) in records.iter().enumerate() {
            if record.last {
                byte |= 1 << (2 * i);
            }
            if record.taken {
                byte |= 1 << (2 * i + 1);
            }
        }
        byte
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_from() {
        for byte in [0x00, 0xff, 0xba, 0xdd, 0xec, 0xaf] {
            let records = Record::from_byte(byte);
            assert_eq!(byte, Record::to_byte(records));
        }
    }
}
