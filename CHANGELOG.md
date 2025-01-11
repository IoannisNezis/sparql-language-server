# Change Log

All notable changes to the "qlue-ls" project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.

## [Unreleased]

## [0.2.2] 2025-01-09

### Fixed

- handle textdocuments-edits with utf-8 characters

## [0.2.1] 2025-01-09

### Fixed

- formatting strings with commas

## [0.2.0] 2025-01-09

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
