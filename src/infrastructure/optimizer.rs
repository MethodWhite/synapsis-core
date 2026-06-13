use std::time::Instant;
use crate::domain::entities::{SearchParams, summarize};
use crate::infrastructure::database::Database;

#[derive(Debug, Clone)]
pub struct OptimizationStats {
    pub total_tokens_before: u64,
    pub total_tokens_after: u64,
    pub entries_removed: u64,
    pub entries_summarized: u64,
    pub avg_latency_us: f64,
    pub budget_utilization_pct: f64,
}

pub struct AutoOptimizer {
    token_budget: u64,
    min_importance: f32,
    target_latency_ms: u64,
    stats: OptimizationStats,
}

impl AutoOptimizer {
    pub fn new(token_budget: u64) -> Self {
        Self {
            token_budget,
            min_importance: 0.0,
            target_latency_ms: 100,
            stats: OptimizationStats {
                total_tokens_before: 0,
                total_tokens_after: 0,
                entries_removed: 0,
                entries_summarized: 0,
                avg_latency_us: 0.0,
                budget_utilization_pct: 0.0,
            },
        }
    }

    pub fn optimize(&self, db: &Database) -> Result<OptimizationStats, String> {
        let start = Instant::now();

        let before = db.stats_db()?;
        let total_tokens_before = before.total_tokens;
        let entries_before = before.total_entries;

        let _freed = db.retain(self.token_budget)?;

        let after_evict = db.stats_db()?;
        let entries_removed = entries_before.saturating_sub(after_evict.total_entries);

        let all_obs = db.get_all_observations()?;
        let threshold = (self.token_budget / 100).max(1) as u32;
        let mut entries_summarized = 0u64;

        for obs in &all_obs {
            if obs.token_count > threshold {
                let new_summary = summarize(&obs.title, &obs.content, (self.token_budget / 200).max(10) as usize);
                db.update_summary(obs.id, &new_summary)?;
                entries_summarized += 1;
            }
        }

        let after = db.stats_db()?;
        let total_tokens_after = after.total_tokens;

        let elapsed_us = start.elapsed().as_micros() as f64;
        let budget_utilization = if self.token_budget > 0 {
            (total_tokens_after as f64 / self.token_budget as f64) * 100.0
        } else {
            0.0
        };

        Ok(OptimizationStats {
            total_tokens_before,
            total_tokens_after,
            entries_removed,
            entries_summarized,
            avg_latency_us: elapsed_us,
            budget_utilization_pct: budget_utilization.min(100.0),
        })
    }

    pub fn auto_tune_budget(&self, db: &Database, target_latency_ms: u64) -> Result<u64, String> {
        let mut budget = self.token_budget;
        let queries = [
            "memory", "data", "system", "learning", "network",
            "algorithm", "processing", "optimization", "storage", "query",
        ];

        for _ in 0..10 {
            let start = Instant::now();
            for q in &queries {
                let params = SearchParams::new(*q)
                    .with_max_tokens(budget as u32)
                    .with_min_importance(self.min_importance);
                db.search_observations(&params)?;
            }
            let avg_latency_us = start.elapsed().as_micros() as f64 / queries.len() as f64;

            if (avg_latency_us / 1000.0) <= target_latency_ms as f64 {
                break;
            }
            budget = (budget as f64 * 0.8) as u64;
            if budget < 10 {
                break;
            }
        }

        Ok(budget)
    }

    pub fn schedule(&self, db: &Database, _interval_minutes: u64) -> Result<OptimizationStats, String> {
        self.optimize(db)
    }
}
