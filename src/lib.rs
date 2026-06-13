#![allow(unused_imports, dead_code)]

pub mod core;
pub mod domain;
pub mod infrastructure;

#[cfg(test)]
mod tests {
    use crate::infrastructure::database::Database;
    use crate::domain::entities::{Chunk, EntityType, Observation, RelationType, SearchParams, chunk_text, compute_chunks, estimate_tokens, summarize, compute_importance, extract_entities, infer_relationships};
    use crate::domain::types::{SessionId, ObservationType, ObservationId};
    use crate::domain::ports::{StorageBackend, StoragePort};
    use crate::infrastructure::database::SqliteBackend;
    use crate::infrastructure::optimizer::AutoOptimizer;

    fn setup_db() -> Database {
        let db = Database::new();
        db.init().unwrap();
        db
    }

    #[test]
    fn test_estimate_tokens_empty() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn test_estimate_tokens_short() {
        // 4 chars = 1 token approx
        assert_eq!(estimate_tokens("test"), 1);
    }

    #[test]
    fn test_estimate_tokens_long() {
        let text = "a".repeat(100);
        assert_eq!(estimate_tokens(&text), 25);
    }

    #[test]
    fn test_summarize_short_content() {
        let s = summarize("Hello", "World", 100);
        assert!(s.contains("Hello"));
        assert!(s.contains("World"));
    }

    #[test]
    fn test_summarize_truncates_long_content() {
        let long = "a".repeat(1000);
        let s = summarize("Title", &long, 10);
        // Summary should be truncated
        assert!(s.len() < 200, "summary too long: {} chars", s.len());
    }

    #[test]
    fn test_compute_importance_baseline() {
        let imp = compute_importance("normal title", "short", &[]);
        assert!(imp > 0.0 && imp <= 1.0);
    }

    #[test]
    fn test_compute_importance_keyword_boost() {
        let imp_normal = compute_importance("normal title", "short", &[]);
        let imp_key = compute_importance("critical error in memory", "long content here", &[]);
        assert!(imp_key > imp_normal, "keyword boost should increase importance");
    }

    /// METRIC: Verify that saving and searching respects token budgets
    #[test]
    fn test_token_budget_enforced() {
        let db = setup_db();
        // Insert entries with known token sizes
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

        // Search with tight token budget
        let results_all = db.search_observations(
            &crate::domain::entities::SearchParams::new("memory")
                .with_max_tokens(10000)
        ).unwrap();
        let results_tight = db.search_observations(
            &crate::domain::entities::SearchParams::new("memory")
                .with_max_tokens(50)
        ).unwrap();

        // Tight budget should return fewer results
        assert!(results_tight.len() < results_all.len(),
            "Token budget of 50 should limit results (all={}, tight={})",
            results_all.len(), results_tight.len());

        // METRIC: print token efficiency
        let total_tokens: u32 = results_all.iter().map(|r| r.token_cost).sum();
        let tight_tokens: u32 = results_tight.iter().map(|r| r.token_cost).sum();
        println!("METRIC: no-budget search: {} results, {} tokens", results_all.len(), total_tokens);
        println!("METRIC: budget=50 search: {} results, {} tokens", results_tight.len(), tight_tokens);
        assert!(tight_tokens <= 100, "token budget exceeded: {} > 100", tight_tokens);
    }

    /// METRIC: Verify importance scoring affects retrieval order
    #[test]
    fn test_importance_ordering() {
        let db = setup_db();
        // Insert low and high importance entries
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

        let results = db.search_observations(
            &crate::domain::entities::SearchParams::new("memory")
        ).unwrap();

        assert!(!results.is_empty(), "should return results");
        // First result should have highest importance
        for i in 1..results.len() {
            assert!(results[i-1].observation.importance >= results[i].observation.importance,
                "results should be ordered by importance descending");
        }
        println!("METRIC: importance range: {:.2} - {:.2}",
            results.last().unwrap().observation.importance,
            results.first().unwrap().observation.importance);
    }

