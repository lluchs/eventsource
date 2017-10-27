# Changelog

## 0.3.0 - 2017-10-27
### Changed
 - Update reqwest to 0.8.0
 - `Client::new()` no longer returns a `Result`. This is a breaking
   change carried over from reqwest 0.8.0.

### Fixed
 - Infinite loop when error occur in the stream (#2)
