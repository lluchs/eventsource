# Changelog

## 0.6.0 - 2021-03-13

### Changed
 - Update reqwest to 0.11.x. This implies tokio ^1.0.

## 0.5.0 - 2020-04-21
### Added
 - `Client::new_with_client()` allows passing a custom reqwest client. ([#8](https://github.com/lluchs/eventsource/pull/8))

### Changed
 - Update reqwest to 0.10.x. We are still using the blocking API.

## 0.4.0 - 2019-09-16
### Changed
 - Update reqwest to 0.9.x

## 0.3.0 - 2017-10-27
### Changed
 - Update reqwest to 0.8.0
 - `Client::new()` no longer returns a `Result`. This is a breaking
   change carried over from reqwest 0.8.0.

### Fixed
 - Infinite loop when error occur in the stream (#2)