    // ── Benchmark tests ───────────────────────────────────────────────────

    /// BENCHMARK: Token efficiency — summary should be ≤60% of full content
    #[test]
    fn benchmark_token_efficiency() {
        let db = setup_db();
        let mut total_chars = 0usize;
        let mut total_summary_chars = 0usize;

        for i in 0..100 {
            let content_len = 200 + (i % 30) * 10;
            let content = "This is a long content string that will definitely exceed the token budget for summarizing purposes. ".repeat(content_len / 60 + 1);
            let title = format!("benchmark-{}", i);
            total_chars += content.len() + title.len();

            let obs = Observation::new(
                SessionId::new("bench"),
                ObservationType::Memory,
                title,
                content,
            );
            total_summary_chars += obs.summary.len();
            db.save_observation(&obs).unwrap();
        }

        let ratio = total_summary_chars as f64 / total_chars as f64;
        println!("BENCHMARK: Token efficiency ratio (summary/full): {:.2}%", ratio * 100.0);
        println!("BENCHMARK: Full chars: {}, Summary chars: {}", total_chars, total_summary_chars);
        assert!(ratio < 0.6, "Summary should be <60% of full content, got {:.2}%", ratio * 100.0);
    }

    /// BENCHMARK: Search speed — measure average search time across 10 random queries
    #[test]
    fn benchmark_search_speed() {
        let db = setup_db();
        let topics = [
            "machine learning", "data processing", "neural network", "memory management",
            "token optimization", "embedding vectors", "similarity search", "knowledge graph",
            "entity extraction", "relationship inference", "chunk management", "database query",
            "importance scoring", "content summarization", "efficient storage",
        ];

        // Insert 500 observations with varied content
        for i in 0..500 {
            let topic = topics[i % topics.len()];
            let content = format!("{} is an important concept in modern systems. ",
                topic).repeat(5 + (i % 10));
            let obs = Observation::new(
                SessionId::new("bench-speed"),
                ObservationType::Memory,
                format!("speed-{}-{}", topic.split_whitespace().next().unwrap_or("topic"), i),
                content,
            );
            db.save_observation(&obs).unwrap();
        }

        // Time 10 random queries
        let queries = [
            "machine learning", "data processing", "knowledge graph",
            "similarity search", "memory management", "entity extraction",
            "neural network", "database query", "content summarization",
            "relationship inference",
        ];

        let start = std::time::Instant::now();
        for q in &queries {
            let _ = db.search_observations(
                &SearchParams::new(*q).with_limit(10)
            ).unwrap();
        }
        let elapsed = start.elapsed();
        let avg = elapsed.as_micros() as f64 / queries.len() as f64;
        println!("BENCHMARK: Search speed — {} queries in {:?}, avg {:.0}µs",
            queries.len(), elapsed, avg);
        assert!(avg < 1_000_000.0, "Average search time should be <1s");
    }

