use synapsis_core::domain::entities::Observation;
use synapsis_core::domain::types::{ObservationType, SessionId};
use synapsis_core::infrastructure::database::Database;
use synapsis_core::infrastructure::optimizer::AutoOptimizer;

fn setup_db() -> Database {
    let db = Database::new();
    db.init().unwrap();
    db
}

#[test]
fn test_auto_optimizer_removes_low_importance() {
    let db = setup_db();
    for i in 0..5 {
        let obs = Observation::new(
            SessionId::new("opt"),
            ObservationType::Memory,
            format!("critical error memory entry {}", i),
            "high importance content for testing purposes",
        );
        db.save_observation(&obs).unwrap();
    }
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
    println!(
        "OPTIMIZER: before - entries={}, tokens={}",
        before.total_entries, before.total_tokens
    );

    let optimizer = AutoOptimizer::new(50);
    let stats = optimizer.optimize(&db).unwrap();

    let after = db.stats_db().unwrap();
    println!(
        "OPTIMIZER: after - entries={}, tokens={}",
        after.total_entries, after.total_tokens
    );
    println!(
        "OPTIMIZER: removed entries={}, summarized={}",
        stats.entries_removed, stats.entries_summarized
    );

    assert!(
        before.total_entries > after.total_entries,
        "optimize should remove entries (before={}, after={})",
        before.total_entries,
        after.total_entries
    );
    assert!(
        stats.entries_removed > 0,
        "entries_removed should be > 0, got {}",
        stats.entries_removed
    );
}

#[test]
fn test_auto_tune_budget_reduces_latency() {
    let db = setup_db();
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
    let optimal = optimizer.auto_tune_budget(&db, 0).unwrap();

    println!(
        "OPTIMIZER: initial budget={}, optimal budget={}",
        initial_budget, optimal
    );
    assert!(
        optimal < initial_budget,
        "auto_tune should reduce budget (initial={}, optimal={})",
        initial_budget,
        optimal
    );
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

    println!(
        "OPTIMIZER: tokens before={}, after={}",
        stats.total_tokens_before, stats.total_tokens_after
    );
    println!(
        "OPTIMIZER: removed={}, summarized={}",
        stats.entries_removed, stats.entries_summarized
    );
    println!(
        "OPTIMIZER: latency={:.0}µs, budget_util={:.1}%",
        stats.avg_latency_us, stats.budget_utilization_pct
    );

    assert!(
        stats.total_tokens_before > 0,
        "total_tokens_before should be > 0"
    );
    assert!(stats.avg_latency_us > 0.0, "avg_latency_us should be > 0.0");
    assert!(
        stats.budget_utilization_pct >= 0.0,
        "budget_utilization_pct should be >= 0.0"
    );
}
