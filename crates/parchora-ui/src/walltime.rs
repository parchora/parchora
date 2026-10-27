//! 墙钟时间桥（DEC-183 · 拾记行时间显示承载）：本地日历字段获取与行内时间
//! 格式化纯函数。零新增依赖（规范 9）——`localtime_r` 经 `extern "C"` 直连
//! libSystem（std 不提供时区 API · DEC-63 免引 chrono：本模块为呈现层最小解，
//! 一次 FFI 取本地字段，格式化为纯函数可测）。
//!
//! 分层说明：`localtime_r` 非 objc2 系（platform 独占边界不受涉）；字段在
//! 视图渲染帧头取一次，行条目仅做纯比较（无逐行 FFI 热路径 · 铁律 12）。

use std::time::{SystemTime, UNIX_EPOCH};

/// macOSDarwin `struct tm`（LP64 布局 · 只读消费 gmtoff 前九字段）。
#[repr(C)]
struct Tm {
    tm_sec: i32,
    tm_min: i32,
    tm_hour: i32,
    tm_mday: i32,
    tm_mon: i32,
    tm_year: i32,
    tm_wday: i32,
    tm_yday: i32,
    tm_isdst: i32,
    tm_gmtoff: i64,
    tm_zone: *const u8,
}

extern "C" {
    /// libSystem 线程安全本地时间换算（time_t = long = i64 · darwin）。
    fn localtime_r(timep: *const i64, result: *mut Tm) -> *mut Tm;
}

/// 行时间戳（捕获时刻快照）：epoch 秒 ＋ 一次 `localtime_r` 派生的本地日历
/// 字段（y=公历年 · mo/d/h/mi/s=月日时分秒 · 1 起月）。epoch=0 为「无时间戳」
/// 哨兵（Default 构造 · 不显示行时）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamp {
    pub epoch: i64,
    pub y: i32,
    pub mo: u32,
    pub d: u32,
    pub h: u32,
    pub mi: u32,
    pub s: u32,
}

impl Stamp {
    /// 无时间戳哨兵（ClipRow::default 承载 · is_zero 门控不显示）。
    pub const fn zero() -> Self {
        Self {
            epoch: 0,
            y: 0,
            mo: 0,
            d: 0,
            h: 0,
            mi: 0,
            s: 0,
        }
    }

    /// 哨兵判定（epoch=0 ⇒ 不显示行时）。
    pub const fn is_zero(&self) -> bool {
        self.epoch == 0
    }
}

/// 当前墙钟 epoch 秒（捕获点/渲染帧取时唯一入口）。
pub fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// epoch 秒 → 本地日历字段快照（一次 FFI；失败兜底哨兵——不 panic 不虚构）。
pub fn local_civil(epoch: i64) -> Stamp {
    let mut tm = Tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: std::ptr::null(),
    };
    let ok = unsafe { localtime_r(&epoch, &mut tm) };
    if ok.is_null() {
        return Stamp::zero();
    }
    Stamp {
        epoch,
        y: tm.tm_year + 1900,
        mo: (tm.tm_mon + 1) as u32,
        d: tm.tm_mday as u32,
        h: tm.tm_hour as u32,
        mi: tm.tm_min as u32,
        s: tm.tm_sec as u32,
    }
}

/// 行时格式化（DEC-183 · 日历日判口径）：条目日历日＝今日 →「时:分:秒」
///；同年异日 →「月-日 时:分:秒」；跨年 →「年-月-日 时:分:秒」（零填充两位）。
/// 纯函数——条目字段取自捕获时快照，今日字段取自渲染帧头（无逐行 FFI）。
pub fn fmt_between(entry: &Stamp, now: &Stamp) -> String {
    let hms = format!("{:02}:{:02}:{:02}", entry.h, entry.mi, entry.s);
    if (entry.y, entry.mo, entry.d) == (now.y, now.mo, now.d) {
        hms
    } else if entry.y == now.y {
        format!("{:02}-{:02} {hms}", entry.mo, entry.d)
    } else {
        format!("{}-{:02}-{:02} {hms}", entry.y, entry.mo, entry.d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_same_day_shows_time_only() {
        let now = Stamp {
            epoch: 100,
            y: 2026,
            mo: 10,
            d: 22,
            h: 9,
            mi: 0,
            s: 0,
        };
        let entry = Stamp {
            epoch: 50,
            y: 2026,
            mo: 10,
            d: 22,
            h: 14,
            mi: 2,
            s: 3,
        };
        assert_eq!(fmt_between(&entry, &now), "14:02:03");
    }

    #[test]
    fn fmt_same_year_shows_month_day_time() {
        let now = Stamp {
            epoch: 100,
            y: 2026,
            mo: 10,
            d: 22,
            h: 9,
            mi: 0,
            s: 0,
        };
        let entry = Stamp {
            epoch: 50,
            y: 2026,
            mo: 10,
            d: 21,
            h: 23,
            mi: 59,
            s: 7,
        };
        assert_eq!(fmt_between(&entry, &now), "10-21 23:59:07");
    }

    #[test]
    fn fmt_cross_year_shows_full_date() {
        let now = Stamp {
            epoch: 100,
            y: 2026,
            mo: 1,
            d: 1,
            h: 0,
            mi: 0,
            s: 0,
        };
        let entry = Stamp {
            epoch: 50,
            y: 2025,
            mo: 12,
            d: 31,
            h: 8,
            mi: 5,
            s: 9,
        };
        assert_eq!(fmt_between(&entry, &now), "2025-12-31 08:05:09");
    }

    #[test]
    fn fmt_zero_pads_two_digits() {
        let now = Stamp {
            epoch: 100,
            y: 2026,
            mo: 3,
            d: 4,
            h: 0,
            mi: 0,
            s: 0,
        };
        let entry = Stamp {
            epoch: 50,
            y: 2026,
            mo: 3,
            d: 4,
            h: 3,
            mi: 4,
            s: 7,
        };
        assert_eq!(fmt_between(&entry, &now), "03:04:07");
    }

    #[test]
    fn zero_stamp_is_sentinel() {
        assert!(Stamp::zero().is_zero());
    }

    #[test]
    fn local_civil_now_in_sane_ranges() {
        let st = local_civil(now_epoch());
        assert!(!st.is_zero());
        assert!((2020..=2100).contains(&st.y), "年 {} 应落在合理区间", st.y);
        assert!((1..=12).contains(&st.mo), "月 {} 应为 1~12", st.mo);
        assert!((1..=31).contains(&st.d), "日 {} 应为 1~31", st.d);
        assert!(st.h < 24 && st.mi < 60 && st.s < 60);
    }
}