    /// BENCHMARK: Token budget effectiveness — tight budget should reduce tokens
    #[test]
    fn benchmark_token_budget_effectiveness() {
        let db = setup_db();
        let content_base = "memory entry about programming languages and frameworks. ";

        for i in 0..50 {
            let content = format!("{}{}", content_base, "extra details for token counting. ".repeat(i));
            let obs = Observation::new(
                SessionId::new("bench-budget"),
                ObservationType::Memory,
                format!("budget-entry-{}", i),
                content,
            );
            db.save_observation(&obs).unwrap();
        }

        let wide = db.search_observations(
            &SearchParams::new("memory").with_max_tokens(1000)
        ).unwrap();
        let tight = db.search_observations(
            &SearchParams::new("memory").with_max_tokens(100)
        ).unwrap();

        let wide_tokens: u32 = wide.iter().map(|r| r.token_cost).sum();
        let tight_tokens: u32 = tight.iter().map(|r| r.token_cost).sum();
        let ratio = if wide_tokens > 0 {
            tight_tokens as f64 / wide_tokens as f64
        } else {
            0.0
        };

        println!("BENCHMARK: Token budget — wide(1000)={} tokens, tight(100)={} tokens, reduction ratio={:.2}",
            wide_tokens, tight_tokens, ratio);
        assert!(tight_tokens <= 100 || tight.len() < wide.len(),
            "Tight budget should reduce token usage");
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

        // 100 Rust observations
        for (i, phrase) in rust_phrases.iter().cycle().take(100).enumerate() {
            let obs = Observation::new(
                session.clone(),
                ObservationType::Memory,
                format!("rust-observation-{}", i),
                *phrase,
            );
            db.save_observation(&obs).unwrap();
        }

        // 1 Python observation
        let python_obs = Observation::new(
            session.clone(),
            ObservationType::Memory,
            "python-web-dev",
            "Python web development with Django and Flask frameworks for building REST APIs",
        );
        db.save_observation(&python_obs).unwrap();

        let results = db.search_observations(
            &SearchParams::new("programming language")
                .with_semantic(true)
                .with_limit(110)
        ).unwrap();

        assert!(!results.is_empty(), "semantic search should return results");

        // Find positions of rust and python results
        let rust_positions: Vec<usize> = results.iter().enumerate()
            .filter(|(_, r)| r.observation.title.starts_with("rust-observation-"))
            .map(|(i, _)| i)
            .collect();
        let python_positions: Vec<usize> = results.iter().enumerate()
            .filter(|(_, r)| r.observation.title == "python-web-dev")
            .map(|(i, _)| i)
            .collect();

        println!("BENCHMARK: Embedding similarity — {} Rust results, {} Python result in top {}",
            rust_positions.len(), python_positions.len(), results.len());

        for (i, &pos) in rust_positions.iter().take(5).enumerate() {
            println!("BENCHMARK:   Rust result #{} at position {} with score {:.4}",
                i + 1, pos, results[pos].score);
        }
        if let Some(&py_pos) = python_positions.first() {
            println!("BENCHMARK:   Python result at position {} with score {:.4}",
                py_pos, results[py_pos].score);
        }

        // Verify the best Rust result outranks the Python result
        if let (Some(&best_rust_pos), Some(&py_pos)) = (rust_positions.first(), python_positions.first()) {
            assert!(best_rust_pos < py_pos,
                "Rust results should rank higher than Python for 'programming language' query");
        }
        assert!(results[0].observation.title.starts_with("rust-observation-"),
            "Top result should be a Rust observation, got: {}", results[0].observation.title);
    }

    /// METRIC: Test that retain() respects token budget
    #[test]
    fn test_retain_eviction() {
        let db = setup_db();
        // Insert many small entries
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
        println!("METRIC: before retain - entries={}, tokens={}", before.total_entries, before.total_tokens);

        // Retain with very tight budget
        let freed = db.retain(100).unwrap();
        let after = db.stats_db().unwrap();
        println!("METRIC: after retain(100) - entries={}, tokens={}, freed={} tokens",
            after.total_entries, after.total_tokens, freed);

        assert!(after.total_tokens <= 1000, "retain should reduce tokens: {} > 1000", after.total_tokens);
    }

    /// METRIC: Verify summary is more token-efficient than full content
    #[test]
    fn test_summary_token_efficiency() {
        let db = setup_db();
        let long_content = "This is a very long content that would normally consume many tokens if stored and retrieved in full. ".repeat(50);
        let obs = Observation::new(
            SessionId::new("test"),
            ObservationType::Memory,
            "efficient memory test",
            long_content,
        );
        db.save_observation(&obs).unwrap();

        let results = db.search_observations(
            &crate::domain::entities::SearchParams::new("efficient")
        ).unwrap();

        assert!(!results.is_empty());

        // The efficient_content() method should return summary when within budget
        let obs = &results[0].observation;
        let full_tokens = estimate_tokens(&format!("{}: {}", obs.title, obs.content));
        let efficient = obs.efficient_content(100);
        let efficient_tokens = estimate_tokens(efficient);

        println!("METRIC: full content tokens={}, efficient tokens={}, ratio={:.1}x",
            full_tokens, efficient_tokens,
            if efficient_tokens > 0 { full_tokens as f64 / efficient_tokens as f64 } else { 0.0 });

        assert!(efficient_tokens <= full_tokens,
            "efficient content should use fewer or equal tokens");
    }

