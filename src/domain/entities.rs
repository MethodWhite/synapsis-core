use serde::{Deserialize, Serialize};
use crate::domain::types::{ObservationId, ObservationType, SessionId, Timestamp};

/// Token-efficient memory entry with importance scoring and automatic summarization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: ObservationId,
    pub session_id: String,
    pub title: String,
    pub summary: String,         // compressed/summarized version for token efficiency
    pub content: String,          // full content (can be empty if summary is sufficient)
    pub project: Option<String>,
    pub tags: Vec<String>,
    pub created_at: Timestamp,
    pub observation_type: ObservationType,
    pub importance: f32,          // 0.0-1.0 priority score
    pub token_count: u32,         // estimated tokens in content+title
    pub access_count: u64,        // times retrieved
}

impl Observation {
    pub fn new(session_id: SessionId, obs_type: ObservationType, title: impl Into<String>, content: impl Into<String>) -> Self {
        let title = title.into();
        let content = content.into();
        let token_count = estimate_tokens(&title) + estimate_tokens(&content);
        let importance = compute_importance(&title, &content, &[]);
        Self {
            id: ObservationId(0i64),
            session_id: session_id.instance_uuid,
            summary: summarize(&title, &content, 50),  // 50 token summary
            content,
            title,
            project: None,
            tags: vec![],
            created_at: Timestamp::now(),
            observation_type: obs_type,
            importance,
            token_count: token_count as u32,
            access_count: 0,
        }
    }

    /// Returns the most token-efficient representation.
    /// If summary is sufficient, prefer it over full content.
    pub fn efficient_content(&self, max_tokens: u32) -> &str {
        let summary_tokens = estimate_tokens(&self.summary) as u32;
        if summary_tokens <= max_tokens && !self.summary.is_empty() {
            &self.summary
        } else {
            &self.content
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: u64,
    pub observation_id: u64,
    pub content: String,
    pub summary: String,
    pub token_count: u32,
    pub embedding: Vec<f32>,
    pub seq: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchParams {
    pub query: String,
    pub project: Option<String>,
    pub limit: Option<i64>,
    pub max_tokens: Option<u32>,    // token budget for results
    pub min_importance: Option<f32>, // minimum importance filter
    pub obs_type: Option<ObservationType>,
    pub scope: Option<String>,
    pub use_semantic: bool,
}

impl SearchParams {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            project: None,
            limit: Some(10),
            max_tokens: Some(1024),
            min_importance: Some(0.0),
            obs_type: None,
            scope: None,
            use_semantic: false,
        }
    }
    pub fn with_limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit as i64);
        self
    }
    pub fn with_max_tokens(mut self, max: u32) -> Self {
        self.max_tokens = Some(max);
        self
    }
    pub fn with_min_importance(mut self, min: f32) -> Self {
        self.min_importance = Some(min);
        self
    }
    pub fn with_semantic(mut self, use_semantic: bool) -> Self {
        self.use_semantic = use_semantic;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry { pub observation: Observation }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub observation: Observation,
    pub score: f64,
    pub token_cost: u32,
    pub matched_chunk_ids: Vec<u64>,
}

impl std::ops::Deref for SearchResult {
    type Target = Observation;
    fn deref(&self) -> &Observation { &self.observation }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub started_at: Timestamp,
    pub ended_at: Option<Timestamp>,
    pub observation_count: i64,
    pub total_tokens: u64,     // total tokens stored
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub agent_type: String,
    pub observation_count: i64,
    pub last_active: Timestamp,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub session_id: SessionId,
    pub content: String,
    pub importance: f32,
    pub token_count: u32,
}

impl MemoryEntry {
    pub fn new(session_id: SessionId, content: String) -> Self {
        let tc = estimate_tokens(&content) as u32;
        Self { session_id, content, importance: 0.5, token_count: tc }
    }
}

// ── Knowledge Graph types ────────────────────────────────────────────

pub type EntityId = u64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    Person, Organization, Location, Concept, Technology,
    Project, Language, Tool, Protocol, Topic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub name: String,
    pub entity_type: EntityType,
    pub aliases: Vec<String>,
    pub embedding: Vec<f32>,
    pub mention_count: u64,
    pub first_seen: Timestamp,
    pub last_seen: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationType {
    Mentions, Depends, Implements, Extends,
    Uses, PartOf, Related, Creates, Modifies,
    Similar, Opposite, Specializes, Example,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: u64,
    pub source_id: EntityId,
    pub target_id: EntityId,
    pub relation_type: RelationType,
    pub weight: f32,
    pub observation_id: u64,
    pub created_at: Timestamp,
}

