# Changelog

All notable changes to Synapsis Core will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-06-12

### Added

- **Token-efficient memory** — configurable summarization engine that compresses conversation history into compact representations, with importance scoring per segment and budget-based retention policies to stay within context limits
- **Semantic search with embeddings** — vector-based retrieval using embedding models for similarity search across stored memories, enabling natural-language queries against historical context
- **Chunking pipeline** — intelligent document/text splitting with configurable chunk size, overlap, and boundary detection strategies for efficient memory ingestion
- **Knowledge Graph** — entity extraction and relationship mapping with graph query support, enabling structured knowledge representation and traversal across connected concepts
- **Modular architecture** — complete restructure into 26 focused source files across domain, storage, and retrieval layers for better maintainability and testability
- **PQC security primitives** — CRYSTALS-Kyber KEM and CRYSTALS-Dilithium digital signatures for post-quantum cryptography

### Changed

- Architecture split from monolithic `lib.rs` into specialized modules
- Core traits and types in dedicated domain module
- SQLite storage layer extracted into standalone repository pattern
- Encryption upgraded to AES-256-GCM with PBKDF2 key derivation (600K iterations)

### Fixed

- N/A (initial public release of structured crate)

## [0.3.0] - 2026-05-03

### Added
- Initial extraction from Synapsis monolith into standalone crate
- SQLite-backed persistent store with encryption
- PQC security primitives (CRYSTALS-Kyber, CRYSTALS-Dilithium)
- Session management
- Distributed lock primitives
- Task queue
- Full MCP tool definitions and server traits

[Unreleased]: https://github.com/methodwhite/synapsis-core/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/methodwhite/synapsis-core/releases/tag/v0.4.0
[0.3.0]: https://github.com/methodwhite/synapsis-core/releases/tag/v0.3.0