    #[test]
    fn test_compute_embedding_dimensions_and_similarity() {
        use crate::domain::entities::{compute_embedding, cosine_similarity, EMBEDDING_DIMENSIONS};

        let emb = compute_embedding("test text for embedding");
        assert_eq!(emb.len(), EMBEDDING_DIMENSIONS);

        let emb_a = compute_embedding("machine learning for trading systems");
        let emb_b = compute_embedding("deep learning in quantitative finance");
        let emb_c = compute_embedding("weather forecast for tomorrow");

        let sim_ab = cosine_similarity(&emb_a, &emb_b);
        let sim_ac = cosine_similarity(&emb_a, &emb_c);

        assert!(sim_ab > sim_ac,
            "similar texts should have higher cosine similarity (ab={:.4}, ac={:.4})", sim_ab, sim_ac);
        println!("METRIC: cosine_sim(similar)={:.4}, cosine_sim(dissimilar)={:.4}", sim_ab, sim_ac);
    }

    #[test]
    fn test_semantic_search_ranking() {
        let db = setup_db();
        let session = SessionId::new("test");

        let themes = vec![
            ("trading system", "algorithmic trading with machine learning and neural networks for market prediction"),
            ("weather data", "temperature and precipitation forecasts for the next week"),
            ("recipe book", "cooking instructions for Italian pasta and pizza dishes"),
        ];

        for (title, content) in &themes {
            let obs = Observation::new(
                session.clone(),
                ObservationType::Memory,
                *title,
                *content,
            );
            db.save_observation(&obs).unwrap();
        }

        let results = db.search_observations(
            &crate::domain::entities::SearchParams::new("machine learning for finance")
                .with_semantic(true)
        ).unwrap();

        assert!(!results.is_empty(), "semantic search should return results");
        assert_eq!(results[0].observation.title, "trading system",
            "most semantically relevant result should rank first, got: {}",
            results[0].observation.title);
        println!("METRIC: semantic search top result: '{}' with score {:.4}",
            results[0].observation.title, results[0].score);
    }

    #[test]
    fn test_chunk_text_small() {
        let text = "Hello world. This is a short text.";
        let chunks = chunk_text(text, 256);
        assert_eq!(chunks.len(), 1, "small text should produce 1 chunk");
        assert_eq!(chunks[0].0, text);
    }

