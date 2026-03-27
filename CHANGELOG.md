# Changelog

## 0.2.2 (2026-03-27)

- Add GitHub issue templates, PR template, and dependabot configuration
- Update README badges and add Support section

## 0.2.1 (2026-03-22)

- Fix CHANGELOG compliance

## 0.2.0 (2026-03-20)

- Add FromStr trait implementation for CronExpr
- Add Display trait implementation for CronExpr
- Add Hash and Copy derives to DateTime
- Add to_timestamp() method on DateTime
- Add #[must_use] attributes on query methods

## 0.1.6 (2026-03-17)

- Add readme, rust-version, documentation to Cargo.toml
- Add Development section to README

## 0.1.5 (2026-03-16)

- Update install snippet to use full version

## 0.1.4 (2026-03-16)

- Add README badges
- Synchronize version across Cargo.toml, README, and CHANGELOG

## 0.1.0 (2026-03-15)

- Initial release
- Standard 5-field cron expression parsing
- Aliases: @hourly, @daily, @weekly, @monthly, @yearly
- Next execution time calculation
- Human-readable cron descriptions
- Zero dependencies