/// Extract entities from text using pattern matching.
/// Returns deduplicated (name, type) pairs.
pub fn extract_entities(text: &str) -> Vec<(String, EntityType)> {
    use std::collections::HashSet;
    let mut entities: Vec<(String, EntityType)> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    let add = |entities: &mut Vec<(String, EntityType)>, seen: &mut HashSet<String>, name: String, typ: EntityType| {
        let key = name.to_lowercase();
        if seen.insert(key) {
            entities.push((name, typ));
        }
    };

    // Language keywords
    let lang_set: HashSet<&str> = [
        "rust", "python", "javascript", "typescript", "golang", "go",
        "java", "c++", "c#", "csharp", "ruby", "php", "swift",
        "kotlin", "scala", "perl", "haskell", "clojure", "elixir",
        "erlang", "lua", "dart", "r", "matlab", "bash", "zsh",
        "zig", "nim", "fortran", "cobol", "ada", "prolog", "lisp",
        "ocaml", "f#", "fsharp", "julia", "delphi", "solidity",
        "vyper", "move", "racket", "scheme", "tcl",
    ].iter().copied().collect();

    // Tech keywords (non-language)
    let tech_map: Vec<(&str, EntityType)> = vec![
        ("sqlite", EntityType::Technology),
        ("http", EntityType::Technology),
        ("https", EntityType::Technology),
        ("api", EntityType::Technology),
        ("json", EntityType::Technology),
        ("docker", EntityType::Technology),
        ("git", EntityType::Technology),
        ("kubernetes", EntityType::Technology),
        ("k8s", EntityType::Technology),
        ("aws", EntityType::Technology),
        ("postgresql", EntityType::Technology),
        ("postgres", EntityType::Technology),
        ("mysql", EntityType::Technology),
        ("redis", EntityType::Technology),
        ("mongodb", EntityType::Technology),
        ("nginx", EntityType::Technology),
        ("linux", EntityType::Technology),
        ("tcp", EntityType::Technology),
        ("udp", EntityType::Technology),
        ("rest", EntityType::Technology),
        ("graphql", EntityType::Technology),
        ("grpc", EntityType::Technology),
        ("ssl", EntityType::Technology),
        ("tls", EntityType::Technology),
        ("oauth", EntityType::Technology),
        ("jwt", EntityType::Technology),
        ("yaml", EntityType::Technology),
        ("xml", EntityType::Technology),
        ("html", EntityType::Technology),
        ("css", EntityType::Technology),
        ("sql", EntityType::Technology),
        ("cli", EntityType::Technology),
        ("sdk", EntityType::Technology),
        ("cors", EntityType::Technology),
        ("restful", EntityType::Technology),
    ];

    // Project-indicator words
    let project_indicators: HashSet<&str> = [
        "project", "system", "framework", "platform", "toolkit",
        "library", "engine", "service", "pipeline", "protocol",
        "infrastructure", "module", "package", "sdk", "api",
    ].iter().copied().collect();

    let mut i = 0;
    let words: Vec<&str> = text.split_whitespace().collect();
    while i < words.len() {
        let token = words[i];

        // URL → Tool
        if token.starts_with("http://") || token.starts_with("https://") {
            add(&mut entities, &mut seen, token.to_string(), EntityType::Tool);
            i += 1;
            continue;
        }

        // @mention → Person
        if token.starts_with('@') && token.len() > 1 {
            let name = token[1..].trim_end_matches(|c: char| !c.is_alphanumeric()).to_string();
            if !name.is_empty() {
                add(&mut entities, &mut seen, name, EntityType::Person);
            }
            i += 1;
            continue;
        }

        // Check single-token keywords
        let clean = token.trim_end_matches(|c: char| matches!(c, '.' | ',' | '!' | '?' | ';' | ':' | '"' | '\'' | ')' | ']' | '}' | '>'))
            .trim_start_matches(|c: char| matches!(c, '"' | '\'' | '(' | '[' | '{' | '<'))
            .to_lowercase();
        if !clean.is_empty() {
            // Language check first (more specific)
            if lang_set.contains(clean.as_str()) {
                add(&mut entities, &mut seen, clean.clone(), EntityType::Language);
                i += 1;
                continue;
            }
            // Tech keyword check
            if let Some((_, typ)) = tech_map.iter().find(|(k, _)| *k == clean.as_str()) {
                add(&mut entities, &mut seen, clean.clone(), typ.clone());
                i += 1;
                continue;
            }
        }

        // Capitalized multi-word phrases → Concept or Project
        let trim_punct = |c: char| matches!(c, '.' | ',' | '!' | '?' | ';' | ':' | '"' | '\'' | ')' | ']' | '}' | '>');
        let trim_leading = |c: char| matches!(c, '"' | '\'' | '(' | '[' | '{' | '<');
        let trimmed_token: &str = token.trim_start_matches(trim_leading);
        let trimmed_clean = trimmed_token.trim_end_matches(trim_punct);
        let starts_upper = trimmed_clean.chars().next().map_or(false, |c| c.is_uppercase());
        if starts_upper && !trimmed_clean.is_empty() && trimmed_clean.chars().all(|c| c.is_alphanumeric() || c == '-') {
            let mut phrase = trimmed_clean.to_string();
            let mut j = i + 1;
            while j < words.len() {
                let next = words[j];
                let next_trimmed = next.trim_start_matches(trim_leading)
                                       .trim_end_matches(trim_punct);
                let next_clean = next_trimmed.to_lowercase();
                // Stop before standalone keywords (language or tech) so they're detected separately
                if lang_set.contains(next_clean.as_str())
                    || tech_map.iter().any(|(k, _)| *k == next_clean.as_str())
                {
                    break;
                }
                if next_trimmed.len() > 1
                    && next_trimmed.chars().next().map_or(false, |c| c.is_uppercase())
                    && next_trimmed.chars().all(|c| c.is_alphanumeric() || c == '-')
                {
                    phrase.push(' ');
                    phrase.push_str(next_trimmed);
                    j += 1;
                } else {
                    break;
                }
            }
            if j > i + 1 {
                let lower_phrase = phrase.to_lowercase();
                let typ = if project_indicators.iter().any(|pi| lower_phrase.contains(pi)) {
                    EntityType::Project
                } else {
                    EntityType::Concept
                };
                add(&mut entities, &mut seen, phrase, typ);
                i = j;
                continue;
            }
        }

        i += 1;
    }

    entities
}

