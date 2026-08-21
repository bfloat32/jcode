use crate::error::{EvalError, EvalErrorKind};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct UtcTimestamp(u16, u8, u8, u8, u8, u8);

impl UtcTimestamp {
    pub(crate) fn parse(value: &str) -> Result<Self, EvalError> {
        let bytes = value.as_bytes();
        if bytes.len() != 20
            || bytes[4] != b'-'
            || bytes[7] != b'-'
            || bytes[10] != b'T'
            || bytes[13] != b':'
            || bytes[16] != b':'
            || bytes[19] != b'Z'
        {
            return Err(invalid_timestamp(value));
        }
        let year = digits_u16(&bytes[0..4]).ok_or_else(|| invalid_timestamp(value))?;
        let month = digits_u8(&bytes[5..7]).ok_or_else(|| invalid_timestamp(value))?;
        let day = digits_u8(&bytes[8..10]).ok_or_else(|| invalid_timestamp(value))?;
        let hour = digits_u8(&bytes[11..13]).ok_or_else(|| invalid_timestamp(value))?;
        let minute = digits_u8(&bytes[14..16]).ok_or_else(|| invalid_timestamp(value))?;
        let second = digits_u8(&bytes[17..19]).ok_or_else(|| invalid_timestamp(value))?;
        let days = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap_year(year) => 29,
            2 => 28,
            _ => 0,
        };
        if day == 0 || day > days || hour > 23 || minute > 59 || second > 59 {
            return Err(invalid_timestamp(value));
        }
        Ok(Self(year, month, day, hour, minute, second))
    }

    pub(crate) fn ordered(started: &str, finished: &str) -> bool {
        match (Self::parse(started), Self::parse(finished)) {
            (Ok(started), Ok(finished)) => started < finished,
            (Err(_), Ok(_)) | (Ok(_), Err(_)) | (Err(_), Err(_)) => false,
        }
    }
}

fn digits_u8(bytes: &[u8]) -> Option<u8> {
    std::str::from_utf8(bytes).ok()?.parse().ok()
}

fn digits_u16(bytes: &[u8]) -> Option<u16> {
    std::str::from_utf8(bytes).ok()?.parse().ok()
}

const fn leap_year(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

fn invalid_timestamp(value: &str) -> EvalError {
    EvalError::new(
        EvalErrorKind::InvalidResult,
        format!("invalid UTC timestamp {value}"),
    )
}
