use synapsis_core::domain::entities::{
    chunk_text, compute_embedding, compute_importance, cosine_similarity, estimate_tokens,
    extract_entities, infer_relationships, summarize, EntityType, Observation, RelationType,
    SearchParams, EMBEDDING_DIMENSIONS,
};
use synapsis_core::domain::types::{ObservationType, SessionId};
use synapsis_core::infrastructure::database::Database;

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
    assert!(
        imp_key > imp_normal,
        "keyword boost should increase importance"
    );
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
    assert!(
        chunks.len() > 1,
        "long text should be split into multiple chunks, got {} chunks",
        chunks.len()
    );
    for (content, summary) in &chunks {
        assert!(!content.is_empty(), "chunk content should not be empty");
        assert!(!summary.is_empty(), "chunk summary should not be empty");
    }
    let reconstructed: String = chunks
        .iter()
        .map(|(c, _)| c.as_str())
        .collect::<Vec<&str>>()
        .join("");
    assert_eq!(
        reconstructed.len(),
        text.len(),
        "reconstructed text should match original"
    );
}

#[test]
fn test_compute_embedding_dimensions_and_similarity() {
    let emb = compute_embedding("test text for embedding");
    assert_eq!(emb.len(), EMBEDDING_DIMENSIONS);

    let emb_a = compute_embedding("machine learning for trading systems");
    let emb_b = compute_embedding("deep learning in quantitative finance");
    let emb_c = compute_embedding("weather forecast for tomorrow");

    let sim_ab = cosine_similarity(&emb_a, &emb_b);
    let sim_ac = cosine_similarity(&emb_a, &emb_c);

    assert!(
        sim_ab > sim_ac,
        "similar texts should have higher cosine similarity (ab={:.4}, ac={:.4})",
        sim_ab,
        sim_ac
    );
    println!(
        "METRIC: cosine_sim(similar)={:.4}, cosine_sim(dissimilar)={:.4}",
        sim_ab, sim_ac
    );
}

#[test]
fn test_extract_entities_basic() {
    let text = "I am working on the Neural Network Project with @alice and @bob. \
                 We use https://github.com/example for version control. \
                 The system depends on Rust and PostgreSQL.";
    let entities = extract_entities(text);
    let names: Vec<String> = entities.iter().map(|(n, _)| n.to_lowercase()).collect();
    assert!(
        names.contains(&"neural network project".to_string()),
        "should detect Neural Network Project, got: {:?}",
        names
    );
    assert!(names.contains(&"alice".to_string()), "should detect alice");
    assert!(names.contains(&"bob".to_string()), "should detect bob");
    assert!(
        names.contains(&"rust".to_string()),
        "should detect Rust as language"
    );
    assert!(
        names.contains(&"postgresql".to_string()),
        "should detect PostgreSQL"
    );
    assert!(
        entities.iter().any(|(_, t)| *t == EntityType::Tool),
        "should have a Tool (URL)"
    );
    println!("extracted entities: {:?}", entities);
}

#[test]
fn test_extract_entities_types() {
    let text = "@john uses Python and Docker for the ML Platform project at Acme Corp. \
                 The REST API is deployed on AWS.";
    let entities = extract_entities(text);
    for (name, typ) in &entities {
        match name.to_lowercase().as_str() {
            "john" => assert_eq!(*typ, EntityType::Person, "@john should be Person"),
            "python" => assert_eq!(*typ, EntityType::Language, "Python should be Language"),
            "docker" => assert_eq!(*typ, EntityType::Technology, "Docker should be Technology"),
            "ml platform" => {
                assert_eq!(*typ, EntityType::Project, "ML Platform should be Project")
            }
            "rest" => assert_eq!(*typ, EntityType::Technology, "REST should be Technology"),
            "api" => assert_eq!(*typ, EntityType::Technology, "API should be Technology"),
            "aws" => assert_eq!(*typ, EntityType::Technology, "AWS should be Technology"),
            _ => {}
        }
    }
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

    let entities = db.entity_search("", None).unwrap();
    assert!(!entities.is_empty(), "entities should be stored");

    let names: Vec<String> = entities.iter().map(|e| e.name.to_lowercase()).collect();
    assert!(
        names.contains(&"quantum computing framework".to_string()),
        "should have Quantum Computing Framework"
    );
    assert!(names.contains(&"eve".to_string()), "should have eve");
    assert!(names.contains(&"rust".to_string()), "should have Rust");
    assert!(names.contains(&"python".to_string()), "should have Python");
    assert!(
        names.contains(&"kubernetes".to_string()),
        "should have kubernetes"
    );

    let languages = db.entity_search("", Some(EntityType::Language)).unwrap();
    assert!(
        languages.iter().any(|e| e.name == "rust"),
        "rust should be a Language"
    );
    assert!(
        languages.iter().any(|e| e.name == "python"),
        "python should be a Language"
    );

    let results = db.entity_search("quantum", None).unwrap();
    assert!(!results.is_empty(), "should find entities by name");
}

