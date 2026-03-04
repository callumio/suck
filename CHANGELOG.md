# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
## [0.0.3] - 2026-03-04

### 🚀 Features

- Remove closed flag from ChannelState
- Add internal constructor for `Sucker`/`Sourcer`
- Implement asynchronous channel support with tokio integration

### 🐛 Bug Fixes

- Correct toolchain in flake

### 🚜 Refactor

- Move traits to sync module and update imports
- Reorganize channel modules and implement async/sync structures

### 🧪 Testing

- Set_mut tests
- Increase code coverage of failure paths

### ⚙️ Miscellaneous Tasks

- Remove unused traits module
- Reorganize module exports for async and sync features
## [0.0.2] - 2025-09-04

### 🚀 Features

- Add multiple channel providers
## [0.0.1] - 2025-09-02

### 🚀 Features

- Add error types
- Add message types to be sent over channels
- Add initial synchronous channel implementation

### 🐛 Bug Fixes

- Rename error to be more descriptive
- Implement error for error types

### 💼 Other

- Typo

### 📚 Documentation

- Populate README
- Reorganise README
- Add instability warning

### 🧪 Testing

- Add basic tests for synchronous channel

### ⚙️ Miscellaneous Tasks

- Add metadata to Cargo.toml
