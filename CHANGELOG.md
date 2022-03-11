# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

* Added `Queue::wait_full` and `Queue::wait_empty` async methods

## [0.2.4] - 2022-11-4

### Fixed

- Updated `tokio` required features to include `macros`

## [0.2.3] - 2022-06-05

### Added

- Added `is_full` to limited and resizable queues

## [0.2.2] - 2022-05-01

### Added

- Fix `resize` implementation of resizable queue

## [0.2.1] - 2022-03-11

### Added

- Implement `Debug` for all queues
- Add `Queue::is_empty` method

## [0.2.0] - 2020-12-26

### Changed

- Update `tokio` dependency to version 1
- Update `crossbeam-queue` dependency to version 0.3

## [0.1.0] - 2020-01-21

### Added

- First release

[unreleased]: https://github.com/bikeshedder/deadqueue/compare/v0.2.4...HEAD
[0.1.0]: https://github.com/bikeshedder/deadqueue/releases/tag/v0.1.0
[0.2.0]: https://github.com/bikeshedder/deadqueue/releases/tag/v0.2.0
[0.2.1]: https://github.com/bikeshedder/deadqueue/releases/tag/v0.2.1
[0.2.2]: https://github.com/bikeshedder/deadqueue/releases/tag/v0.2.2
[0.2.3]: https://github.com/bikeshedder/deadqueue/releases/tag/v0.2.3
[0.2.4]: https://github.com/bikeshedder/deadqueue/releases/tag/v0.2.4
