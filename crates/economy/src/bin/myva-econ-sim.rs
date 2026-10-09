//! Chạy mô phỏng kinh tế headless, in CSV theo giờ ra stdout và tóm tắt ra stderr.
//!
//! ```text
//! cargo run -p myva-economy --bin myva-econ-sim -- --seed 7 --days 90 > econ.csv
//! ```
//!
//! Thoát với mã lỗi khi có vi phạm invariant.

use std::process::ExitCode;

use myva_economy::sim::{self, HourRow, SURGE_HOUR_OF_DAY, SimConfig};

const USAGE: &str = "dùng: myva-econ-sim [--seed N] [--days N] [--surge-day N|none]";

fn parse_args() -> Result<SimConfig, String> {
    let mut config = SimConfig::fiber(1);
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = || {
            args.next()
                .ok_or_else(|| format!("thiếu giá trị cho {flag}"))
        };
        match flag.as_str() {
            "--seed" => config.seed = value()?.parse().map_err(|e| format!("--seed: {e}"))?,
            "--days" => {
                let days: u32 = value()?.parse().map_err(|e| format!("--days: {e}"))?;
                config.hours = days * 24;
            }
            "--surge-day" => {
                config.surge_hour = match value()?.as_str() {
                    "none" => None,
                    day => Some(
                        day.parse::<u32>()
                            .map_err(|e| format!("--surge-day: {e}"))?
                            * 24
                            + SURGE_HOUR_OF_DAY,
                    ),
                }
            }
            "-h" | "--help" => return Err(USAGE.to_owned()),
            other => return Err(format!("tham số lạ: {other}\n{USAGE}")),
        }
    }
    Ok(config)
}

fn main() -> ExitCode {
    let config = match parse_args() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };
    let report = sim::run(&config);

    println!("{}", HourRow::CSV_HEADER);
    for row in &report.rows {
        println!("{}", row.to_csv());
    }

    // Ghi phiên bản, seed và cấu hình để kết quả có thể tái lập.
    eprintln!(
        "myva-econ-sim {} · seed {} · {} giờ",
        env!("CARGO_PKG_VERSION"),
        config.seed,
        config.hours
    );
    eprintln!("{config:?}");
    if let Some(last) = report.rows.last() {
        let [r, n, i, x, q, z] = last.balances;
        eprintln!("cuối kỳ: R={r} N={n} I={i} X={x} Q={q} Z={z}");
    }
    eprintln!("gửi lại cùng mã được nhận diện: {}", report.replays);
    if report.violations.is_empty() {
        eprintln!("không có vi phạm invariant");
        ExitCode::SUCCESS
    } else {
        for violation in &report.violations {
            eprintln!("VI PHẠM: {violation}");
        }
        ExitCode::FAILURE
    }
}
