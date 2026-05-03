# 🧩 Synapsis Core

> **Core library for Synapsis - Persistent Memory Engine with PQC Security**

[![License: BUSL-1.1](https://img.shields.io/badge/License-BUSL--1.1-red.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-v1.75+-orange.svg)](https://www.rust-lang.org)
[![Security](https://img.shields.io/badge/Security-PQC-green.svg)](docs/SECURITY.md)
[![Plugins](https://img.shields.io/badge/Plugins-Dynamic-blue.svg)](docs/PLUGIN_SYSTEM_GUIDE.md)

---

## ⚠️ License Important Notice

**This software is licensed under BUSL-1.1 (Business Source License 1.1)**

- ✅ **Permitted:** Personal, educational, and research use
- ❌ **Restricted:** Commercial/enterprise use requires commercial license
- ⚖️ **Violations:** 100% of profits + statutory damages up to $150,000

**For commercial licensing:** methodwhite@proton.me · methodwhite.developer@gmail.com

---

## Overview

Synapsis Core is the foundational library providing:

- 🔐 **Post-Quantum Cryptography** - Kyber-512/768/1024, Dilithium-2/3/5
- 🧩 **Plugin System** - Dynamic loading of .so/.dylib/.dll plugins
- 📦 **Domain Types** - Core entities, errors, and traits
- ⚙️ **Business Logic** - Auth, task queue, vault, orchestration
- 🗄️ **Infrastructure** - Database, events, agents, skills

---

## Installation

```toml
[dependencies]
synapsis-core = { path = "../synapsis-core", features = ["full"] }
```

### Features

| Feature | Description |
|---------|-------------|
| `security` | Encryption, HMAC, Argon2 (default) |
| `pqc` | Post-quantum cryptography (default) |
| `network` | mDNS discovery |
| `monitoring` | System monitoring with sysinfo |
| `full` | All features enabled |

---

## Quick Start

### PQC Cryptography

```rust
use synapsis_core::{PqcryptoProvider, domain::crypto::{CryptoProvider, PqcAlgorithm}};

let provider = PqcryptoProvider::new();

// Generate Kyber-768 keypair
let (pk, sk) = provider.generate_keypair(PqcAlgorithm::Kyber768)?;

// Encapsulate shared secret
let (ct, shared_secret) = provider.encapsulate(&pk, PqcAlgorithm::Kyber768)?;

// Decapsulate
let shared_secret2 = provider.decapsulate(&ct, &sk, PqcAlgorithm::Kyber768)?;

assert_eq!(shared_secret, shared_secret2);
```

### Plugin System

```rust
use synapsis_core::domain::plugin::{DynamicPluginLoader, PluginRegistry};

let mut loader = DynamicPluginLoader::new();
let mut registry = PluginRegistry::new();

// Load plugin dynamically
loader.load_and_register("/path/to/my_plugin.so", &mut registry)?;

println!("Loaded {} plugins", loader.loaded_count());
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│              synapsis-core (library)                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │ domain/     │  │ core/       │  │ infrastructure/ │ │
│  │ - crypto    │  │ - auth      │  │ - database      │ │
│  │ - plugin    │  │ - task_queue│  │ - event_bus     │ │
│  │ - errors    │  │ - vault     │  │ - agents        │ │
│  │ - types     │  │ - pqc       │  │ - skills        │ │
│  └─────────────┘  └─────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

---

## Extension Points

Plugins can register at these extension points:

| Extension Point | Purpose |
|----------------|---------|
| `CryptoProvider` | Cryptography implementations |
| `AuthProvider` | Authentication systems |
| `StorageBackend` | Storage backends (S3, IPFS, etc.) |
| `LlmProvider` | LLM providers (Ollama, OpenAI) |
| `WorkerAgent` | Worker agents (Code, Search, Shell) |
| `RpcHandler` | Custom RPC handlers |
| `TaskQueue` | Task queue implementations |
| `DatabaseAdapter` | Database adapters |
| `Monitoring` | Monitoring and telemetry |
| `AuditLogging` | Audit logging systems |

---

## Documentation

| Document | Description |
|----------|-------------|
| [PLUGIN_SYSTEM_GUIDE.md](docs/PLUGIN_SYSTEM_GUIDE.md) | Complete plugin development guide |
| [PQC_PLUGIN_MIGRATION.md](docs/PQC_PLUGIN_MIGRATION.md) | PQC provider consolidation |
| [MODULARIZACION_ESTADO_REAL.md](docs/MODULARIZACION_ESTADO_REAL.md) | Ecosystem analysis |

---

## Development

### Build

```bash
# Debug build
cargo build

# Release build
cargo build --release

# All features
cargo build --features full
```

### Test

```bash
# Run all tests
cargo test

# Dynamic plugin tests (requires compiled plugin)
cargo test --test dynamic_plugin_loading

# With coverage
cargo tarpaulin --out Html
```

### Generate Docs

```bash
cargo doc --open
```

---

## Project Structure

```
synapsis-core/
├── Cargo.toml
├── LICENSE              # BUSL-1.1 (restrictive)
├── README.md
├── src/
│   ├── lib.rs
│   ├── domain/          # Core types, traits, errors
│   │   ├── crypto.rs    # CryptoProvider trait
│   │   ├── plugin.rs    # Plugin system
│   │   ├── plugin_loader.rs  # Dynamic loading
│   │   ├── errors.rs    # Error types
│   │   └── types.rs     # Domain types
│   ├── core/            # Business logic
│   │   ├── auth/        # Authentication system
│   │   ├── pqc.rs       # PQC implementation
│   │   ├── pqcrypto_provider.rs  # Unified PQC provider
│   │   ├── task_queue.rs  # Task management
│   │   └── vault.rs     # Secure vault
│   └── infrastructure/  # Adapters
│       ├── database.rs  # SQLite database
│       ├── event_bus.rs # Event system
│       └── agents.rs    # Agent registry
├── tests/
│   └── dynamic_plugin_loading.rs  # Plugin loading tests
└── docs/                # Documentation
```

---

## Security

Synapsis Core implements military-grade security:

| Level | Component | Technology |
|-------|-----------|------------|
| ⭐ | PQC Cryptography | CRYSTALS-Kyber, CRYSTALS-Dilithium |
| ⭐⭐ | Symmetric Encryption | AES-256-GCM, ChaCha20-Poly1305 |
| ⭐⭐⭐ | Key Derivation | Argon2id |
| ⭐⭐⭐⭐ | Secure Storage | SQLCipher, encrypted vault |
| ⭐⭐⭐⭐⭐ | Plugin Security | Signature verification (planned) |

See [SECURITY.md](docs/SECURITY.md) for details.

---

## Commercial Use

### Who Needs a Commercial License?

You need a commercial license if you:

- ❌ Use Synapsis Core in a **business or organization**
- ❌ Deploy in **production** for commercial purposes
- ❌ Offer **SaaS** or hosted services using Synapsis
- ❌ **Integrate** into commercial products
- ❌ Provide **consulting services** using Synapsis
- ❌ Run **training programs** for profit

### License Tiers

| Tier | Employees | Revenue | Contact |
|------|-----------|---------|---------|
| **Startup** | <10 | <$1M | methodwhite@proton.me |
| **Business** | 10-100 | <$10M | methodwhite.developer@gmail.com |
| **Enterprise** | 100+ | Unlimited | methodwhite.developer@gmail.com |
| **SaaS** | Any | Any | methodwhite.developer@gmail.com |
| **OEM** | Any | Any | methodwhite.developer@gmail.com |

---

## Legal Notice

**VIOLATION CONSEQUENCES:**

1. **Immediate termination** of all rights
2. **100% disgorgement** of all profits from unauthorized use
3. **Statutory damages** up to $150,000 per willful violation
4. **Treble damages** for willful and malicious violations
5. **Legal fees** and enforcement costs
6. **Injunctive relief** without bond
7. **Criminal referral** for willful copyright infringement

**Audit Rights:** Licensor reserves the right to audit your use upon reasonable notice if violation is suspected.

---

## Contact

**Author:** MethodWhite  
**Email:** methodwhite@proton.me (primary) · methodwhite.developer@gmail.com (enterprise)  
**GitHub:** https://github.com/methodwhite/synapsis  

---

**Built with ❤️ and 🦀 by MethodWhite**

*Last updated: 2026-03-24*