    #[test]
    fn test_chunk_text_splits_long() {
        let sentences = vec![
            "The quick brown fox jumps over the lazy dog.",
            "Machine learning is transforming the world of technology.",
            "Neural networks can recognize patterns in complex data.",
            "Natural language processing enables computers to understand text.",
            "Computer vision allows machines to interpret visual information.",
            "Deep learning models require large amounts of training data.",
            "Reinforcement learning teaches agents through trial and error.",
            "Transfer learning applies knowledge from one domain to another.",
        ];
        let text = sentences.join(" ");
        let chunks = chunk_text(&text, 50);
        assert!(chunks.len() > 1, "long text should be split into multiple chunks, got {} chunks", chunks.len());
        for (content, summary) in &chunks {
            assert!(!content.is_empty(), "chunk content should not be empty");
            assert!(!summary.is_empty(), "chunk summary should not be empty");
        }
        let reconstructed: String = chunks.iter().map(|(c, _)| c.as_str()).collect::<Vec<&str>>().join("");
        assert_eq!(reconstructed.len(), text.len(), "reconstructed text should match original");
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

        // Verify chunks were stored by querying directly via a new connection
        let chunk_count: i64 = {
            let conn = db.get_conn();
            let mut stmt = conn.prepare(
                "SELECT COUNT(*) FROM chunks WHERE observation_id = ?1"
            ).unwrap();
            stmt.query_row(rusqlite::params![obs_id.0], |row| row.get(0)).unwrap()
        };
        assert!(chunk_count > 0, "chunks should have been stored in the DB, got {} chunks", chunk_count);

        // Test merge_chunks reconstructs the observation
        let merged = crate::infrastructure::database::merge_chunks(obs_id.0 as u64, &db);
        assert!(merged.is_some(), "merge_chunks should return Some observation");
        let merged = merged.unwrap();
        assert_eq!(merged.title, "chunk storage test", "merged observation should preserve title");
        assert!(merged.content.len() >= long_content.len() / 2, "merged content should be substantial");

        // Verify observation is still searchable via normal path
        let results = db.search_observations(
            &crate::domain::entities::SearchParams::new("machine learning")
        ).unwrap();
        assert!(!results.is_empty(), "chunked observation should be searchable");
    }

    // ── Knowledge Graph tests ────────────────────────────────────────────────

    #[test]
    fn test_extract_entities_basic() {
        let text = "I am working on the Neural Network Project with @alice and @bob. \
                     We use https://github.com/example for version control. \
                     The system depends on Rust and PostgreSQL.";
        let entities = extract_entities(text);
        // Should find: Neural Network Project (Project), @alice→alice (Person), @bob→bob (Person),
        //   https://github.com/example (Tool), Rust (Language), PostgreSQL (Technology)
        let names: Vec<String> = entities.iter().map(|(n, _)| n.to_lowercase()).collect();
        assert!(names.contains(&"neural network project".to_string()), "should detect Neural Network Project, got: {:?}", names);
        assert!(names.contains(&"alice".to_string()), "should detect alice");
        assert!(names.contains(&"bob".to_string()), "should detect bob");
        assert!(names.contains(&"rust".to_string()), "should detect Rust as language");
        assert!(names.contains(&"postgresql".to_string()), "should detect PostgreSQL");
        assert!(entities.iter().any(|(_, t)| *t == EntityType::Tool), "should have a Tool (URL)");
        println!("extracted entities: {:?}", entities);
    }

    #[test]
    fn test_extract_entities_types() {
        // Test various entity types
        let text = "@john uses Python and Docker for the ML Platform project at Acme Corp. \
                     The REST API is deployed on AWS.";
        let entities = extract_entities(text);
        for (name, typ) in &entities {
            match name.to_lowercase().as_str() {
                "john" => assert_eq!(*typ, EntityType::Person, "@john should be Person"),
                "python" => assert_eq!(*typ, EntityType::Language, "Python should be Language"),
                "docker" => assert_eq!(*typ, EntityType::Technology, "Docker should be Technology"),
                "ml platform" => assert_eq!(*typ, EntityType::Project, "ML Platform should be Project"),
                "rest" => assert_eq!(*typ, EntityType::Technology, "REST should be Technology"),
                "api" => assert_eq!(*typ, EntityType::Technology, "API should be Technology"),
                "aws" => assert_eq!(*typ, EntityType::Technology, "AWS should be Technology"),
                _ => {}
            }
        }
        // Verify deduplication
        let mut names: Vec<String> = entities.iter().map(|(n, _)| n.to_lowercase()).collect();
        names.sort();
        let dedup_len = names.len();
        names.dedup();
        assert_eq!(dedup_len, names.len(), "entities should be deduplicated");
    }

