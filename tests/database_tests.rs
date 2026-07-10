use synapsis_core::domain::entities::{
    Observation, SearchParams,
};
use synapsis_core::domain::ports::StorageBackend;
use synapsis_core::domain::types::{ObservationType, SessionId};
use synapsis_core::infrastructure::database::{get_str, Database, SqliteBackend};
use synapsis_core::DbValue;

fn setup_db() -> Database {
    let db = Database::new();
    db.init().unwrap();
    db
}

#[test]
fn test_backend_trait_with_sqlite() {
    use rusqlite::Connection;
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY, val TEXT)")
        .unwrap();
    let backend = SqliteBackend::new(conn);
    backend
        .execute_batch("INSERT INTO test (id, val) VALUES (1, 'hello'), (2, 'world')")
        .unwrap();
    let rows = backend
        .query("SELECT id, val FROM test ORDER BY id", &[])
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(get_str(&rows[0][1]), Some("hello"));
    assert_eq!(get_str(&rows[1][1]), Some("world"));
    let affected = backend
        .execute(
            "UPDATE test SET val = ?1 WHERE id = ?2",
            &[
                DbValue::Text("updated".into()),
                DbValue::Integer(1),
            ],
        )
        .unwrap();
    assert_eq!(affected, 1);
    let rows = backend
        .query("SELECT val FROM test WHERE id = 1", &[])
        .unwrap();
    assert_eq!(get_str(&rows[0][0]), Some("updated"));
}

#[test]
fn test_database_with_backend_trait() {
    let backend = SqliteBackend::new(rusqlite::Connection::open_in_memory().unwrap());
    let db = Database::from_backend(Box::new(backend));
    db.init().unwrap();
    let obs = Observation::new(
        SessionId::new("backend-test"),
        ObservationType::Memory,
        "backend trait test",
        "testing the storage backend trait integration",
    );
    let id = db.save_observation(&obs).unwrap();
    assert!(id.0 > 0);
    let results = db
        .search_observations(&SearchParams::new("backend trait"))
        .unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].observation.title, "backend trait test");
}

#[test]
fn test_chunk_storage_and_retrieval() {
    let db = setup_db();
    let session = SessionId::new("test");
    let long_content = "Machine learning is a subset of artificial intelligence. ".repeat(80);
    let obs = Observation::new(
        session.clone(),
        ObservationType::Memory,
        "chunk storage test",
        long_content.clone(),
    );
    let obs_id = db.save_observation(&obs).unwrap();

    let chunk_count: i64 = {
        let conn = db.get_conn().unwrap();
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM chunks WHERE observation_id = ?1")
            .unwrap();
        stmt.query_row(rusqlite::params![obs_id.0], |row| row.get(0))
            .unwrap()
    };
    assert!(
        chunk_count > 0,
        "chunks should have been stored in the DB, got {} chunks",
        chunk_count
    );

    let merged = synapsis_core::infrastructure::database::merge_chunks(obs_id.0 as u64, &db);
    assert!(
        merged.is_some(),
        "merge_chunks should return Some observation"
    );
    let merged = merged.unwrap();
    assert_eq!(
        merged.title, "chunk storage test",
        "merged observation should preserve title"
    );
    assert!(
        merged.content.len() >= long_content.len() / 2,
        "merged content should be substantial"
    );

    let results = db
        .search_observations(&SearchParams::new("machine learning"))
        .unwrap();
    assert!(
        !results.is_empty(),
        "chunked observation should be searchable"
    );
}

/// METRIC: Verify that saving and searching respects token budgets
#[test]
fn test_token_budget_enforced() {
    let db = setup_db();
    for i in 0..20 {
        let content = format!("memory entry number {} with some extra padding text to make it longer and consume more tokens for testing purposes", i);
        let obs = Observation::new(
            SessionId::new("test"),
            ObservationType::Memory,
            format!("entry-{}", i),
            content,
        );
        db.save_observation(&obs).unwrap();
    }

    let results_all = db
        .search_observations(&SearchParams::new("memory").with_max_tokens(10000))
        .unwrap();
    let results_tight = db
        .search_observations(&SearchParams::new("memory").with_max_tokens(50))
        .unwrap();

    assert!(
        results_tight.len() < results_all.len(),
        "Token budget of 50 should limit results (all={}, tight={})",
        results_all.len(),
        results_tight.len()
    );

    let total_tokens: u32 = results_all.iter().map(|r| r.token_cost).sum();
    let tight_tokens: u32 = results_tight.iter().map(|r| r.token_cost).sum();
    println!(
        "METRIC: no-budget search: {} results, {} tokens",
        results_all.len(),
        total_tokens
    );
    println!(
        "METRIC: budget=50 search: {} results, {} tokens",
        results_tight.len(),
        tight_tokens
    );
    assert!(
        tight_tokens <= 100,
        "token budget exceeded: {} > 100",
        tight_tokens
    );
}

