//! # philiprehberger-cron-parser
//!
//! Cron expression parsing, scheduling, and human-readable descriptions.
//! Zero external dependencies — uses only the standard library.
//!
//! ## Quick Start
//!
//! ```
//! use philiprehberger_cron_parser::{CronExpr, DateTime};
//!
//! let expr = CronExpr::parse("*/15 * * * *").unwrap();
//! let now = DateTime { year: 2026, month: 3, day: 15, hour: 10, minute: 3, second: 0 };
//! let next = expr.next_from(&now).unwrap();
//! assert_eq!(next, DateTime { year: 2026, month: 3, day: 15, hour: 10, minute: 15, second: 0 });
//!
//! // Human-readable description
//! assert_eq!(expr.describe(), "Every 15 minutes");
//! ```
//!
//! ## Supported Syntax
//!
//! Standard 5-field cron: `minute hour day-of-month month day-of-week`
//!
//! Each field supports: single values (`5`), ranges (`1-5`), steps (`*/15`, `1-5/2`),
//! lists (`1,3,5`), and wildcards (`*`).
//!
//! Aliases: `@hourly`, `@daily`, `@midnight`, `@weekly`, `@monthly`, `@yearly`, `@annually`

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// DateTime
// ---------------------------------------------------------------------------

/// A simple UTC date-time with second precision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

/// Returns `true` if `year` is a leap year.
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Returns the number of days in the given month (1-12) for the given year.
pub fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 => 31,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        3 => 31,
        4 => 30,
        5 => 31,
        6 => 30,
        7 => 31,
        8 => 31,
        9 => 30,
        10 => 31,
        11 => 30,
        12 => 31,
        _ => 30,
    }
}

