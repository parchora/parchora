//! ISO8601 UTC 秒级时间桥（DB `created_at`/`modified_at` 承载 · 零依赖（规范 9））。
//!
//! 口径：写入恒为 `YYYY-MM-DDTHH:MM:SSZ`（UTC，无小数秒）；读取容错小数秒与
//! 显式偏移（外部数据兜底，不虚构时区）。算法采用 Howard Hinnant `civil_from_days`
//! / `days_from_civil`（公有领域），与既有 DEC-63「ISO 字典序=时间序」决策同轴。

/// epoch 秒 → ISO8601 UTC（`2026-10-17T08:00:00Z` 形态）。
pub fn iso_utc(epoch_secs: i64) -> String {
    let days = epoch_secs.div_euclid(86_400);
    let secs_of_day = epoch_secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        secs_of_day / 3600,
        (secs_of_day % 3600) / 60,
        secs_of_day % 60
    )
}

/// ISO8601 → epoch 秒（容错：小数秒截断；偏移 `Z` / `±HH:MM` / 无尾缀按 UTC 承载）。
/// 形态非法（缺段/非数字/越界字段）→ None（禁止虚构时间）。
pub fn epoch_from_iso(s: &str) -> Option<i64> {
    let s = s.trim();
    let (date, time_raw) = s.split_once(['T', ' '])?;
    let mut date_it = date.split('-');
    let y: i64 = date_it.next()?.trim().parse().ok()?;
    let m: u32 = date_it.next()?.trim().parse().ok()?;
    let d: u32 = date_it.next()?.trim().parse().ok()?;
    if date_it.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }

    // 偏移剥离（尾缀）：Z / ±HH:MM（含 +00:00）；小数秒先截断。
    let time = time_raw.trim_end();
    let time = match time.find('.').or_else(|| time.find(',')) {
        Some(idx) => &time[..idx],
        None => time,
    };
    let (time, offset_secs) = if let Some(stripped) = time.strip_suffix('Z') {
        (stripped, 0i64)
    } else if time.len() >= 6 {
        let marker = time.as_bytes()[time.len() - 6];
        if marker == b'+' || marker == b'-' {
            let (body, off) = time.split_at(time.len() - 6);
            let off = &off[1..]; // 去符号位
            let mut off_it = off.split(':');
            let oh: i64 = off_it.next()?.trim().parse().ok()?;
            let om: i64 = off_it.next()?.trim().parse().ok()?;
            if off_it.next().is_some() || !(0..=23).contains(&oh) || !(0..=59).contains(&om) {
                return None;
            }
            let mag = oh * 3600 + om * 60;
            (body, if marker == b'+' { mag } else { -mag })
        } else {
            (time, 0i64)
        }
    } else {
        (time, 0i64)
    };

    let mut time_it = time.split(':');
    let hh: i64 = time_it.next()?.trim().parse().ok()?;
    let mi: i64 = time_it.next()?.trim().parse().ok()?;
    let ss: i64 = time_it.next().unwrap_or("0").trim().parse().ok()?;
    if time_it.next().is_some()
        || !(0..=23).contains(&hh)
        || !(0..=59).contains(&mi)
        || !(0..=59).contains(&ss)
    {
        return None;
    }

    let days = days_from_civil(y, m, d);
    Some(days * 86_400 + hh * 3600 + mi * 60 + ss - offset_secs)
}

/// Hinnant `civil_from_days`：days since 1970-01-01 → (年, 月, 日)。
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Hinnant `days_from_civil`：(年, 月, 日) → days since 1970-01-01。
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64; // [0, 399]
    let mp = if m > 2 { m - 3 } else { m + 9 } as u64;
    let doy = (153 * mp + 2) / 5 + d as u64 - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe as i64 - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_zero_and_first_day_anchors() {
        assert_eq!(iso_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_utc(86_400), "1970-01-02T00:00:00Z");
        assert_eq!(epoch_from_iso("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(epoch_from_iso("1970-01-02T00:00:00Z"), Some(86_400));
    }

    #[test]
    fn known_wide_epoch_anchor() {
        // 公认锚点：1700000000 = 2023-11-14T22:13:20Z
        assert_eq!(iso_utc(1_700_000_000), "2023-11-14T22:13:20Z");
        assert_eq!(epoch_from_iso("2023-11-14T22:13:20Z"), Some(1_700_000_000));
    }

    #[test]
    fn pre_epoch_negative_anchor() {
        // 公认锚点：1969-12-31T00:00:00Z = -86400
        assert_eq!(iso_utc(-86_400), "1969-12-31T00:00:00Z");
        assert_eq!(epoch_from_iso("1969-12-31T00:00:00Z"), Some(-86_400));
    }

    /// 写读互逆全样本（含闰年边界 2-29 与年代际跨度）。
    #[test]
    fn roundtrip_across_samples() {
        let samples = [
            0i64,
            86_399,
            86_400,
            -1,
            -86_400,
            951_782_400, // 2000-02-29T00:00:00Z（世纪闰日）
            1_709_164_800,
            1_745_000_000,
            4_102_344_000, // 2100-01-01T00:00:00Z
        ];
        for epoch in samples {
            let iso = iso_utc(epoch);
            assert_eq!(epoch_from_iso(&iso), Some(epoch), "iso={iso}");
        }
    }

    /// 闰日往返显式断言（civil 算法闰表位）。
    #[test]
    fn leap_day_roundtrip() {
        assert_eq!(iso_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(epoch_from_iso("2000-02-29T00:00:00Z"), Some(951_782_400));
    }

    #[test]
    fn malformed_inputs_rejected() {
        assert_eq!(epoch_from_iso("2026-10-17"), None); // 无时间段
        assert_eq!(epoch_from_iso("2026-13-01T00:00:00Z"), None); // 月越界
        assert_eq!(epoch_from_iso("2026-10-17T24:00:00Z"), None); // 时越界
        assert_eq!(epoch_from_iso("bogus"), None);
        assert_eq!(epoch_from_iso(""), None);
    }

    #[test]
    fn fractional_and_offset_tolerated() {
        // 小数秒截断 + 显式偏移回算（外部数据兜底口径）。
        assert_eq!(epoch_from_iso("1970-01-01T00:00:00.500Z"), Some(0));
        assert_eq!(epoch_from_iso("1970-01-01T08:00:00+08:00"), Some(0));
        assert_eq!(epoch_from_iso("1970-01-01T00:00:00-01:00"), Some(3_600));
    }
}
