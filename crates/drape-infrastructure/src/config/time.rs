use drape_core::TimestampMillis;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Instant {
    pub(super) millis: TimestampMillis,
    pub(super) sub_millis: String,
}

pub(super) fn parse(text: &str) -> Option<Instant> {
    if !text.is_ascii() {
        return None;
    }
    let bytes = text.as_bytes();
    if bytes.len() < 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !matches!(bytes[10], b'T' | b't')
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return None;
    }
    let year = number(bytes.get(..4)?)?;
    let month = number(bytes.get(5..7)?)?;
    let day = number(bytes.get(8..10)?)?;
    let hour = number(bytes.get(11..13)?)?;
    let minute = number(bytes.get(14..16)?)?;
    let second = number(bytes.get(17..19)?)?;
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    if !(1..=9999).contains(&year)
        || !(1..=month_days).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }
    let mut index = 19;
    let mut fraction = "";
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == start {
            return None;
        }
        fraction = text.get(start..index)?;
    }
    let zone = bytes.get(index..)?;
    let offset = match zone.len() {
        1 if matches!(zone[0], b'Z' | b'z') => 0,
        6 if matches!(zone[0], b'+' | b'-') && zone[3] == b':' => {
            let hours = number(&zone[1..3])?;
            let minutes = number(&zone[4..6])?;
            if hours > 23 || minutes > 59 {
                return None;
            }
            let offset = hours
                .checked_mul(3600)?
                .checked_add(minutes.checked_mul(60)?)?;
            if zone[0] == b'-' {
                offset.checked_neg()?
            } else {
                offset
            }
        }
        _ => return None,
    };
    let mut millis = 0_i64;
    for byte in fraction.as_bytes().iter().take(3) {
        millis = millis
            .checked_mul(10)?
            .checked_add(i64::from(*byte - b'0'))?;
    }
    for _ in fraction.len().min(3)..3 {
        millis = millis.checked_mul(10)?;
    }
    let seconds = days_from_civil(year, month, day)?
        .checked_mul(86400)?
        .checked_add(hour.checked_mul(3600)?)?
        .checked_add(minute.checked_mul(60)?)?
        .checked_add(second)?
        .checked_sub(offset)?;
    Some(Instant {
        millis: TimestampMillis(seconds.checked_mul(1000)?.checked_add(millis)?),
        sub_millis: fraction
            .get(3..)
            .unwrap_or_default()
            .trim_end_matches('0')
            .to_owned(),
    })
}

pub(super) fn format(time: TimestampMillis) -> Option<String> {
    let days = time.0.div_euclid(86_400_000);
    if !(days_from_civil(1, 1, 1)?..=days_from_civil(9999, 12, 31)?).contains(&days) {
        return None;
    }
    let (year, month, day) = civil_from_days(days);
    let remainder = time.0.rem_euclid(86_400_000);
    let hour = remainder / 3_600_000;
    let minute = remainder / 60_000 % 60;
    let second = remainder / 1000 % 60;
    let millis = remainder % 1000;
    let date = format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}");
    Some(if millis == 0 {
        format!("{date}Z")
    } else {
        format!("{date}.{millis:03}Z")
    })
}

fn number(bytes: &[u8]) -> Option<i64> {
    bytes.iter().try_fold(0_i64, |value, byte| {
        if !byte.is_ascii_digit() {
            return None;
        }
        value.checked_mul(10)?.checked_add(i64::from(*byte - b'0'))
    })
}