/// Returns the day of the week for a given date.
/// 0 = Sunday, 1 = Monday, ..., 6 = Saturday.
///
/// Uses Tomohiko Sakamoto's algorithm.
pub fn day_of_week(year: i32, month: u8, day: u8) -> u8 {
    let t = [0i32, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = year;
    if month < 3 {
        y -= 1;
    }
    let dow = (y + y / 4 - y / 100 + y / 400 + t[(month - 1) as usize] + day as i32) % 7;
    // Ensure non-negative result
    ((dow + 7) % 7) as u8
}

impl DateTime {
    /// Returns the current UTC date-time derived from `SystemTime::now()`.
    pub fn now() -> Self {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self::from_timestamp(secs as i64)
    }

    /// Constructs a `DateTime` from a Unix timestamp (seconds since 1970-01-01 00:00:00 UTC).
    pub fn from_timestamp(mut ts: i64) -> Self {
        let second = (ts % 60) as u8;
        ts /= 60;
        let minute = (ts % 60) as u8;
        ts /= 60;
        let hour = (ts % 24) as u8;
        ts /= 24;

        // ts is now days since epoch (1970-01-01 = day 0)
        let mut days = ts;

        // Compute year
        let mut year: i32 = 1970;
        loop {
            let days_in_year: i64 = if is_leap_year(year) { 366 } else { 365 };
            if days < days_in_year {
                break;
            }
            days -= days_in_year;
            year += 1;
        }

        // Compute month and day
        let mut month: u8 = 1;
        loop {
            let dim = days_in_month(year, month) as i64;
            if days < dim {
                break;
            }
            days -= dim;
            month += 1;
        }
        let day = days as u8 + 1;

        DateTime {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    /// Advance by one minute, returning a new `DateTime` with second set to 0.
    pub fn next_minute(&self) -> DateTime {
        let mut year = self.year;
        let mut month = self.month;
        let mut day = self.day;
        let mut hour = self.hour;
        let mut minute = self.minute + 1;

        if minute >= 60 {
            minute = 0;
            hour += 1;
        }
        if hour >= 24 {
            hour = 0;
            day += 1;
        }
        if day > days_in_month(year, month) {
            day = 1;
            month += 1;
        }
        if month > 12 {
            month = 1;
            year += 1;
        }

        DateTime {
            year,
            month,
            day,
            hour,
            minute,
            second: 0,
        }
    }

    /// Returns the day of the week (0 = Sunday .. 6 = Saturday).
    pub fn day_of_week(&self) -> u8 {
        day_of_week(self.year, self.month, self.day)
    }
}

impl Ord for DateTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.year
            .cmp(&other.year)
            .then(self.month.cmp(&other.month))
            .then(self.day.cmp(&other.day))
            .then(self.hour.cmp(&other.hour))
            .then(self.minute.cmp(&other.minute))
            .then(self.second.cmp(&other.second))
    }
}

impl PartialOrd for DateTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

// ---------------------------------------------------------------------------
// ParseError
// ---------------------------------------------------------------------------

/// Errors that can occur when parsing a cron expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The expression did not have exactly 5 space-separated fields.
    InvalidFieldCount,
    /// A field could not be parsed.
    InvalidField {
        field: String,
        value: String,
    },
    /// An unknown alias was used (e.g. `@bogus`).
    InvalidAlias(String),
    /// A numeric value was outside the allowed range for its field.
    ValueOutOfRange {
        field: String,
        value: u8,
        min: u8,
        max: u8,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidFieldCount => {
                write!(f, "cron expression must have exactly 5 fields")
            }
            ParseError::InvalidField { field, value } => {
                write!(f, "invalid value '{}' for field '{}'", value, field)
            }
            ParseError::InvalidAlias(alias) => {
                write!(f, "unknown alias '{}'", alias)
            }
            ParseError::ValueOutOfRange {
                field,
                value,
                min,
                max,
            } => {
                write!(
                    f,
                    "value {} out of range for field '{}' ({}..{})",
                    value, field, min, max
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

// ---------------------------------------------------------------------------
// CronField (internal)
// ---------------------------------------------------------------------------

/// The set of allowed values for a single cron field, stored as a sorted `Vec<u8>`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CronField {
    values: Vec<u8>,
}

impl CronField {
    fn contains(&self, v: u8) -> bool {
        self.values.contains(&v)
    }
}

/// Field metadata: name, min, max.
struct FieldSpec {
    name: &'static str,
    min: u8,
    max: u8,
}

const FIELD_SPECS: [FieldSpec; 5] = [
    FieldSpec { name: "minute", min: 0, max: 59 },
    FieldSpec { name: "hour", min: 0, max: 23 },
    FieldSpec { name: "day-of-month", min: 1, max: 31 },
    FieldSpec { name: "month", min: 1, max: 12 },
    FieldSpec { name: "day-of-week", min: 0, max: 7 },
];

/// Map month/day-of-week names to numbers. Returns `None` if not a name.
fn name_to_number(s: &str, field_index: usize) -> Option<u8> {
    let upper = s.to_ascii_uppercase();
    if field_index == 3 {
        // month
        match upper.as_str() {
            "JAN" => Some(1),
            "FEB" => Some(2),
            "MAR" => Some(3),
            "APR" => Some(4),
            "MAY" => Some(5),
            "JUN" => Some(6),
            "JUL" => Some(7),
            "AUG" => Some(8),
            "SEP" => Some(9),
            "OCT" => Some(10),
            "NOV" => Some(11),
            "DEC" => Some(12),
            _ => None,
        }
    } else if field_index == 4 {
        // day-of-week
        match upper.as_str() {
            "SUN" => Some(0),
            "MON" => Some(1),
            "TUE" => Some(2),
            "WED" => Some(3),
            "THU" => Some(4),
            "FRI" => Some(5),
            "SAT" => Some(6),
            _ => None,
        }
    } else {
        None
    }
}

fn parse_single_value(s: &str, field_index: usize, spec: &FieldSpec) -> Result<u8, ParseError> {
    if let Some(v) = name_to_number(s, field_index) {
        return Ok(v);
    }
    let v: u8 = s.parse().map_err(|_| ParseError::InvalidField {
        field: spec.name.to_string(),
        value: s.to_string(),
    })?;
    // Normalise day-of-week 7 -> 0
    let v = if field_index == 4 && v == 7 { 0 } else { v };
    if v < spec.min || v > spec.max {
        return Err(ParseError::ValueOutOfRange {
            field: spec.name.to_string(),
            value: v,
            min: spec.min,
            max: if field_index == 4 { 6 } else { spec.max },
        });
    }
    Ok(v)
}

fn parse_field(token: &str, field_index: usize) -> Result<CronField, ParseError> {
    let spec = &FIELD_SPECS[field_index];
    let mut all_values: Vec<u8> = Vec::new();

    for part in token.split(',') {
        // Check for step: e.g. */15 or 1-5/2
        let (range_part, step) = if let Some(pos) = part.find('/') {
            let step_str = &part[pos + 1..];
            let step: u8 = step_str.parse().map_err(|_| ParseError::InvalidField {
                field: spec.name.to_string(),
                value: part.to_string(),
            })?;
            if step == 0 {
                return Err(ParseError::InvalidField {
                    field: spec.name.to_string(),
                    value: part.to_string(),
                });
            }
            (&part[..pos], Some(step))
        } else {
            (part, None)
        };

        let (start, end) = if range_part == "*" {
            (spec.min, if field_index == 4 { 6 } else { spec.max })
        } else if let Some(dash) = range_part.find('-') {
            let s = parse_single_value(&range_part[..dash], field_index, spec)?;
            let e = parse_single_value(&range_part[dash + 1..], field_index, spec)?;
            (s, e)
        } else {
            let v = parse_single_value(range_part, field_index, spec)?;
            (v, v)
        };

        if let Some(step) = step {
            let mut v = start;
            loop {
                if v > end {
                    break;
                }
                all_values.push(v);
                v = v.saturating_add(step);
            }
        } else if start <= end {
            for v in start..=end {
                all_values.push(v);
            }
        } else {
            // Wrap-around range for day-of-week (e.g. 5-1 means FRI,SAT,SUN,MON)
            let upper = if field_index == 4 { 6 } else { spec.max };
            for v in start..=upper {
                all_values.push(v);
            }
            for v in spec.min..=end {
                all_values.push(v);
            }
        }
    }

    all_values.sort();
    all_values.dedup();

    Ok(CronField { values: all_values })
}

// ---------------------------------------------------------------------------
// CronExpr
// ---------------------------------------------------------------------------

/// A parsed cron expression.
///
/// Use [`CronExpr::parse`] to create one from a string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronExpr {
    minute: CronField,
    hour: CronField,
    day_of_month: CronField,
    month: CronField,
    day_of_week: CronField,
    /// The original expression string (for describe).
    raw: String,
}

impl CronExpr {
    /// Parse a 5-field cron expression or an alias.
    ///
    /// # Examples
    ///
    /// ```
    /// use philiprehberger_cron_parser::CronExpr;
    ///
    /// let expr = CronExpr::parse("0 9 * * 1-5").unwrap();
    /// let hourly = CronExpr::parse("@hourly").unwrap();
    /// ```
    pub fn parse(expr: &str) -> Result<CronExpr, ParseError> {
        let trimmed = expr.trim();

        // Handle aliases
        if trimmed.starts_with('@') {
            let alias = trimmed.to_ascii_lowercase();
            let expanded = match alias.as_str() {
                "@hourly" => "0 * * * *",
                "@daily" | "@midnight" => "0 0 * * *",
                "@weekly" => "0 0 * * 0",
                "@monthly" => "0 0 1 * *",
                "@yearly" | "@annually" => "0 0 1 1 *",
                _ => return Err(ParseError::InvalidAlias(trimmed.to_string())),
            };
            return Self::parse_fields(expanded, trimmed);
        }

        Self::parse_fields(trimmed, trimmed)
    }

    fn parse_fields(fields_str: &str, raw: &str) -> Result<CronExpr, ParseError> {
        let tokens: Vec<&str> = fields_str.split_whitespace().collect();
        if tokens.len() != 5 {
            return Err(ParseError::InvalidFieldCount);
        }

        let minute = parse_field(tokens[0], 0)?;
        let hour = parse_field(tokens[1], 1)?;
        let day_of_month = parse_field(tokens[2], 2)?;
        let month = parse_field(tokens[3], 3)?;
        let day_of_week = parse_field(tokens[4], 4)?;

        Ok(CronExpr {
            minute,
            hour,
            day_of_month,
            month,
            day_of_week,
            raw: raw.to_string(),
        })
    }

    /// Returns `true` if the given datetime matches this cron expression.
    ///
    /// Only minute, hour, day, month, and day-of-week are checked; seconds are ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// use philiprehberger_cron_parser::{CronExpr, DateTime};
    ///
    /// let expr = CronExpr::parse("0 9 * * *").unwrap();
    /// let dt = DateTime { year: 2026, month: 3, day: 15, hour: 9, minute: 0, second: 0 };
    /// assert!(expr.matches(&dt));
    /// ```
    pub fn matches(&self, dt: &DateTime) -> bool {
        self.minute.contains(dt.minute)
            && self.hour.contains(dt.hour)
            && self.day_of_month.contains(dt.day)
            && self.month.contains(dt.month)
            && self.day_of_week.contains(dt.day_of_week())
    }

    /// Find the next datetime that matches this cron expression, strictly after `dt`.
    ///
    /// Returns `None` if no match is found within ~4 years of searching.
    ///
    /// # Examples
    ///
    /// ```
    /// use philiprehberger_cron_parser::{CronExpr, DateTime};
    ///
    /// let expr = CronExpr::parse("0 9 * * *").unwrap();
    /// let dt = DateTime { year: 2026, month: 3, day: 15, hour: 10, minute: 0, second: 0 };
    /// let next = expr.next_from(&dt).unwrap();
    /// assert_eq!(next.day, 16);
    /// assert_eq!(next.hour, 9);
    /// assert_eq!(next.minute, 0);
    /// ```
    pub fn next_from(&self, dt: &DateTime) -> Option<DateTime> {
        // Start from the next minute after dt
        let mut candidate = DateTime {
            year: dt.year,
            month: dt.month,
            day: dt.day,
            hour: dt.hour,
            minute: dt.minute,
            second: 0,
        }
        .next_minute();

        // Safety limit: don't search more than ~4 years of minutes
        let max_iterations = 4 * 366 * 24 * 60;
        for _ in 0..max_iterations {
            // Fast-skip: if month doesn't match, jump to next month
            if !self.month.contains(candidate.month) {
                candidate = DateTime {
                    year: candidate.year,
                    month: candidate.month,
                    day: 1,
                    hour: 0,
                    minute: 0,
                    second: 0,
                };
                // Advance to next month
                candidate.month += 1;
                if candidate.month > 12 {
                    candidate.month = 1;
                    candidate.year += 1;
                }
                continue;
            }

            // Fast-skip: if day doesn't match both day-of-month and day-of-week
            if !self.day_of_month.contains(candidate.day)
                || !self.day_of_week.contains(candidate.day_of_week())
            {
                // Advance to next day
                candidate = DateTime {
                    year: candidate.year,
                    month: candidate.month,
                    day: candidate.day,
                    hour: 23,
                    minute: 59,
                    second: 0,
                }
                .next_minute();
                continue;
            }

            // Fast-skip: if hour doesn't match
            if !self.hour.contains(candidate.hour) {
                candidate = DateTime {
                    year: candidate.year,
                    month: candidate.month,
                    day: candidate.day,
                    hour: candidate.hour,
                    minute: 59,
                    second: 0,
                }
                .next_minute();
                continue;
            }

            if self.matches(&candidate) {
                return Some(candidate);
            }

            candidate = candidate.next_minute();
        }

        None
    }

    /// Returns the next `n` matching datetimes after `dt`.
    ///
    /// # Examples
    ///
    /// ```
    /// use philiprehberger_cron_parser::{CronExpr, DateTime};
    ///
    /// let expr = CronExpr::parse("0 * * * *").unwrap();
    /// let dt = DateTime { year: 2026, month: 1, day: 1, hour: 0, minute: 0, second: 0 };
    /// let times = expr.next_n_from(&dt, 3);
    /// assert_eq!(times.len(), 3);
    /// ```
    pub fn next_n_from(&self, dt: &DateTime, n: usize) -> Vec<DateTime> {
        let mut results = Vec::with_capacity(n);
        let mut current = *dt;
        for _ in 0..n {
            match self.next_from(&current) {
                Some(next) => {
                    results.push(next);
                    current = next;
                }
                None => break,
            }
        }
        results
    }

    /// Returns a human-readable description of the cron expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use philiprehberger_cron_parser::CronExpr;
    ///
    /// let expr = CronExpr::parse("*/15 * * * *").unwrap();
    /// assert_eq!(expr.describe(), "Every 15 minutes");
    /// ```
    pub fn describe(&self) -> String {
        // Check for common aliases first
        let norm = self.raw.trim().to_ascii_lowercase();
        match norm.as_str() {
            "@hourly" => return "Every hour".to_string(),
            "@daily" | "@midnight" => return "Every day at midnight".to_string(),
            "@weekly" => return "Every Sunday at midnight".to_string(),
            "@monthly" => return "At midnight on the 1st of every month".to_string(),
            "@yearly" | "@annually" => return "At midnight on January 1st".to_string(),
            _ => {}
        }

        let mut parts: Vec<String> = Vec::new();

        // Describe minute
        let min_desc = describe_field(&self.minute, 0);
        // Describe hour
        let hour_desc = describe_field(&self.hour, 1);

        // Special common patterns
        let all_minutes = self.minute.values.len() == 60;
        let all_hours = self.hour.values.len() == 24;
        let all_days = self.day_of_month.values.len() == 31;
        let all_months = self.month.values.len() == 12;
        let all_dow = self.day_of_week.values.len() == 7;

        // "Every N minutes" pattern
        if all_hours && all_days && all_months && all_dow {
            if let Some(step) = detect_step(&self.minute, 0, 59) {
                if self.minute.values[0] == 0 {
                    if step == 1 {
                        return "Every minute".to_string();
                    }
                    return format!("Every {} minutes", step);
                }
            }
            if all_minutes {
                return "Every minute".to_string();
            }
        }

        // "Every N hours" pattern
        if all_minutes.not() && self.minute.values.len() == 1 && self.minute.values[0] == 0
            && all_days && all_months && all_dow
        {
            if let Some(step) = detect_step(&self.hour, 0, 23) {
                if self.hour.values[0] == 0 {
                    return format!("Every {} hours", step);
                }
            }
            if all_hours {
                return "Every hour".to_string();
            }
        }

        // Time description
        if self.minute.values.len() == 1 && self.hour.values.len() == 1 {
            let h = self.hour.values[0];
            let m = self.minute.values[0];
            let ampm = if h == 0 {
                "12:00 AM".to_string()
            } else if h < 12 {
                format!("{}:{:02} AM", h, m)
            } else if h == 12 {
                format!("12:{:02} PM", m)
            } else {
                format!("{}:{:02} PM", h - 12, m)
            };
            parts.push(format!("At {}", ampm));
        } else if self.minute.values.len() == 1 {
            parts.push(format!("At minute {}", self.minute.values[0]));
            if !all_hours {
                parts.push(format!("past {}", hour_desc));
            }
        } else {
            parts.push(min_desc);
            if !all_hours {
                parts.push(format!("past {}", hour_desc));
            }
        }

        // Day-of-week
        if !all_dow {
            let dow_desc = describe_dow(&self.day_of_week);
            parts.push(dow_desc);
        }

        // Day-of-month
        if !all_days {
            let dom_desc = describe_field(&self.day_of_month, 2);
            parts.push(format!("on day {}", dom_desc));
        }

        // Month
        if !all_months {
            let month_desc = describe_month_field(&self.month);
            parts.push(format!("in {}", month_desc));
        }

        parts.join(", ")
    }
}

// Helper to detect a step pattern in a field's values.
fn detect_step(field: &CronField, min: u8, max: u8) -> Option<u8> {
    if field.values.len() < 2 {
        return None;
    }
    let step = field.values[1] - field.values[0];
    if step == 0 {
        return None;
    }
    // Verify all values follow the step
    for i in 1..field.values.len() {
        if field.values[i] - field.values[i - 1] != step {
            return None;
        }
    }
    // Verify it covers from min with step
    let expected_count = ((max - min) / step) + 1;
    if field.values.len() == expected_count as usize && field.values[0] == min {
        Some(step)
    } else {
        None
    }
}

fn describe_field(field: &CronField, field_index: usize) -> String {
    if field.values.len() == 1 {
        return field.values[0].to_string();
    }

    // Check if it's a contiguous range
    let is_contiguous = field
        .values
        .windows(2)
        .all(|w| w[1] == w[0] + 1);

    if is_contiguous && field.values.len() >= 2 {
        let start = field.values[0];
        let end = *field.values.last().unwrap();
        if field_index == 1 {
            return format!("hour {} through {}", start, end);
        }
        return format!("{} through {}", start, end);
    }

    // Check for step pattern
    if let Some(step) = detect_step(field, FIELD_SPECS[field_index].min, FIELD_SPECS[field_index].max) {
        return format!("every {} values", step);
    }

    // List
    let strs: Vec<String> = field.values.iter().map(|v| v.to_string()).collect();
    strs.join(", ")
}

fn describe_dow(field: &CronField) -> String {
    let names = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

    // Check if contiguous
    let is_contiguous = field
        .values
        .windows(2)
        .all(|w| w[1] == w[0] + 1);

    if is_contiguous && field.values.len() >= 2 {
        let start = names[field.values[0] as usize];
        let end = names[*field.values.last().unwrap() as usize];
        return format!("{} through {}", start, end);
    }

    let day_names: Vec<&str> = field
        .values
        .iter()
        .map(|v| names[*v as usize])
        .collect();

    if day_names.len() == 1 {
        format!("on {}", day_names[0])
    } else {
        format!("on {}", day_names.join(", "))
    }
}

fn describe_month_field(field: &CronField) -> String {
    let names = [
        "", "January", "February", "March", "April", "May", "June", "July", "August",
        "September", "October", "November", "December",
    ];

    let month_names: Vec<&str> = field
        .values
        .iter()
        .map(|v| names[*v as usize])
        .collect();

    month_names.join(", ")
}

trait Not {
    fn not(&self) -> bool;
}

impl Not for bool {
    fn not(&self) -> bool {
        !*self
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- DateTime tests --

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2023));
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2023, 1), 31);
        assert_eq!(days_in_month(2023, 4), 30);
    }

    #[test]
    fn test_day_of_week() {
        // 2026-03-15 is a Sunday
        assert_eq!(day_of_week(2026, 3, 15), 0);
        // 2026-03-16 is a Monday
        assert_eq!(day_of_week(2026, 3, 16), 1);
        // 1970-01-01 was a Thursday
        assert_eq!(day_of_week(1970, 1, 1), 4);
    }

    #[test]
    fn test_datetime_from_timestamp() {
        let dt = DateTime::from_timestamp(0);
        assert_eq!(dt, DateTime { year: 1970, month: 1, day: 1, hour: 0, minute: 0, second: 0 });

        // 2026-03-15 00:00:00 UTC = 1773532800
        let dt = DateTime::from_timestamp(1_773_532_800);
        assert_eq!(dt.year, 2026);
        assert_eq!(dt.month, 3);
        assert_eq!(dt.day, 15);
    }

    #[test]
    fn test_datetime_next_minute() {
        let dt = DateTime { year: 2026, month: 12, day: 31, hour: 23, minute: 59, second: 30 };
        let next = dt.next_minute();
        assert_eq!(next, DateTime { year: 2027, month: 1, day: 1, hour: 0, minute: 0, second: 0 });
    }

    #[test]
    fn test_datetime_next_minute_end_of_feb_leap() {
        let dt = DateTime { year: 2024, month: 2, day: 29, hour: 23, minute: 59, second: 0 };
        let next = dt.next_minute();
        assert_eq!(next, DateTime { year: 2024, month: 3, day: 1, hour: 0, minute: 0, second: 0 });
    }

    #[test]
    fn test_datetime_display() {
        let dt = DateTime { year: 2026, month: 3, day: 5, hour: 9, minute: 7, second: 3 };
        assert_eq!(format!("{}", dt), "2026-03-05T09:07:03");
    }

    #[test]
    fn test_datetime_ord() {
        let a = DateTime { year: 2026, month: 1, day: 1, hour: 0, minute: 0, second: 0 };
        let b = DateTime { year: 2026, month: 1, day: 1, hour: 0, minute: 1, second: 0 };
        assert!(a < b);
    }

    // -- CronExpr parsing tests --

    #[test]
    fn test_parse_all_wildcards() {
        let expr = CronExpr::parse("* * * * *").unwrap();
        assert_eq!(expr.minute.values.len(), 60);
        assert_eq!(expr.hour.values.len(), 24);
        assert_eq!(expr.day_of_month.values.len(), 31);
        assert_eq!(expr.month.values.len(), 12);
        assert_eq!(expr.day_of_week.values.len(), 7);
    }

    #[test]
    fn test_parse_single_values() {
        let expr = CronExpr::parse("5 9 15 3 1").unwrap();
        assert_eq!(expr.minute.values, vec![5]);
        assert_eq!(expr.hour.values, vec![9]);
        assert_eq!(expr.day_of_month.values, vec![15]);
        assert_eq!(expr.month.values, vec![3]);
        assert_eq!(expr.day_of_week.values, vec![1]);
    }

    #[test]
    fn test_parse_ranges() {
        let expr = CronExpr::parse("0-5 9-17 1-15 1-6 1-5").unwrap();
        assert_eq!(expr.minute.values, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(expr.hour.values, vec![9, 10, 11, 12, 13, 14, 15, 16, 17]);
        assert_eq!(expr.day_of_week.values, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_parse_steps() {
        let expr = CronExpr::parse("*/15 */6 * * *").unwrap();
        assert_eq!(expr.minute.values, vec![0, 15, 30, 45]);
        assert_eq!(expr.hour.values, vec![0, 6, 12, 18]);
    }

    #[test]
    fn test_parse_range_with_step() {
        let expr = CronExpr::parse("1-10/3 * * * *").unwrap();
        assert_eq!(expr.minute.values, vec![1, 4, 7, 10]);
    }

    #[test]
    fn test_parse_lists() {
        let expr = CronExpr::parse("0,15,30,45 * * * *").unwrap();
        assert_eq!(expr.minute.values, vec![0, 15, 30, 45]);
    }

    #[test]
    fn test_parse_dow_7_is_sunday() {
        let expr = CronExpr::parse("0 0 * * 7").unwrap();
        assert_eq!(expr.day_of_week.values, vec![0]); // 7 normalised to 0
    }

    #[test]
    fn test_parse_month_names() {
        let expr = CronExpr::parse("0 0 1 JAN-MAR *").unwrap();
        assert_eq!(expr.month.values, vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_dow_names() {
        let expr = CronExpr::parse("0 9 * * MON-FRI").unwrap();
        assert_eq!(expr.day_of_week.values, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_parse_mixed_case_names() {
        let expr = CronExpr::parse("0 0 1 jan *").unwrap();
        assert_eq!(expr.month.values, vec![1]);
        let expr = CronExpr::parse("0 0 * * Mon").unwrap();
        assert_eq!(expr.day_of_week.values, vec![1]);
    }

    // -- Alias tests --

    #[test]
    fn test_alias_hourly() {
        let expr = CronExpr::parse("@hourly").unwrap();
        assert_eq!(expr.minute.values, vec![0]);
        assert_eq!(expr.hour.values.len(), 24);
    }

    #[test]
    fn test_alias_daily() {
        let expr = CronExpr::parse("@daily").unwrap();
        assert_eq!(expr.minute.values, vec![0]);
        assert_eq!(expr.hour.values, vec![0]);
    }

    #[test]
    fn test_alias_midnight() {
        let a = CronExpr::parse("@midnight").unwrap();
        let b = CronExpr::parse("@daily").unwrap();
        assert_eq!(a.minute, b.minute);
        assert_eq!(a.hour, b.hour);
    }

    #[test]
    fn test_alias_weekly() {
        let expr = CronExpr::parse("@weekly").unwrap();
        assert_eq!(expr.day_of_week.values, vec![0]);
    }

    #[test]
    fn test_alias_monthly() {
        let expr = CronExpr::parse("@monthly").unwrap();
        assert_eq!(expr.day_of_month.values, vec![1]);
    }

    #[test]
    fn test_alias_yearly() {
        let expr = CronExpr::parse("@yearly").unwrap();
        assert_eq!(expr.month.values, vec![1]);
        assert_eq!(expr.day_of_month.values, vec![1]);
    }

    #[test]
    fn test_alias_annually() {
        let a = CronExpr::parse("@annually").unwrap();
        let b = CronExpr::parse("@yearly").unwrap();
        assert_eq!(a.month, b.month);
    }

    #[test]
    fn test_invalid_alias() {
        assert!(matches!(
            CronExpr::parse("@bogus"),
            Err(ParseError::InvalidAlias(_))
        ));
    }

    // -- Error tests --

    #[test]
    fn test_invalid_field_count() {
        assert!(matches!(
            CronExpr::parse("* * *"),
            Err(ParseError::InvalidFieldCount)
        ));
        assert!(matches!(
            CronExpr::parse("* * * * * *"),
            Err(ParseError::InvalidFieldCount)
        ));
    }

    #[test]
    fn test_invalid_field_value() {
        assert!(matches!(
            CronExpr::parse("abc * * * *"),
            Err(ParseError::InvalidField { .. })
        ));
    }

    #[test]
    fn test_value_out_of_range() {
        assert!(matches!(
            CronExpr::parse("60 * * * *"),
            Err(ParseError::ValueOutOfRange { .. })
        ));
        assert!(matches!(
            CronExpr::parse("* 25 * * *"),
            Err(ParseError::ValueOutOfRange { .. })
        ));
    }

    // -- matches() tests --

    #[test]
    fn test_matches_every_minute() {
        let expr = CronExpr::parse("* * * * *").unwrap();
        let dt = DateTime { year: 2026, month: 6, day: 15, hour: 14, minute: 30, second: 0 };
        assert!(expr.matches(&dt));
    }

    #[test]
    fn test_matches_specific_time() {
        let expr = CronExpr::parse("30 9 * * *").unwrap();
        let dt = DateTime { year: 2026, month: 3, day: 15, hour: 9, minute: 30, second: 0 };
        assert!(expr.matches(&dt));
        let dt2 = DateTime { year: 2026, month: 3, day: 15, hour: 10, minute: 30, second: 0 };
        assert!(!expr.matches(&dt2));
    }

    #[test]
    fn test_matches_weekday() {
        let expr = CronExpr::parse("0 9 * * 1-5").unwrap();
        // 2026-03-16 is Monday
        let monday = DateTime { year: 2026, month: 3, day: 16, hour: 9, minute: 0, second: 0 };
        assert!(expr.matches(&monday));
        // 2026-03-15 is Sunday
        let sunday = DateTime { year: 2026, month: 3, day: 15, hour: 9, minute: 0, second: 0 };
        assert!(!expr.matches(&sunday));
    }

    // -- next_from() tests --

    #[test]
    fn test_next_from_every_minute() {
        let expr = CronExpr::parse("* * * * *").unwrap();
        let dt = DateTime { year: 2026, month: 3, day: 15, hour: 10, minute: 30, second: 0 };
        let next = expr.next_from(&dt).unwrap();
        assert_eq!(next.minute, 31);
        assert_eq!(next.hour, 10);
    }

    #[test]
    fn test_next_from_weekday_morning() {
        let expr = CronExpr::parse("0 9 * * 1-5").unwrap();
        // Monday 8am -> should give Monday 9am
        let dt = DateTime { year: 2026, month: 3, day: 16, hour: 8, minute: 0, second: 0 };
        let next = expr.next_from(&dt).unwrap();
        assert_eq!(next, DateTime { year: 2026, month: 3, day: 16, hour: 9, minute: 0, second: 0 });
    }

    #[test]
    fn test_next_from_weekday_after_time() {
        let expr = CronExpr::parse("0 9 * * 1-5").unwrap();
        // Monday 10am -> should give Tuesday 9am
        let dt = DateTime { year: 2026, month: 3, day: 16, hour: 10, minute: 0, second: 0 };
        let next = expr.next_from(&dt).unwrap();
        assert_eq!(next, DateTime { year: 2026, month: 3, day: 17, hour: 9, minute: 0, second: 0 });
    }

    #[test]
    fn test_next_from_friday_to_monday() {
        let expr = CronExpr::parse("0 9 * * 1-5").unwrap();
        // Friday 10am -> Monday 9am
        let dt = DateTime { year: 2026, month: 3, day: 20, hour: 10, minute: 0, second: 0 };
        let next = expr.next_from(&dt).unwrap();
        // 2026-03-23 is Monday
        assert_eq!(next, DateTime { year: 2026, month: 3, day: 23, hour: 9, minute: 0, second: 0 });
    }

    #[test]
    fn test_next_from_end_of_month() {
        let expr = CronExpr::parse("0 0 1 * *").unwrap();
        let dt = DateTime { year: 2026, month: 1, day: 31, hour: 12, minute: 0, second: 0 };
        let next = expr.next_from(&dt).unwrap();
        assert_eq!(next.month, 2);
        assert_eq!(next.day, 1);
    }

    #[test]
    fn test_next_from_leap_year_feb_29() {
        let expr = CronExpr::parse("0 0 29 2 *").unwrap();
        let dt = DateTime { year: 2024, month: 3, day: 1, hour: 0, minute: 0, second: 0 };
        let next = expr.next_from(&dt).unwrap();
        // Next Feb 29 is 2028
        assert_eq!(next.year, 2028);
        assert_eq!(next.month, 2);
        assert_eq!(next.day, 29);
    }

    #[test]
    fn test_next_from_every_15_minutes() {
        let expr = CronExpr::parse("*/15 * * * *").unwrap();
        let dt = DateTime { year: 2026, month: 3, day: 15, hour: 10, minute: 3, second: 0 };
        let next = expr.next_from(&dt).unwrap();
        assert_eq!(next.minute, 15);
        assert_eq!(next.hour, 10);
    }

    // -- next_n_from() tests --

    #[test]
    fn test_next_n_from_count() {
        let expr = CronExpr::parse("0 * * * *").unwrap();
        let dt = DateTime { year: 2026, month: 1, day: 1, hour: 0, minute: 0, second: 0 };
        let times = expr.next_n_from(&dt, 5);
        assert_eq!(times.len(), 5);
        assert_eq!(times[0].hour, 1);
        assert_eq!(times[1].hour, 2);
        assert_eq!(times[4].hour, 5);
    }

    #[test]
    fn test_next_n_from_returns_ordered() {
        let expr = CronExpr::parse("*/30 * * * *").unwrap();
        let dt = DateTime { year: 2026, month: 1, day: 1, hour: 0, minute: 0, second: 0 };
        let times = expr.next_n_from(&dt, 4);
        for w in times.windows(2) {
            assert!(w[0] < w[1]);
        }
    }

    // -- describe() tests --

    #[test]
    fn test_describe_every_15_minutes() {
        let expr = CronExpr::parse("*/15 * * * *").unwrap();
        assert_eq!(expr.describe(), "Every 15 minutes");
    }

    #[test]
    fn test_describe_every_minute() {
        let expr = CronExpr::parse("* * * * *").unwrap();
        assert_eq!(expr.describe(), "Every minute");
    }

    #[test]
    fn test_describe_at_specific_time_weekdays() {
        let expr = CronExpr::parse("0 9 * * 1-5").unwrap();
        let desc = expr.describe();
        assert!(desc.contains("9:00 AM"), "got: {}", desc);
        assert!(desc.contains("Monday through Friday"), "got: {}", desc);
    }

    #[test]
    fn test_describe_monthly() {
        let expr = CronExpr::parse("0 9 1 * *").unwrap();
        let desc = expr.describe();
        assert!(desc.contains("9:00 AM"), "got: {}", desc);
        assert!(desc.contains("day"), "got: {}", desc);
    }

    #[test]
    fn test_describe_hourly_alias() {
        let expr = CronExpr::parse("@hourly").unwrap();
        assert_eq!(expr.describe(), "Every hour");
    }

    #[test]
    fn test_describe_daily_alias() {
        let expr = CronExpr::parse("@daily").unwrap();
        assert_eq!(expr.describe(), "Every day at midnight");
    }

    #[test]
    fn test_describe_yearly_alias() {
        let expr = CronExpr::parse("@yearly").unwrap();
        assert_eq!(expr.describe(), "At midnight on January 1st");
    }

    // -- Edge cases --

    #[test]
    fn test_end_of_year_rollover() {
        let expr = CronExpr::parse("0 0 1 1 *").unwrap();
        let dt = DateTime { year: 2026, month: 12, day: 31, hour: 23, minute: 59, second: 0 };
        let next = expr.next_from(&dt).unwrap();
        assert_eq!(next, DateTime { year: 2027, month: 1, day: 1, hour: 0, minute: 0, second: 0 });
    }

    #[test]
    fn test_specific_month_and_dow() {
        let expr = CronExpr::parse("0 10 * 6 MON").unwrap();
        let dt = DateTime { year: 2026, month: 5, day: 1, hour: 0, minute: 0, second: 0 };
        let next = expr.next_from(&dt).unwrap();
        assert_eq!(next.month, 6);
        assert_eq!(next.day_of_week(), 1); // Monday
        assert_eq!(next.hour, 10);
        assert_eq!(next.minute, 0);
    }

    #[test]
    fn test_datetime_now_is_reasonable() {
        let now = DateTime::now();
        assert!(now.year >= 2024);
        assert!((1..=12).contains(&now.month));
        assert!((1..=31).contains(&now.day));
    }

    #[test]
    fn test_parse_list_with_names() {
        let expr = CronExpr::parse("0 0 * * MON,WED,FRI").unwrap();
        assert_eq!(expr.day_of_week.values, vec![1, 3, 5]);
    }

    #[test]
    fn test_parse_month_list_names() {
        let expr = CronExpr::parse("0 0 1 JAN,JUN,DEC *").unwrap();
        assert_eq!(expr.month.values, vec![1, 6, 12]);
    }

    #[test]
    fn test_zero_step_is_error() {
        assert!(CronExpr::parse("*/0 * * * *").is_err());
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::InvalidFieldCount;
        assert_eq!(format!("{}", err), "cron expression must have exactly 5 fields");

        let err = ParseError::InvalidAlias("@bogus".to_string());
        assert!(format!("{}", err).contains("@bogus"));
    }
}
