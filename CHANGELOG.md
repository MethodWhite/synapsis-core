# Changelog

All notable changes to Synapsis Core will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0](https://github.com/MethodWhite/synapsis-core/compare/v0.6.0...v0.7.0) (2026-07-14)


### Features

* implement real PQC (ML-KEM-1024 + ML-DSA-87) ([6b8885b](https://github.com/MethodWhite/synapsis-core/commit/6b8885bc25f14311fb1bbbc99e57a0c30cf7fd95))

## [0.6.0](https://github.com/MethodWhite/synapsis-core/compare/v0.5.1...v0.6.0) (2026-07-11)


### Features

* add action version switcher script ([432cb15](https://github.com/MethodWhite/synapsis-core/commit/432cb151f961cd9682fe0a306924f34043a108a9))
* add Gitleaks scanning, migrate labeler v5→v6 ([92d2432](https://github.com/MethodWhite/synapsis-core/commit/92d2432ffa5728f3de73f3336254bdaa7b66cd55))
* add release-please workflow for automated versioning ([ebea1c9](https://github.com/MethodWhite/synapsis-core/commit/ebea1c94782f61eba551df2df36a6fac321df811))
* apply PR [#44](https://github.com/MethodWhite/synapsis-core/issues/44) improvements to core - deny.toml, CodeQL, OSV, autoformat, lock ([43afe85](https://github.com/MethodWhite/synapsis-core/commit/43afe856847605f91051aa1466dd25b3238dc73a))
* centralized action version management ([4bbeb70](https://github.com/MethodWhite/synapsis-core/commit/4bbeb70eb1c25aa8410377bc83c876a88f57b3f0))
* enhance synapsis-core with FTS5, PQC migration, doc comments, cleanup ([d7936d9](https://github.com/MethodWhite/synapsis-core/commit/d7936d94417fedfa0c2e58f7b0605fc6d7dedcf3))


### Bug Fixes

* add .gitignore, remove tracked build artifacts ([efc320c](https://github.com/MethodWhite/synapsis-core/commit/efc320c66ff1dedb20b3dbebd5733f97cc6baaf6))
* align public API with synapsis (audit_log, pqc, antibrick types) ([63d7ff6](https://github.com/MethodWhite/synapsis-core/commit/63d7ff691733ef42d1a886fd16af413fc61b2f35))
* align public API with synapsis (audit_log, pqc, antibrick types) ([6108c60](https://github.com/MethodWhite/synapsis-core/commit/6108c601c539061ddc4934cde5b7672436ded615))
* CI permissions and formatting ([83dc8a8](https://github.com/MethodWhite/synapsis-core/commit/83dc8a8ace5b2e363ff3168c5e7bc0ee66ca0496))
* CI respects rust-toolchain.toml, add timeouts ([b88d086](https://github.com/MethodWhite/synapsis-core/commit/b88d0869bc4c57fa1c010021b17a40f1b4d4f94c))
* dtolnay/rust-toolchain requires [@master](https://github.com/master) ref ([62269f4](https://github.com/MethodWhite/synapsis-core/commit/62269f464eb9f622dd6040214426ce3089531a90))
* exclude component name from release tag ([70ed0b4](https://github.com/MethodWhite/synapsis-core/commit/70ed0b439ce7103b15f68b9f7664591db45895f2))
* labeler v5 any: inline array ([9a1a9be](https://github.com/MethodWhite/synapsis-core/commit/9a1a9bec4a19f1c0c7d776630a386da66acd2678))
* labeler v5 flat format ([86452f7](https://github.com/MethodWhite/synapsis-core/commit/86452f72624f0b805b13b4a84c0e76f7f18244df))
* remove Windows from CI matrix ([6548c13](https://github.com/MethodWhite/synapsis-core/commit/6548c13c4a4838497d699f59de88c40b02145456))
* replace deprecated Nonce::from_slice with TryFrom ([962d21d](https://github.com/MethodWhite/synapsis-core/commit/962d21d0eca804580a9a6eb47329670e7a620450))
* restrict dependabot to avoid broken major version bumps ([09a869a](https://github.com/MethodWhite/synapsis-core/commit/09a869a92e8b1b7df270d078354434426729af1a))
* update OSV scanner to v2.3.8 (fixes deprecated upload-artifact v3) ([31adcb1](https://github.com/MethodWhite/synapsis-core/commit/31adcb11bd9ce49f6bee12cd9ae0877348257643))
* use dtolnay/rust-toolchain@stable ([7def37c](https://github.com/MethodWhite/synapsis-core/commit/7def37cb6f9632b98dadb9b7766d26f3cbd3a341))
* use osv-scanner v1.9.2 (v2 startup failure) ([76880eb](https://github.com/MethodWhite/synapsis-core/commit/76880eb54e40026af4fabe4075cf0d75d5b18f14))

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