/// METRIC: Verify importance scoring affects retrieval order
#[test]
fn test_importance_ordering() {
    let db = setup_db();
    for i in 0..10 {
        let content = if i < 3 {
            format!("critical error in memory system at iteration {}", i)
        } else {
            format!("regular log entry number {}", i)
        };
        let title = content.chars().take(60).collect::<String>();
        let obs = Observation::new(
            SessionId::new("test"),
            ObservationType::Memory,
            title,
            content,
        );
        db.save_observation(&obs).unwrap();
    }

    let results = db
        .search_observations(&SearchParams::new("memory"))
        .unwrap();

    assert!(!results.is_empty(), "should return results");
    for i in 1..results.len() {
        assert!(
            results[i - 1].observation.importance >= results[i].observation.importance,
            "results should be ordered by importance descending"
        );
    }
    println!(
        "METRIC: importance range: {:.2} - {:.2}",
        results.last().unwrap().observation.importance,
        results.first().unwrap().observation.importance
    );
}

/// METRIC: Test that retain() respects token budget
#[test]
fn test_retain_eviction() {
    let db = setup_db();
    for i in 0..100 {
        let obs = Observation::new(
            SessionId::new("test"),
            ObservationType::Memory,
            format!("entry-{}", i),
            "content ".repeat(10),
        );
        db.save_observation(&obs).unwrap();
    }

    let before = db.stats_db().unwrap();
    println!(
        "METRIC: before retain - entries={}, tokens={}",
        before.total_entries, before.total_tokens
    );

    let freed = db.retain(100).unwrap();
    let after = db.stats_db().unwrap();
    println!(
        "METRIC: after retain(100) - entries={}, tokens={}, freed={} tokens",
        after.total_entries, after.total_tokens, freed
    );

    assert!(
        after.total_tokens <= 1000,
        "retain should reduce tokens: {} > 1000",
        after.total_tokens
    );
}

/// BENCHMARK: Search speed — measure average search time across 10 random queries
#[test]
fn benchmark_search_speed() {
    let db = setup_db();
    let topics = [
        "machine learning",
        "data processing",
        "neural network",
        "memory management",
        "token optimization",
        "embedding vectors",
        "similarity search",
        "knowledge graph",
        "entity extraction",
        "relationship inference",
        "chunk management",
        "database query",
        "importance scoring",
        "content summarization",
        "efficient storage",
    ];

    for i in 0..500 {
        let topic = topics[i % topics.len()];
        let content = format!("{} is an important concept in modern systems. ", topic)
            .repeat(5 + (i % 10));
        let obs = Observation::new(
            SessionId::new("bench-speed"),
            ObservationType::Memory,
            format!(
                "speed-{}-{}",
                topic.split_whitespace().next().unwrap_or("topic"),
                i
            ),
            content,
        );
        db.save_observation(&obs).unwrap();
    }

    let queries = [
        "machine learning",
        "data processing",
        "knowledge graph",
        "similarity search",
        "memory management",
        "entity extraction",
        "neural network",
        "database query",
        "content summarization",
        "relationship inference",
    ];

    let start = std::time::Instant::now();
    for q in &queries {
        let _ = db
            .search_observations(&SearchParams::new(*q).with_limit(10))
            .unwrap();
    }
    let elapsed = start.elapsed();
    let avg = elapsed.as_micros() as f64 / queries.len() as f64;
    println!(
        "BENCHMARK: Search speed — {} queries in {:?}, avg {:.0}µs",
        queries.len(),
        elapsed,
        avg
    );
    assert!(avg < 1_000_000.0, "Average search time should be <1s");
}

/// BENCHMARK: Token budget effectiveness — tight budget should reduce tokens
#[test]
fn benchmark_token_budget_effectiveness() {
    let db = setup_db();
    let content_base = "memory entry about programming languages and frameworks. ";

    for i in 0..50 {
        let content = format!(
            "{}{}",
            content_base,
            "extra details for token counting. ".repeat(i)
        );
        let obs = Observation::new(
            SessionId::new("bench-budget"),
            ObservationType::Memory,
            format!("budget-entry-{}", i),
            content,
        );
        db.save_observation(&obs).unwrap();
    }

    let wide = db
        .search_observations(&SearchParams::new("memory").with_max_tokens(1000))
        .unwrap();
    let tight = db
        .search_observations(&SearchParams::new("memory").with_max_tokens(100))
        .unwrap();

    let wide_tokens: u32 = wide.iter().map(|r| r.token_cost).sum();
    let tight_tokens: u32 = tight.iter().map(|r| r.token_cost).sum();
    let ratio = if wide_tokens > 0 {
        tight_tokens as f64 / wide_tokens as f64
    } else {
        0.0
    };

    println!("BENCHMARK: Token budget — wide(1000)={} tokens, tight(100)={} tokens, reduction ratio={:.2}",
        wide_tokens, tight_tokens, ratio);
    assert!(
        tight_tokens <= 100 || tight.len() < wide.len(),
        "Tight budget should reduce token usage"
    );
}

