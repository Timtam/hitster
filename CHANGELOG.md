# Hitster Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2025.7.23-1] - 2025-07-23

### Added

-   [added option to select random packs within the game settings for a totally chaotic and fun gaming experience (#28)](https://github.com/Timtam/hitster/issues/28)

### Fixed

-   [fixed currently playing hits to restart whenever the SFX volume is changed (#30)](https://github.com/Timtam/hitster/issues/30)
-   [fixed html characters in hit data to cause frontend render issues (#31)](https://github.com/Timtam/hitster/issues/31)

### Changed

-   [changed the hits download process to run in the background while already providing a web frontend, allowing the user to monitor the progress. This also introduces features to decide on the hits download method (Docker containers will use yt-dlp by default now) (#27)](https://github.com/Timtam/hitster/issues/27)
-   updated Rust to 1.88
-   updated yt-dlp to 2025.06.30
-   tiny translation changes
-   update dependencies
-   update hits, including two new packs (Movie Soundtracks and Poland)

## [2025.5.3-1] - 2025-05-03

### Added

-   [added various key shortcuts across the entire app (#2)](https://github.com/Timtam/hitster/issues/2)
-   [added a how to play section on the welcome page (#11)](https://github.com/Timtam/hitster/issues/11)
-   [added 'belongs to' field to hits view table (#21)](https://github.com/Timtam/hitster/issues/21)
-   [added confirmation prompt when leaving a game that is currently running (#20)](https://github.com/Timtam/hitster/issues/20)

### Fixed

-   [fixed client not returning to the lobby if the creator of a local game leaves the game (#19)](https://github.com/Timtam/hitster/issues/19)
-   [fixed a server crash when starting a game with too few hits (#18)](https://github.com/Timtam/hitster/issues/18)

### Changed

-   [restyled the slot selector UI to look much nicer for sighted folk (#13)](https://github.com/Timtam/hitster/issues/13)
-   updated Rust to 1.86
-   updated yt-dlp to 2025.03.31
-   updated crates
-   added and fixed some hits
-   [added some debug messages to inspect user authentification issues (#24)](https://github.com/Timtam/hitster/issues/24)

## [2024.12.13-1] - 2024-12-13

### Changed

-   [use ogg instead of mp3 for SFX to change from html5 to webaudio, improving browser compatibility (#12)](https://github.com/Timtam/hitster/issues/12)
-   updated several packages for client and server
-   updated CI pipeline to hopefully address some versioning issues

## [2024.12.11-1] - 2024-12-11

### Added

-   [Add dark mode option (change within settings) (#16)](https://github.com/Timtam/hitster/issues/16)
-   loads of hits (including several japanese anime songs)

### Fixed

-   [kicking a non-existing player from a game no longer crashes the server (#14)](https://github.com/Timtam/hitster/issues/14)

### Changed

-   [Add local player modal no longer requires you to press the "Add" button, you can also just press return within the input box now (#15)](https://github.com/Timtam/hitster/issues/15)
-   [New versioning (mixture of release date and semver) + changelog #17)](https://github.com/Timtam/hitster/issues/17)

[Unreleased]: https://github.com/Timtam/hitster/compare/2025.7.23-1...HEAD

[2025.7.23-1]: https://github.com/Timtam/hitster/compare/2025.5.3-1...2025.7.23-1

[2025.5.3-1]: https://github.com/Timtam/hitster/compare/2024.12.13-1...2025.5.3-1

[2024.12.13-1]: https://github.com/Timtam/hitster/compare/2024.12.11-1...2024.12.13-1

[2024.12.11-1]: https://github.com/Timtam/hitster/releases/tag/2024.12.11-1