/// Infer relationships between entities based on sentence-level co-occurrence and keywords.
/// Returns (source_index, target_index, relation_type, weight).
pub fn infer_relationships(text: &str, entities: &[(String, EntityType)]) -> Vec<(usize, usize, RelationType, f32)> {
    if entities.len() < 2 {
        return vec![];
    }

    let sentences: Vec<&str> = text.split(|c: char| c == '.' || c == '!' || c == '?')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut relations: Vec<(usize, usize, RelationType, f32)> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let rel_keywords: Vec<(&str, RelationType, f32)> = vec![
        ("depends", RelationType::Depends, 0.9),
        ("dependency", RelationType::Depends, 0.9),
        ("requires", RelationType::Depends, 0.8),
        ("uses", RelationType::Uses, 0.8),
        ("use", RelationType::Uses, 0.7),
        ("implement", RelationType::Implements, 0.9),
        ("implements", RelationType::Implements, 0.9),
        ("extend", RelationType::Extends, 0.8),
        ("extends", RelationType::Extends, 0.8),
        ("similar", RelationType::Similar, 0.8),
        ("like", RelationType::Similar, 0.5),
        ("creates", RelationType::Creates, 0.8),
        ("create", RelationType::Creates, 0.7),
        ("modifies", RelationType::Modifies, 0.8),
        ("modify", RelationType::Modifies, 0.7),
        ("part of", RelationType::PartOf, 0.9),
        ("belongs to", RelationType::PartOf, 0.8),
    ];

    for sentence in &sentences {
        let sent_lower = sentence.to_lowercase();
        let mut sent_entity_indices: Vec<usize> = Vec::new();

        for (idx, (name, _)) in entities.iter().enumerate() {
            if sent_lower.contains(&name.to_lowercase()) {
                sent_entity_indices.push(idx);
            }
        }

        // Determine relation type from keywords in this sentence
        let mut sent_rel_type = RelationType::Mentions;
        let mut sent_weight = 0.6;
        for (kw, rt, w) in &rel_keywords {
            if sent_lower.contains(kw) {
                sent_rel_type = rt.clone();
                sent_weight = *w;
                break;
            }
        }

        // Create relations between all entity pairs in the same sentence
        for a in 0..sent_entity_indices.len() {
            for b in (a + 1)..sent_entity_indices.len() {
                let src = sent_entity_indices[a];
                let tgt = sent_entity_indices[b];
                let key = if src < tgt { (src, tgt) } else { (tgt, src) };
                if seen.insert(key) {
                    relations.push((src, tgt, sent_rel_type.clone(), sent_weight));
                }
            }
        }
    }

    relations
}

// ── Token-efficient helpers ──────────────────────────────────────────

/// Rough token estimation (chars / 4 for English text).
pub fn estimate_tokens(text: &str) -> usize {
    (text.len() + 3) / 4
}

/// Create a summary by truncating to `max_tokens`.
pub fn summarize(title: &str, content: &str, max_tokens: usize) -> String {
    let full = format!("{}: {}", title, content);
    let estimated = estimate_tokens(&full);
    if estimated <= max_tokens { return full; }

    // Truncate: keep title + first N chars of content
    let title_tokens = estimate_tokens(title);
    let content_budget = max_tokens.saturating_sub(title_tokens + 2);
    let content_chars = content_budget * 4;
    if content_chars < 20 {
        title.chars().take(80).collect()
    } else {
        let truncated: String = content.chars().take(content_chars).collect();
        format!("{}: {}…", title, truncated)
    }
}