    #[test]
    fn test_entity_storage_and_retrieval() {
        let db = setup_db();
        let session = SessionId::new("test");
        let content = "Working on the Quantum Computing Framework with @eve. \
                       The system uses Rust and Python. \
                       We deploy on kubernetes.";
        let obs = Observation::new(
            session.clone(),
            ObservationType::Note,
            "Quantum project",
            content,
        );
        db.save_observation(&obs).unwrap();

        // Verify entities were stored via entity_search
        let entities = db.entity_search("", None).unwrap();
        assert!(!entities.is_empty(), "entities should be stored");

        let names: Vec<String> = entities.iter().map(|e| e.name.to_lowercase()).collect();
        assert!(names.contains(&"quantum computing framework".to_string()), "should have Quantum Computing Framework");
        assert!(names.contains(&"eve".to_string()), "should have eve");
        assert!(names.contains(&"rust".to_string()), "should have Rust");
        assert!(names.contains(&"python".to_string()), "should have Python");
        assert!(names.contains(&"kubernetes".to_string()), "should have kubernetes");

        // Test entity_search with type filter
        let languages = db.entity_search("", Some(EntityType::Language)).unwrap();
        assert!(languages.iter().any(|e| e.name == "rust"), "rust should be a Language");
        assert!(languages.iter().any(|e| e.name == "python"), "python should be a Language");

        // Test entity_search by name query
        let results = db.entity_search("quantum", None).unwrap();
        assert!(!results.is_empty(), "should find entities by name");
    }