// Calendar arithmetic uses checked operations within the validated year range.
fn days_from_civil(year: i64, month: i64, day: i64) -> Option<i64> {
    let year = year.checked_sub(if month <= 2 { 1 } else { 0 })?;
    let era = year.div_euclid(400);
    let year_of_era = year.checked_sub(era.checked_mul(400)?)?;
    let month = month.checked_add(if month > 2 { -3 } else { 9 })?;
    let day_of_year = 153_i64
        .checked_mul(month)?
        .checked_add(2)?
        .checked_div(5)?
        .checked_add(day)?
        .checked_sub(1)?;
    let day_of_era = year_of_era
        .checked_mul(365)?
        .checked_add(year_of_era.checked_div(4)?)?
        .checked_sub(year_of_era.checked_div(100)?)?
        .checked_add(day_of_year)?;
    era.checked_mul(146097)?
        .checked_add(day_of_era)?
        .checked_sub(719468)
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719468;
    let era = days.div_euclid(146097);
    let day_of_era = days - era * 146097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month + 2) / 5 + 1;
    let month = month + if month < 10 { 3 } else { -9 };
    (year + if month <= 2 { 1 } else { 0 }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets_and_fractional_precision_are_projected_without_rounding() {
        let instant = parse("2026-10-09T14:43:00.123456700+07:00").unwrap();
        assert_eq!(instant.millis, TimestampMillis(1_791_531_780_123));
        assert_eq!(instant.sub_millis, "4567");
        assert_eq!(
            parse("1970-01-01t00:00:00.1z").unwrap().millis,
            TimestampMillis(100)
        );
        assert_eq!(
            parse("1970-01-01T00:00:00-01:30").unwrap().millis,
            TimestampMillis(5_400_000)
        );
        assert_eq!(parse("1970-01-01T00:00:00.123000Z").unwrap().sub_millis, "");
        assert_eq!(
            parse("1970-01-01T00:00:00.000000001Z").unwrap().sub_millis,
            "000001"
        );
        let long = format!("1970-01-01T00:00:00.{}Z", "1".repeat(1024));
        assert_eq!(parse(&long).unwrap().sub_millis.len(), 1021);
    }

    #[test]
    fn negative_epoch_and_calendar_boundaries_round_trip() {
        for (text, millis) in [
            ("0001-01-01T00:00:00Z", -62_135_596_800_000),
            ("1969-12-31T23:59:59.999Z", -1),
            ("1970-01-01T00:00:00Z", 0),
            ("2000-02-29T00:00:00Z", 951_782_400_000),
            ("9999-12-31T23:59:59.999Z", 253_402_300_799_999),
        ] {
            assert_eq!(parse(text).unwrap().millis, TimestampMillis(millis));
            assert_eq!(format(TimestampMillis(millis)).as_deref(), Some(text));
        }
        for millis in [i64::MIN, -62_135_596_800_001, 253_402_300_800_000, i64::MAX] {
            assert_eq!(format(TimestampMillis(millis)), None);
        }
        for year in [1, 4, 100, 400, 1600, 1900, 2000, 2100, 9999] {
            for month in 1..=12 {
                let text = format!("{year:04}-{month:02}-01T00:00:00Z");
                assert_eq!(format(parse(&text).unwrap().millis), Some(text));
            }
        }
    }

    #[test]
    fn rejects_invalid_calendar_syntax_and_non_ascii() {
        for text in [
            "",
            "2026-02-30T00:00:00Z",
            "1900-02-29T00:00:00Z",
            "0000-01-01T00:00:00Z",
            "10000-01-01T00:00:00Z",
            "2026-00-01T00:00:00Z",
            "2026-13-01T00:00:00Z",
            "2026-01-00T00:00:00Z",
            "2026-01-01T24:00:00Z",
            "2026-01-01T00:60:00Z",
            "2026-01-01T00:00:60Z",
            "2026-01-01T00:00:00.Z",
            "2026-01-01T00:00:00+24:00",
            "2026-01-01T00:00:00+00:60",
            "2026-01-01T00:00:00+0000",
            "2026-01-01 00:00:00Z",
            "2026-01-01T00:00:00",
            "2026-01-01T00:00:00Z ",
            "２０２６-01-01T00:00:00Z",
            "2026-01-01T00:00:00.éZ",
        ] {
            assert_eq!(parse(text), None, "{text}");
        }
    }
}
