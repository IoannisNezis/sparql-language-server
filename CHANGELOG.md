# Change Log

All notable changes to the "Qlue-ls" project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]


## [0.3.3] - 2025-02-02

### Fixed

- Fixed bugs in formatter

## [0.3.2] - 2025-01-31

### Added

- stability test for formatter

### Fixed

- fixed typo in diagnostic
- reimplemented formatting options for new formatting algorithm

## [0.3.1] - 2025-01-30

### Added

- formatting inline format statements

### Fixed

- formatting input with comments at any location

## [0.3.0] - 2025-01-20

### Added

- new format option "check": dont write anything, just check if it would

## [0.2.4] - 2025-01-20

### Fixed

- add trailing newline when formatting with format cli subcommand

## [0.2.3] - 2025-01-12

### Fixed

- positions are (by default) utf-16 based, i changed the implementation to respect this

## [0.2.2] - 2025-01-09

### Fixed

- handle textdocuments-edits with utf-8 characters

## [0.2.1] - 2025-01-09

### Fixed

- formatting strings with commas

## [0.2.0] - 2025-01-09

### Added

- new code-action: declare prefix
- example for monaco-editor with a language-client attached to this language-server
- formatter subcommand uses user-configuration
- this CHANGELOG

### Fixed

- format subcommand writeback-bug
- formatting of Blank and ANON nodes

### Changed

- format cli subcommand: --writeback option, prints to stdout by default