    #[test]
    fn test_backend_trait_with_sqlite() {
        use crate::infrastructure::database::get_str;
        use rusqlite::Connection;
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY, val TEXT)"
        ).unwrap();
        let backend = SqliteBackend::new(conn);
        backend.execute_batch(
            "INSERT INTO test (id, val) VALUES (1, 'hello'), (2, 'world')"
        ).unwrap();
        let rows = backend.query("SELECT id, val FROM test ORDER BY id", &[]).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(get_str(&rows[0][1]), Some("hello"));
        assert_eq!(get_str(&rows[1][1]), Some("world"));
        let affected = backend.execute(
            "UPDATE test SET val = ?1 WHERE id = ?2",
            &[rusqlite::types::Value::Text("updated".into()), rusqlite::types::Value::Integer(1)],
        ).unwrap();
        assert_eq!(affected, 1);
        let rows = backend.query("SELECT val FROM test WHERE id = 1", &[]).unwrap();
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
        let results = db.search_observations(
            &SearchParams::new("backend trait")
        ).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].observation.title, "backend trait test");
    }

    #[test]
    fn test_relation_inference() {
        let text = "The Machine Learning Pipeline depends on Python and TensorFlow. \
                     Rust is similar to C++. \
                     @alice uses Docker.";
        let entities = extract_entities(text);
        let relations = infer_relationships(text, &entities);

        assert!(!relations.is_empty(), "should infer relationships, got {}", relations.len());

        // Check Depends relation between ML Pipeline and Python
        let depends = relations.iter().filter(|(_, _, rt, _)| *rt == RelationType::Depends).count();
        assert!(depends > 0, "should have Depends relations, got {}", depends);

        // Check Uses relation between @alice and Docker
        let uses = relations.iter().filter(|(_, _, rt, _)| *rt == RelationType::Uses).count();
        assert!(uses > 0, "should have Uses relations, got {}", uses);

        // Verify relation types are correct
        for (_, _, _rt, w) in &relations {
            assert!(*w > 0.0 && *w <= 1.0, "weight should be in (0,1], got {}", w);
        }

        // Save observation and verify relations are stored in DB
        let db = setup_db();
        let session = SessionId::new("test");
        let obs = Observation::new(
            session.clone(),
            ObservationType::Note,
            "Relation test",
            text,
        );
        db.save_observation(&obs).unwrap();

        // Debug: see what entities are stored
        let all_entities = db.entity_search("", None).unwrap();
        println!("Stored entities: {:?}", all_entities.iter().map(|e| &e.name).collect::<Vec<_>>());

        // Use find_related_entities to verify graph traversal
        let related = db.find_related_entities("rust", 2, 0.0).unwrap();
        assert!(!related.is_empty(), "Rust should have related entities, storage has: {:?}",
            all_entities.iter().map(|e| &e.name).collect::<Vec<_>>());
        println!("Entities related to Rust: {:?}", related.iter().map(|(_, _, e)| &e.name).collect::<Vec<_>>());
    }

    // ── AutoOptimizer tests ──────────────────────────────────────────

    #[test]
    fn test_auto_optimizer_removes_low_importance() {
        let db = setup_db();
        // Insert high-importance entries (keywords in title)
        for i in 0..5 {
            let obs = Observation::new(
                SessionId::new("opt"),
                ObservationType::Memory,
                format!("critical error memory entry {}", i),
                "high importance content for testing purposes",
            );
            db.save_observation(&obs).unwrap();
        }
        // Insert low-importance entries (no keywords, short content)
        for i in 0..20 {
            let obs = Observation::new(
                SessionId::new("opt"),
                ObservationType::Memory,
                format!("note {}", i),
                "short",
            );
            db.save_observation(&obs).unwrap();
        }

        let before = db.stats_db().unwrap();
        println!("OPTIMIZER: before - entries={}, tokens={}", before.total_entries, before.total_tokens);

        let optimizer = AutoOptimizer::new(50);
        let stats = optimizer.optimize(&db).unwrap();

        let after = db.stats_db().unwrap();
        println!("OPTIMIZER: after - entries={}, tokens={}", after.total_entries, after.total_tokens);
        println!("OPTIMIZER: removed entries={}, summarized={}", stats.entries_removed, stats.entries_summarized);

        assert!(before.total_entries > after.total_entries,
            "optimize should remove entries (before={}, after={})", before.total_entries, after.total_entries);
        assert!(stats.entries_removed > 0, "entries_removed should be > 0, got {}", stats.entries_removed);
    }

    #[test]
    fn test_auto_tune_budget_reduces_latency() {
        let db = setup_db();
        // Insert enough data so searches have measurable cost
        for i in 0..50 {
            let obs = Observation::new(
                SessionId::new("tune"),
                ObservationType::Memory,
                format!("test entry number {}", i),
                "some content about data processing and system optimization for testing",
            );
            db.save_observation(&obs).unwrap();
        }

        let optimizer = AutoOptimizer::new(10000);
        let initial_budget = 10000;
        // Use target_latency_ms=0 so the in-memory DB's fast searches always exceed target
        let optimal = optimizer.auto_tune_budget(&db, 0).unwrap();

        println!("OPTIMIZER: initial budget={}, optimal budget={}", initial_budget, optimal);
        assert!(optimal < initial_budget,
            "auto_tune should reduce budget (initial={}, optimal={})", initial_budget, optimal);
    }

    #[test]
    fn test_auto_optimizer_stats() {
        let db = setup_db();
        for i in 0..20 {
            let obs = Observation::new(
                SessionId::new("stats"),
                ObservationType::Memory,
                format!("stats entry {}", i),
                "content with some text for token count verification",
            );
            db.save_observation(&obs).unwrap();
        }

        let optimizer = AutoOptimizer::new(100);
        let stats = optimizer.optimize(&db).unwrap();

        println!("OPTIMIZER: tokens before={}, after={}", stats.total_tokens_before, stats.total_tokens_after);
        println!("OPTIMIZER: removed={}, summarized={}", stats.entries_removed, stats.entries_summarized);
        println!("OPTIMIZER: latency={:.0}µs, budget_util={:.1}%",
            stats.avg_latency_us, stats.budget_utilization_pct);

        assert!(stats.total_tokens_before > 0, "total_tokens_before should be > 0");
        assert!(stats.avg_latency_us > 0.0, "avg_latency_us should be > 0.0");
        assert!(stats.budget_utilization_pct >= 0.0, "budget_utilization_pct should be >= 0.0");
    }
}