/// Compute importance score (0.0-1.0) from content signals.
pub fn compute_importance(title: &str, content: &str, _tags: &[&str]) -> f32 {
    let mut score = 0.5;
    // Longer content tends to be more important
    let len_score = (content.len() as f32 / 2000.0).min(0.3);
    score += len_score;
    // Title with keywords boosts importance
    let keywords = ["error", "critical", "important", "memory", "learn", "trade", "signal", "alert"];
    let kw_score = keywords.iter().filter(|k| title.to_lowercase().contains(*k)).count() as f32 * 0.1;
    score += kw_score;
    score.min(1.0)
}

/// Update importance with recency boost (decay factor).
pub fn decay_importance(importance: f32, hours_old: f64) -> f32 {
    importance * (1.0_f32 - (hours_old as f32 * 0.001).min(0.5))
}

pub type Embedding = Vec<f32>;
pub const EMBEDDING_DIMENSIONS: usize = 384;

pub fn compute_embedding(text: &str) -> Embedding {
    const DIM: usize = EMBEDDING_DIMENSIONS;
    let mut vec = vec![0.0f32; DIM];
    let text = text.to_lowercase();
    let chars: Vec<char> = text.chars().collect();

    if chars.len() < 3 {
        return vec;
    }

    for i in 0..chars.len() - 2 {
        let trigram: String = chars[i..i + 3].iter().collect();
        let hash = fnv1a_hash(&trigram);
        let dim1 = (hash as usize) % DIM;
        let dim2 = ((hash >> 8) as usize) % DIM;
        let dim3 = ((hash >> 16) as usize) % DIM;
        vec[dim1] += 1.0;
        vec[dim2] += 1.0;
        vec[dim3] += 0.5;
    }

    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in &mut vec {
            *x /= norm;
        }
    }

    vec
}

fn fnv1a_hash(s: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    for b in s.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0_f64;
    let mut norm_a = 0.0_f64;
    let mut norm_b = 0.0_f64;
    for i in 0..a.len() {
        let av = a[i] as f64;
        let bv = b[i] as f64;
        dot += av * bv;
        norm_a += av * av;
        norm_b += bv * bv;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 { 0.0 } else { dot / denom }
}

pub fn chunk_text(text: &str, max_chunk_tokens: usize) -> Vec<(String, String)> {
    if text.is_empty() {
        return vec![];
    }
    let estimated_tokens = estimate_tokens(text);
    if estimated_tokens <= max_chunk_tokens {
        let summary = summarize("chunk", text, 50);
        return vec![(text.to_string(), summary)];
    }
    let mut chunks = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let char_budget = max_chunk_tokens * 4;
    let mut start = 0;
    while start < len {
        let mut end = (start + char_budget).min(len);
        if end < len {
            let search_start = start.max(end.saturating_sub(40));
            let mut split_at = end;
            for i in (search_start..end).rev() {
                let c = chars[i];
                if c == '\n' && i + 1 < len && chars[i + 1] == '\n' {
                    split_at = i + 2;
                    break;
                }
                if (c == '.' || c == '!' || c == '?') && (i + 1 >= len || chars[i + 1] == ' ' || chars[i + 1] == '\n') {
                    split_at = i + 1;
                    break;
                }
            }
            if split_at == end {
                for i in (search_start..end).rev() {
                    if chars[i] == '\n' {
                        split_at = i + 1;
                        break;
                    }
                }
            }
            end = split_at;
            if end <= start {
                end = (start + char_budget).min(len);
            }
        }
        let chunk_str: String = chars[start..end].iter().collect();
        if chunk_str.trim().is_empty() {
            break;
        }
        let summary = summarize("chunk", &chunk_str, 50);
        chunks.push((chunk_str, summary));
        start = end;
        while start < len && chars[start].is_whitespace() {
            start += 1;
        }
    }
    chunks
}

pub fn compute_chunks(observation: &Observation) -> Vec<Chunk> {
    let segments = chunk_text(&observation.content, 256);
    let obs_id = observation.id.0 as u64;
    let mut chunks = Vec::new();
    for (seq, (content, summary)) in segments.into_iter().enumerate() {
        let token_count = estimate_tokens(&content) as u32;
        let embedding = compute_embedding(&content);
        chunks.push(Chunk {
            id: 0,
            observation_id: obs_id,
            content,
            summary,
            token_count,
            embedding,
            seq: seq as u32,
        });
    }
    chunks
}