#[test]
fn test_relation_inference() {
    let text = "The Machine Learning Pipeline depends on Python and TensorFlow. \
                 Rust is similar to C++. \
                 @alice uses Docker.";
    let entities = extract_entities(text);
    let relations = infer_relationships(text, &entities);

    assert!(
        !relations.is_empty(),
        "should infer relationships, got {}",
        relations.len()
    );

    let depends = relations
        .iter()
        .filter(|(_, _, rt, _)| *rt == RelationType::Depends)
        .count();
    assert!(
        depends > 0,
        "should have Depends relations, got {}",
        depends
    );

    let uses = relations
        .iter()
        .filter(|(_, _, rt, _)| *rt == RelationType::Uses)
        .count();
    assert!(uses > 0, "should have Uses relations, got {}", uses);

    for (_, _, _rt, w) in &relations {
        assert!(
            *w > 0.0 && *w <= 1.0,
            "weight should be in (0,1], got {}",
            w
        );
    }

    let db = setup_db();
    let session = SessionId::new("test");
    let obs = Observation::new(session.clone(), ObservationType::Note, "Relation test", text);
    db.save_observation(&obs).unwrap();

    let all_entities = db.entity_search("", None).unwrap();
    println!(
        "Stored entities: {:?}",
        all_entities.iter().map(|e| &e.name).collect::<Vec<_>>()
    );

    let related = db.find_related_entities("rust", 2, 0.0).unwrap();
    assert!(
        !related.is_empty(),
        "Rust should have related entities, storage has: {:?}",
        all_entities.iter().map(|e| &e.name).collect::<Vec<_>>()
    );
    println!(
        "Entities related to Rust: {:?}",
        related.iter().map(|(_, _, e)| &e.name).collect::<Vec<_>>()
    );
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
        let obs = Observation::new(session.clone(), ObservationType::Memory, *title, *content);
        db.save_observation(&obs).unwrap();
    }

    let results = db
        .search_observations(
            &SearchParams::new("machine learning for finance")
                .with_semantic(true),
        )
        .unwrap();

    assert!(!results.is_empty(), "semantic search should return results");
    assert_eq!(
        results[0].observation.title, "trading system",
        "most semantically relevant result should rank first, got: {}",
        results[0].observation.title
    );
    println!(
        "METRIC: semantic search top result: '{}' with score {:.4}",
        results[0].observation.title, results[0].score
    );
}

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

    let results = db
        .search_observations(&SearchParams::new("efficient"))
        .unwrap();

    assert!(!results.is_empty());

    let obs = &results[0].observation;
    let full_tokens = estimate_tokens(&format!("{}: {}", obs.title, obs.content));
    let efficient = obs.efficient_content(100);
    let efficient_tokens = estimate_tokens(efficient);

    println!(
        "METRIC: full content tokens={}, efficient tokens={}, ratio={:.1}x",
        full_tokens,
        efficient_tokens,
        if efficient_tokens > 0 {
            full_tokens as f64 / efficient_tokens as f64
        } else {
            0.0
        }
    );

    assert!(
        efficient_tokens <= full_tokens,
        "efficient content should use fewer or equal tokens"
    );
}

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
    println!(
        "BENCHMARK: Token efficiency ratio (summary/full): {:.2}%",
        ratio * 100.0
    );
    println!(
        "BENCHMARK: Full chars: {}, Summary chars: {}",
        total_chars, total_summary_chars
    );
    assert!(
        ratio < 0.6,
        "Summary should be <60% of full content, got {:.2}%",
        ratio * 100.0
    );
}