/// BENCHMARK: Embedding similarity — Rust results rank higher than Python for "programming language"
#[test]
fn benchmark_embedding_similarity() {
    let db = setup_db();
    let session = SessionId::new("bench-embed");

    let rust_phrases = [
        "Rust programming language for systems development",
        "Rust has zero-cost abstractions and memory safety",
        "Writing efficient concurrent code in Rust",
        "Rust's ownership model eliminates data races",
        "The Rust compiler enforces memory safety at compile time",
        "Rust is used for building high-performance web servers",
        "Rust's pattern matching and type system are powerful",
        "Cargo is Rust's build system and package manager",
        "Rust supports functional programming patterns",
        "Rust's borrow checker ensures safe memory management",
        "Building CLI tools with Rust and clap",
        "Rust's async/await for concurrent programming",
        "Rust enums and match expressions for error handling",
        "Rust traits enable generic programming",
        "Rust's standard library provides rich collections",
        "Using Rust for embedded systems development",
        "Rust's FFI enables seamless C interoperability",
        "Rust's module system organizes code effectively",
        "Writing safe network protocols in Rust",
        "Rust's macro system for metaprogramming",
        "Rust's performance rivals C and C++",
        "Rust's documentation testing ensures correct examples",
        "Building WebAssembly modules with Rust",
        "Rust's iterator adaptors for data processing",
        "Rust's smart pointers for heap allocation",
        "Rust's lifetimes prevent dangling references",
        "Rust's const generics for compile-time computations",
        "Rust's procedural macros for code generation",
        "Rust's workspace for multi-crate projects",
        "Rust's cross-compilation for multiple targets",
        "Rust's type inference reduces boilerplate code",
        "Rust's Result type for robust error handling",
        "Rust's Option type eliminates null pointer exceptions",
        "Rust's tuples and structs for data modeling",
        "Rust's closures for anonymous functions",
        "Rust's iterators and generators for lazy evaluation",
        "Rust's channels for message passing concurrency",
        "Rust's Arc and Mutex for shared state",
        "Rust's Cow type for copy-on-write optimization",
        "Rust's formatting macros for string interpolation",
        "Rust's test framework for unit and integration tests",
        "Rust's bench tests for performance measurement",
        "Rust's attributes for conditional compilation",
        "Rust's derive macros for automatic trait implementations",
        "Rust's serde for serialization and deserialization",
        "Rust's tokio runtime for asynchronous I/O",
        "Rust's rayon for data parallelism",
        "Rust's nom for parser combinators",
        "Rust's clap for command-line argument parsing",
        "Rust's diesel ORM for database access",
    ];

    for (i, phrase) in rust_phrases.iter().cycle().take(100).enumerate() {
        let obs = Observation::new(
            session.clone(),
            ObservationType::Memory,
            format!("rust-observation-{}", i),
            *phrase,
        );
        db.save_observation(&obs).unwrap();
    }

    let python_obs = Observation::new(
        session.clone(),
        ObservationType::Memory,
        "python-web-dev",
        "Python web development with Django and Flask frameworks for building REST APIs",
    );
    db.save_observation(&python_obs).unwrap();

    let results = db
        .search_observations(
            &SearchParams::new("programming language")
                .with_semantic(true)
                .with_limit(110),
        )
        .unwrap();

    assert!(!results.is_empty(), "semantic search should return results");

    let rust_positions: Vec<usize> = results
        .iter()
        .enumerate()
        .filter(|(_, r)| r.observation.title.starts_with("rust-observation-"))
        .map(|(i, _)| i)
        .collect();
    let python_positions: Vec<usize> = results
        .iter()
        .enumerate()
        .filter(|(_, r)| r.observation.title == "python-web-dev")
        .map(|(i, _)| i)
        .collect();

    println!(
        "BENCHMARK: Embedding similarity — {} Rust results, {} Python result in top {}",
        rust_positions.len(),
        python_positions.len(),
        results.len()
    );

    for (i, &pos) in rust_positions.iter().take(5).enumerate() {
        println!(
            "BENCHMARK:   Rust result #{} at position {} with score {:.4}",
            i + 1,
            pos,
            results[pos].score
        );
    }
    if let Some(&py_pos) = python_positions.first() {
        println!(
            "BENCHMARK:   Python result at position {} with score {:.4}",
            py_pos, results[py_pos].score
        );
    }

    if let (Some(&best_rust_pos), Some(&py_pos)) =
        (rust_positions.first(), python_positions.first())
    {
        assert!(
            best_rust_pos < py_pos,
            "Rust results should rank higher than Python for 'programming language' query"
        );
    }
    assert!(
        results[0]
            .observation
            .title
            .starts_with("rust-observation-"),
        "Top result should be a Rust observation, got: {}",
        results[0].observation.title
    );
}
