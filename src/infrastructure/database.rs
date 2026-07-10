use crate::domain::entities::{
    compute_chunks, compute_embedding, cosine_similarity, extract_entities, infer_relationships,
    summarize, Entity, EntityType, MemoryEntry, Observation, Relation, RelationType, SearchParams,
    SearchResult, SessionInfo,
};
use crate::domain::ports::DbValue;
use crate::domain::ports::StorageBackend;
use crate::domain::ports::StoragePort;
use crate::domain::types::{ObservationId, ObservationType};
use rusqlite::Connection;
use serde_json::{json, Value};
use std::sync::Mutex;

// ── DbValue ↔ rusqlite conversion ──────────────────────────────────

fn dbvalue_to_rusqlite(v: &DbValue) -> rusqlite::types::Value {
    match v {
        DbValue::Null => rusqlite::types::Value::Null,
        DbValue::Integer(i) => rusqlite::types::Value::Integer(*i),
        DbValue::Real(f) => rusqlite::types::Value::Real(*f),
        DbValue::Text(s) => rusqlite::types::Value::Text(s.clone()),
        DbValue::Blob(b) => rusqlite::types::Value::Blob(b.clone()),
    }
}

fn rusqlite_to_dbvalue(v: &rusqlite::types::Value) -> DbValue {
    match v {
        rusqlite::types::Value::Null => DbValue::Null,
        rusqlite::types::Value::Integer(i) => DbValue::Integer(*i),
        rusqlite::types::Value::Real(f) => DbValue::Real(*f),
        rusqlite::types::Value::Text(s) => DbValue::Text(s.clone()),
        rusqlite::types::Value::Blob(b) => DbValue::Blob(b.clone()),
    }
}

#[derive(Debug)]
pub struct SqliteBackend {
    conn: Mutex<Connection>,
}

impl SqliteBackend {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Mutex::new(conn),
        }
    }

    pub fn conn_lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }
}

impl StorageBackend for SqliteBackend {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn execute(&self, sql: &str, params: &[DbValue]) -> Result<u64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let rp: Vec<rusqlite::types::Value> = params.iter().map(dbvalue_to_rusqlite).collect();
        conn.execute(sql, rusqlite::params_from_iter(rp.iter()))
            .map(|c| c as u64)
            .map_err(|e| e.to_string())
    }

    fn query(&self, sql: &str, params: &[DbValue]) -> Result<Vec<Vec<DbValue>>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let col_count = stmt.column_count();
        let rp: Vec<rusqlite::types::Value> = params.iter().map(dbvalue_to_rusqlite).collect();
        let rows = stmt
            .query_map(rusqlite::params_from_iter(rp.iter()), |row| {
                let mut values = Vec::with_capacity(col_count);
                for i in 0..col_count {
                    values.push(rusqlite_to_dbvalue(
                        &row.get::<_, rusqlite::types::Value>(i)?,
                    ));
                }
                Ok(values)
            })
            .map_err(|e| e.to_string())?;
        let mut result = Vec::with_capacity(rows.size_hint().0);
        for row in rows {
            result.push(row.map_err(|e| e.to_string())?);
        }
        Ok(result)
    }

    fn execute_batch(&self, sql: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute_batch(sql).map_err(|e| e.to_string())
    }
}

#[cfg(feature = "postgres")]
pub struct PgBackend {
    pool: deadpool_postgres::Pool,
    rt: tokio::runtime::Runtime,
}

#[cfg(feature = "postgres")]
impl PgBackend {
    pub fn new(conn_string: &str) -> Result<Self, String> {
        use deadpool_postgres::Manager;
        use tokio_postgres::NoTls;

        let pg_config: tokio_postgres::Config = conn_string
            .parse()
            .map_err(|e| format!("Invalid connection string: {}", e))?;
        let mgr = Manager::new(pg_config, NoTls);
        let pool = deadpool_postgres::Pool::builder(mgr)
            .max_size(16)
            .build()
            .map_err(|e| e.to_string())?;
        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        Ok(Self { pool, rt })
    }
}

#[cfg(feature = "postgres")]
impl StorageBackend for PgBackend {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn execute(&self, sql: &str, params: &[DbValue]) -> Result<u64, String> {
        let client = self
            .rt
            .block_on(self.pool.get())
            .map_err(|e| e.to_string())?;
        let pg_params = pg_params_from_values(params);
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            pg_params.iter().map(|p| p.as_ref()).collect();
        self.rt
            .block_on(client.execute(sql, &param_refs))
            .map_err(|e| e.to_string())
    }

    fn query(&self, sql: &str, params: &[DbValue]) -> Result<Vec<Vec<DbValue>>, String> {
        let client = self
            .rt
            .block_on(self.pool.get())
            .map_err(|e| e.to_string())?;
        let stmt = self
            .rt
            .block_on(client.prepare(sql))
            .map_err(|e| e.to_string())?;
        let pg_params = pg_params_from_values(params);
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            pg_params.iter().map(|p| p.as_ref()).collect();
        let rows = self
            .rt
            .block_on(client.query(&stmt, &param_refs))
            .map_err(|e| e.to_string())?;
        let mut result = Vec::with_capacity(rows.len());
        for row in &rows {
            let mut values = Vec::with_capacity(row.len());
            for i in 0..row.len() {
                values.push(pg_value_from_row(row, i));
            }
            result.push(values);
        }
        Ok(result)
    }

    fn execute_batch(&self, sql: &str) -> Result<(), String> {
        let client = self
            .rt
            .block_on(self.pool.get())
            .map_err(|e| e.to_string())?;
        for statement in sql.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                self.rt
                    .block_on(client.execute(trimmed, &[]))
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}

#[cfg(feature = "postgres")]
fn pg_params_from_values(params: &[DbValue]) -> Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> {
    params
        .iter()
        .map(|v| match v {
            DbValue::Null => Box::new(None::<i64>) as Box<dyn tokio_postgres::types::ToSql + Sync>,
            DbValue::Integer(i) => Box::new(*i) as Box<dyn tokio_postgres::types::ToSql + Sync>,
            DbValue::Real(f) => Box::new(*f) as Box<dyn tokio_postgres::types::ToSql + Sync>,
            DbValue::Text(s) => Box::new(s.clone()) as Box<dyn tokio_postgres::types::ToSql + Sync>,
            DbValue::Blob(b) => Box::new(b.clone()) as Box<dyn tokio_postgres::types::ToSql + Sync>,
        })
        .collect()
}

#[cfg(feature = "postgres")]
fn pg_value_from_row(row: &tokio_postgres::Row, idx: usize) -> DbValue {
    if let Some(column) = row.columns().get(idx) {
        match column.type_().name() {
            "int2" | "smallint" => row
                .try_get::<_, i16>(idx)
                .map(|v| DbValue::Integer(v as i64))
                .unwrap_or(DbValue::Null),
            "int4" | "integer" => row
                .try_get::<_, i32>(idx)
                .map(|v| DbValue::Integer(v as i64))
                .unwrap_or(DbValue::Null),
            "int8" | "bigint" => row
                .try_get::<_, i64>(idx)
                .map(DbValue::Integer)
                .unwrap_or(DbValue::Null),
            "float4" | "real" => row
                .try_get::<_, f32>(idx)
                .map(|v| DbValue::Real(v as f64))
                .unwrap_or(DbValue::Null),
            "float8" | "double precision" => row
                .try_get::<_, f64>(idx)
                .map(DbValue::Real)
                .unwrap_or(DbValue::Null),
            "text" | "varchar" | "bpchar" | "name" => row
                .try_get::<_, String>(idx)
                .map(DbValue::Text)
                .unwrap_or(DbValue::Null),
            "bytea" => row
                .try_get::<_, Vec<u8>>(idx)
                .map(DbValue::Blob)
                .unwrap_or(DbValue::Null),
            "bool" | "boolean" => row
                .try_get::<_, bool>(idx)
                .map(|v| DbValue::Integer(if v { 1 } else { 0 }))
                .unwrap_or(DbValue::Null),
            _ => DbValue::Null,
        }
    } else {
        DbValue::Null
    }
}

pub struct Database {
    backend: Box<dyn StorageBackend>,
}

impl StoragePort for Database {
    fn save_observation(&self, obs: &Observation) -> Result<ObservationId, String> {
        Database::save_observation_impl(self, obs)
    }
    fn search_observations(&self, params: &SearchParams) -> Result<Vec<SearchResult>, String> {
        Database::search_observations_impl(self, params)
    }
    fn recent_observations(&self, limit: usize) -> Result<Vec<Observation>, String> {
        Database::recent_observations_impl(self, limit)
    }
    fn get_by_id(&self, id: i64) -> Result<Option<Observation>, String> {
        Database::get_by_id_impl(self, id)
    }
    fn delete(&self, id: i64) -> Result<(), String> {
        Database::delete_impl(self, id)
    }
}

impl Database {
    pub fn new() -> Self {
        let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
        Self {
            backend: Box::new(SqliteBackend::new(conn)),
        }
    }

    pub fn new_with_key(_key: Option<Vec<u8>>) -> Self {
        Self {
            backend: Box::new(SqliteBackend::new(
                Connection::open_in_memory().expect("Failed to create in-memory database"),
            )),
        }
    }

    pub fn new_with_path(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        Ok(Self {
            backend: Box::new(SqliteBackend::new(conn)),
        })
    }

    #[cfg(feature = "postgres")]
    pub fn new_postgres(conn_string: &str) -> Result<Self, String> {
        let backend = PgBackend::new(conn_string)?;
        Ok(Self {
            backend: Box::new(backend),
        })
    }

    pub fn from_backend(backend: Box<dyn StorageBackend>) -> Self {
        Self { backend }
    }

    pub fn get_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>, String> {
        self.backend
            .as_any()
            .downcast_ref::<SqliteBackend>()
            .map(|sb| sb.conn_lock())
            .ok_or_else(|| "get_conn() requires SQLite backend".to_string())
    }

    pub fn init(&self) -> Result<(), String> {
        self.run_migrations()
    }

    fn schema_version(&self) -> Result<i64, String> {
        let rows = self.backend.query("PRAGMA user_version", &[])?;
        Ok(rows
            .first()
            .and_then(|row| row.first())
            .and_then(|v| {
                if let DbValue::Integer(i) = v {
                    Some(*i)
                } else {
                    None
                }
            })
            .unwrap_or(0))
    }

    fn run_migrations(&self) -> Result<(), String> {
        let version = self.schema_version()?;

        if version < 1 {
            self.backend.execute_batch(
                "CREATE TABLE IF NOT EXISTS observations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id TEXT,
                    title TEXT,
                    summary TEXT DEFAULT '',
                    content TEXT,
                    project TEXT,
                    tags TEXT DEFAULT '',
                    created_at INTEGER DEFAULT (strftime('%s','now')),
                    obs_type TEXT DEFAULT 'Note',
                    importance REAL DEFAULT 0.5,
                    token_count INTEGER DEFAULT 0,
                    access_count INTEGER DEFAULT 0
                )",
            )?;
            self.backend.execute_batch(
                "CREATE INDEX IF NOT EXISTS idx_obs_importance ON observations(importance DESC)"
            ).ok();
            self.backend
                .execute_batch(
                    "CREATE INDEX IF NOT EXISTS idx_obs_created ON observations(created_at DESC)",
                )
                .ok();
            self.backend.execute_batch(
                "CREATE TABLE IF NOT EXISTS embeddings (
                    observation_id INTEGER PRIMARY KEY,
                    vector BLOB NOT NULL,
                    FOREIGN KEY (observation_id) REFERENCES observations(id) ON DELETE CASCADE
                )",
            )?;
            self.backend.execute_batch(
                "CREATE TABLE IF NOT EXISTS chunks (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    observation_id INTEGER NOT NULL,
                    content TEXT NOT NULL,
                    summary TEXT DEFAULT '',
                    token_count INTEGER DEFAULT 0,
                    embedding BLOB,
                    seq INTEGER DEFAULT 0,
                    FOREIGN KEY (observation_id) REFERENCES observations(id) ON DELETE CASCADE
                )",
            )?;
            self.backend
                .execute_batch(
                    "CREATE INDEX IF NOT EXISTS idx_chunks_obs ON chunks(observation_id)",
                )
                .ok();
            self.backend.execute_batch(
                "CREATE TABLE IF NOT EXISTS entities (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    entity_type TEXT NOT NULL,
                    aliases TEXT DEFAULT '',
                    embedding BLOB,
                    mention_count INTEGER DEFAULT 1,
                    first_seen INTEGER NOT NULL,
                    last_seen INTEGER NOT NULL
                )",
            )?;
            self.backend.execute_batch(
                "CREATE TABLE IF NOT EXISTS relations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    source_id INTEGER NOT NULL,
                    target_id INTEGER NOT NULL,
                    relation_type TEXT NOT NULL,
                    weight REAL DEFAULT 1.0,
                    observation_id INTEGER,
                    created_at INTEGER NOT NULL,
                    FOREIGN KEY (source_id) REFERENCES entities(id) ON DELETE CASCADE,
                    FOREIGN KEY (target_id) REFERENCES entities(id) ON DELETE CASCADE
                )",
            )?;
            self.backend
                .execute_batch(
                    "CREATE INDEX IF NOT EXISTS idx_relations_source ON relations(source_id)",
                )
                .ok();
            self.backend
                .execute_batch(
                    "CREATE INDEX IF NOT EXISTS idx_relations_target ON relations(target_id)",
                )
                .ok();
            self.backend
                .execute_batch(
                    "CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(entity_type)",
                )
                .ok();
            self.backend.execute_batch("PRAGMA user_version = 1")?;
        }

        if version < 2 {
            // Migrate from old schema (20 columns, no summary/tags/obs_type/importance/etc.)
            self.backend
                .execute_batch("ALTER TABLE observations ADD COLUMN summary TEXT DEFAULT ''")
                .ok();
            self.backend
                .execute_batch("ALTER TABLE observations ADD COLUMN tags TEXT DEFAULT ''")
                .ok();
            self.backend
                .execute_batch("ALTER TABLE observations ADD COLUMN obs_type TEXT DEFAULT 'Note'")
                .ok();
            self.backend
                .execute_batch("ALTER TABLE observations ADD COLUMN importance REAL DEFAULT 0.5")
                .ok();
            self.backend
                .execute_batch("ALTER TABLE observations ADD COLUMN token_count INTEGER DEFAULT 0")
                .ok();
            self.backend
                .execute_batch("ALTER TABLE observations ADD COLUMN access_count INTEGER DEFAULT 0")
                .ok();
            self.backend.execute_batch("PRAGMA user_version = 2")?;
        }

        if version < 3 {
            // Add FTS5 full-text search virtual table
            self.backend.execute_batch(
                "CREATE VIRTUAL TABLE IF NOT EXISTS observations_fts USING fts5(
                    title, summary, content, project,
                    content=observations,
                    content_rowid=id,
                    tokenize='porter unicode61'
                )",
            )?;
            // Populate FTS index from existing data
            self.backend.execute_batch(
                "INSERT INTO observations_fts(rowid, title, summary, content, project)
                 SELECT id, title, summary, content, COALESCE(project, '') FROM observations",
            )?;
            self.backend.execute_batch("PRAGMA user_version = 3")?;
        }

        Ok(())
    }

    fn save_observation_impl(&self, obs: &Observation) -> Result<ObservationId, String> {
        let tags_str = obs.tags.join(",");
        self.backend.execute(
            "INSERT INTO observations (session_id, title, summary, content, project, tags, obs_type, importance, token_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            &[
                DbValue::Text(obs.session_id.clone()),
                DbValue::Text(obs.title.clone()),
                DbValue::Text(obs.summary.clone()),
                DbValue::Text(obs.content.clone()),
                obs.project.as_ref().map(|p| DbValue::Text(p.clone())).unwrap_or(DbValue::Null),
                DbValue::Text(tags_str),
                DbValue::Text(format!("{:?}", obs.observation_type)),
                DbValue::Real(obs.importance as f64),
                DbValue::Integer(obs.token_count as i64),
            ],
        )?;
        let rows = self.backend.query("SELECT last_insert_rowid()", &[])?;
        let id = get_i64(&rows[0][0]).unwrap_or(0);

        // Sync FTS5 index
        self.backend.execute(
            "INSERT INTO observations_fts(rowid, title, summary, content, project) VALUES (?1, ?2, ?3, ?4, ?5)",
            &[
                DbValue::Integer(id),
                DbValue::Text(obs.title.clone()),
                DbValue::Text(obs.summary.clone()),
                DbValue::Text(obs.content.clone()),
                obs.project.as_ref().map(|p| DbValue::Text(p.clone())).unwrap_or(DbValue::Text(String::new())),
            ],
        )?;

        let text = format!("{} {}", obs.title, obs.content);
        let embedding = compute_embedding(&text);
        let blob = serialize_embedding(&embedding);
        self.backend.execute(
            "INSERT OR REPLACE INTO embeddings (observation_id, vector) VALUES (?1, ?2)",
            &[DbValue::Integer(id), DbValue::Blob(blob)],
        )?;
        let mut obs_clone = obs.clone();
        obs_clone.id = ObservationId::new(id);
        let chunks = compute_chunks(&obs_clone);
        for chunk in &chunks {
            let emb_blob = serialize_embedding(&chunk.embedding);
            self.backend.execute(
                "INSERT INTO chunks (observation_id, content, summary, token_count, embedding, seq)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                &[
                    DbValue::Integer(chunk.observation_id as i64),
                    DbValue::Text(chunk.content.clone()),
                    DbValue::Text(chunk.summary.clone()),
                    DbValue::Integer(chunk.token_count as i64),
                    DbValue::Blob(emb_blob),
                    DbValue::Integer(chunk.seq as i64),
                ],
            )?;
        }
        let text = format!("{} {}", obs.title, obs.content);
        let entity_list = extract_entities(&text);
        let relationships = infer_relationships(&text, &entity_list);

        let now = crate::domain::types::Timestamp::now().0;
        for (name, entity_type) in &entity_list {
            let emb = compute_embedding(name);
            let emb_blob = serialize_embedding(&emb);
            self.backend.execute(
                "INSERT INTO entities (name, entity_type, aliases, embedding, mention_count, first_seen, last_seen)
                 VALUES (?1, ?2, '', ?3, 1, ?4, ?4)
                 ON CONFLICT(name) DO UPDATE SET
                    mention_count = mention_count + 1,
                    last_seen = ?4",
                &[
                    DbValue::Text(name.clone()),
                    DbValue::Text(format!("{:?}", entity_type)),
                    DbValue::Blob(emb_blob),
                    DbValue::Integer(now),
                ],
            )?;
        }

        for (src_idx, tgt_idx, rel_type, weight) in relationships {
            let src_name = &entity_list[src_idx].0;
            let tgt_name = &entity_list[tgt_idx].0;
            let src_rows = self.backend.query(
                "SELECT id FROM entities WHERE name = ?1",
                &[DbValue::Text(src_name.clone())],
            )?;
            let tgt_rows = self.backend.query(
                "SELECT id FROM entities WHERE name = ?1",
                &[DbValue::Text(tgt_name.clone())],
            )?;
            let src_id = src_rows
                .first()
                .map(|r| get_i64(&r[0]))
                .flatten()
                .unwrap_or(0);
            let tgt_id = tgt_rows
                .first()
                .map(|r| get_i64(&r[0]))
                .flatten()
                .unwrap_or(0);
            self.backend.execute(
                "INSERT INTO relations (source_id, target_id, relation_type, weight, observation_id, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                &[
                    DbValue::Integer(src_id),
                    DbValue::Integer(tgt_id),
                    DbValue::Text(format!("{:?}", rel_type)),
                    DbValue::Real(weight as f64),
                    DbValue::Integer(id),
                    DbValue::Integer(now),
                ],
            )?;
        }

        Ok(ObservationId::new(id))
    }

    fn search_observations_impl(&self, params: &SearchParams) -> Result<Vec<SearchResult>, String> {
        let query = params.query.to_lowercase();
        let limit = params.limit.unwrap_or(10) as usize;
        let max_tokens = params.max_tokens.unwrap_or(1024);
        let min_imp = params.min_importance.unwrap_or(0.0);

        let rows = self.backend.query(
            "SELECT id, session_id, title, summary, content, project, tags, created_at, obs_type, importance, token_count, access_count
             FROM observations WHERE importance >= ?1 ORDER BY importance DESC",
            &[DbValue::Real(min_imp as f64)],
        )?;

        let mut results: Vec<SearchResult> = Vec::new();
        let mut token_budget = max_tokens as i64;

        if params.use_semantic {
            let query_embedding = compute_embedding(&params.query);
            let mut candidates: Vec<(Observation, f64)> = Vec::new();

            for row in &rows {
                let obs = row_to_observation(row)?;
                let text_score = text_match_score(&query, &obs.title, &obs.content);
                let emb_rows = self.backend.query(
                    "SELECT vector FROM embeddings WHERE observation_id = ?1",
                    &[DbValue::Integer(obs.id.0)],
                )?;
                let semantic_score = emb_rows
                    .first()
                    .map(|r| get_blob(&r[0]))
                    .flatten()
                    .and_then(|blob| deserialize_embedding(blob))
                    .map(|emb| cosine_similarity(&query_embedding, &emb))
                    .unwrap_or(0.0);

                let combined_score = text_score * 0.4 + semantic_score * 0.6;
                if combined_score > 0.0 {
                    candidates.push((obs, combined_score));
                }
            }

            candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            for (obs, score) in candidates {
                if token_budget <= 0 || results.len() >= limit {
                    break;
                }
                let tc = obs.token_count;
                token_budget -= tc as i64;
                results.push(SearchResult {
                    observation: obs,
                    score,
                    token_cost: tc,
                    matched_chunk_ids: vec![],
                });
            }
        } else {
            for row in &rows {
                let obs = row_to_observation(row)?;
                if token_budget <= 0 {
                    break;
                }

                let score = text_match_score(&query, &obs.title, &obs.content);
                let tc = obs.token_count as i64;

                if score > 0.0 {
                    token_budget -= tc;
                    results.push(SearchResult {
                        observation: obs.clone(),
                        score,
                        token_cost: obs.token_count,
                        matched_chunk_ids: vec![],
                    });
                }
                if results.len() >= limit {
                    break;
                }
            }
        }

        // Fallback: search chunks when no observations match the query
        if results.is_empty() && !query.is_empty() {
            if params.use_semantic {
                let query_embedding = compute_embedding(&params.query);
                let crows = self.backend.query(
                    "SELECT c.id, c.observation_id, c.embedding, c.content
                     FROM chunks c JOIN observations o ON c.observation_id = o.id
                     WHERE o.importance >= ?1",
                    &[DbValue::Real(min_imp as f64)],
                )?;
                let mut obs_scores: std::collections::HashMap<i64, (f64, Vec<u64>)> =
                    std::collections::HashMap::new();
                for r in &crows {
                    let cid = get_i64(&r[0]).unwrap_or(0);
                    let oid = get_i64(&r[1]).unwrap_or(0);
                    let emb = get_blob(&r[2]).and_then(|b| deserialize_embedding(b));
                    let content = get_str(&r[3]).unwrap_or("");
                    if let Some(ref emb) = emb {
                        let ts = text_match_score(&query, "", content);
                        let ss = cosine_similarity(&query_embedding, emb);
                        let combo = ts * 0.3 + ss * 0.7;
                        if combo > 0.0 {
                            let e = obs_scores.entry(oid).or_insert((0.0, vec![]));
                            e.0 = e.0.max(combo);
                            e.1.push(cid as u64);
                        }
                    }
                }
                let mut sorted: Vec<(i64, f64, Vec<u64>)> =
                    obs_scores.into_iter().map(|(k, v)| (k, v.0, v.1)).collect();
                sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                for (oid, score, cids) in sorted {
                    if token_budget <= 0 || results.len() >= limit {
                        break;
                    }
                    let obs_rows = self.backend.query(
                        "SELECT id, session_id, title, summary, content, project, tags, created_at, obs_type, importance, token_count, access_count
                         FROM observations WHERE id = ?1",
                        &[DbValue::Integer(oid)],
                    )?;
                    if let Some(orow) = obs_rows.first() {
                        if let Ok(obs) = row_to_observation(orow) {
                            let tc = obs.token_count;
                            token_budget -= tc as i64;
                            results.push(SearchResult {
                                observation: obs,
                                score,
                                token_cost: tc,
                                matched_chunk_ids: cids,
                            });
                        }
                    }
                }
            } else {
                let like = format!("%{}%", query);
                let crows = self.backend.query(
                    "SELECT c.id, c.observation_id
                     FROM chunks c JOIN observations o ON c.observation_id = o.id
                     WHERE LOWER(c.content) LIKE ?1 AND o.importance >= ?2
                     ORDER BY c.id",
                    &[DbValue::Text(like), DbValue::Real(min_imp as f64)],
                )?;
                let mut obs_chunks: std::collections::HashMap<i64, Vec<u64>> =
                    std::collections::HashMap::new();
                for r in &crows {
                    let cid = get_i64(&r[0]).unwrap_or(0);
                    let oid = get_i64(&r[1]).unwrap_or(0);
                    obs_chunks.entry(oid).or_default().push(cid as u64);
                }
                for (oid, cids) in &obs_chunks {
                    if token_budget <= 0 || results.len() >= limit {
                        break;
                    }
                    let obs_rows = self.backend.query(
                        "SELECT id, session_id, title, summary, content, project, tags, created_at, obs_type, importance, token_count, access_count
                         FROM observations WHERE id = ?1",
                        &[DbValue::Integer(*oid)],
                    )?;
                    if let Some(orow) = obs_rows.first() {
                        if let Ok(obs) = row_to_observation(orow) {
                            let tc = obs.token_count;
                            token_budget -= tc as i64;
                            let score = cids.len() as f64;
                            results.push(SearchResult {
                                observation: obs,
                                score,
                                token_cost: tc,
                                matched_chunk_ids: cids.clone(),
                            });
                        }
                    }
                }
            }
        }

        // Update access_count for retrieved entries
        let ids: Vec<String> = results
            .iter()
            .map(|r| r.observation.id.0.to_string())
            .collect();
        if !ids.is_empty() {
            self.backend
                .execute_batch(&format!(
                    "UPDATE observations SET access_count = access_count + 1 WHERE id IN ({})",
                    ids.join(",")
                ))
                .ok();
        }

        Ok(results)
    }

    fn recent_observations_impl(&self, limit: usize) -> Result<Vec<Observation>, String> {
        let rows = self.backend.query(
            "SELECT id, session_id, title, summary, content, project, tags, created_at, obs_type, importance, token_count, access_count
             FROM observations ORDER BY created_at DESC LIMIT ?1",
            &[DbValue::Integer(limit as i64)],
        )?;

        let mut results = Vec::new();
        for row in &rows {
            results.push(row_to_observation(row)?);
        }
        Ok(results)
    }

    fn get_by_id_impl(&self, id: i64) -> Result<Option<Observation>, String> {
        let rows = self.backend.query(
            "SELECT id, session_id, title, summary, content, project, tags, created_at, obs_type, importance, token_count, access_count
             FROM observations WHERE id = ?1",
            &[DbValue::Integer(id)],
        )?;

        match rows.first() {
            Some(row) => Ok(Some(row_to_observation(row)?)),
            None => Ok(None),
        }
    }

    fn delete_impl(&self, id: i64) -> Result<(), String> {
        self.backend
            .execute(
                "DELETE FROM observations_fts WHERE rowid = ?1",
                &[DbValue::Integer(id)],
            )
            .ok();
        self.backend.execute(
            "DELETE FROM observations WHERE id = ?1",
            &[DbValue::Integer(id)],
        )?;
        Ok(())
    }

    // ── Memory (token-efficient) methods ─────────────────────────────

    pub fn acquire_lock(
        &self,
        _resource: &str,
        _session_id: &str,
        _lock_type: &str,
        _metadata: Option<&str>,
        _ttl_secs: i64,
    ) -> Result<bool, String> {
        Ok(true)
    }
    pub fn release_lock(&self, _resource: &str) -> Result<(), String> {
        Ok(())
    }
    pub fn delete_observation(
        &self,
        _id: ObservationId,
        _agent_id: &str,
        _reason: Option<&str>,
    ) -> Result<(), String> {
        Ok(())
    }
    pub fn get_global_context(&self, _session_id: &str) -> Result<Option<Value>, String> {
        Ok(Some(serde_json::json!({})))
    }
    pub fn update_observation(
        &self,
        _id: ObservationId,
        _content: &str,
        _agent_id: &str,
        _reason: Option<&str>,
    ) -> Result<(), String> {
        Ok(())
    }

    pub fn save_memory(&self, memory: &MemoryEntry) -> Result<(), String> {
        let title = format!("memory-{}", memory.session_id.as_str());
        let token_count = memory.token_count;
        let importance = memory.importance;
        self.backend.execute(
            "INSERT INTO observations (session_id, title, summary, content, obs_type, importance, token_count)
             VALUES (?1, ?2, ?3, ?4, 'Memory', ?5, ?6)",
            &[
                DbValue::Text(memory.session_id.as_str().to_string()),
                DbValue::Text(title.clone()),
                DbValue::Text(summarize(&title, &memory.content, 30)),
                DbValue::Text(memory.content.clone()),
                DbValue::Real(importance as f64),
                DbValue::Integer(token_count as i64),
            ],
        )?;
        Ok(())
    }

    /// Full-text search using FTS5 with BM25 ranking.
    ///
    /// Searches across title, summary, content, and project fields.
    /// Results are ranked by FTS5 relevance and limited by token budget.
    pub fn search_fts(
        &self,
        query: &str,
        project: Option<&str>,
        limit: i32,
        max_tokens: Option<u32>,
    ) -> Result<Vec<Value>, String> {
        let limit = limit.max(1).min(100) as usize;
        let max_tokens = max_tokens.unwrap_or(1024) as i64;
        let mut token_budget = max_tokens;

        let sql = if let Some(_p) = project {
            format!(
                "SELECT o.id, o.title, o.summary, o.content, o.importance, o.token_count,
                        bm25(observations_fts, 0.0, 1.0, 1.0, 2.0) AS score
                 FROM observations_fts
                 JOIN observations o ON o.id = observations_fts.rowid
                 WHERE observations_fts MATCH ?1 AND o.project = ?2
                 ORDER BY score DESC LIMIT {}",
                limit * 2
            )
        } else {
            format!(
                "SELECT o.id, o.title, o.summary, o.content, o.importance, o.token_count,
                        bm25(observations_fts, 0.0, 1.0, 1.0, 2.0) AS score
                 FROM observations_fts
                 JOIN observations o ON o.id = observations_fts.rowid
                 WHERE observations_fts MATCH ?1
                 ORDER BY score DESC LIMIT {}",
                limit * 2
            )
        };

        let rows = if let Some(p) = project {
            self.backend.query(
                &sql,
                &[
                    DbValue::Text(query.to_string()),
                    DbValue::Text(p.to_string()),
                ],
            )?
        } else {
            self.backend
                .query(&sql, &[DbValue::Text(query.to_string())])?
        };

        let mut results = Vec::new();
        for row in &rows {
            if token_budget <= 0 || results.len() >= limit {
                break;
            }
            let id = get_i64(&row[0]).unwrap_or(0);
            let title = get_str(&row[1]).unwrap_or("").to_string();
            let summary = get_str(&row[2]).unwrap_or("").to_string();
            let content = get_str(&row[3]).unwrap_or("").to_string();
            let importance = get_f64(&row[4]).unwrap_or(0.0);
            let token_count = get_i64(&row[5]).unwrap_or(0) as u32;
            let score = get_f64(&row[6]).unwrap_or(0.0);

            token_budget -= token_count as i64;
            results.push(json!({
                "id": id,
                "title": title,
                "summary": summary,
                "content": if content.len() > 200 { format!("{}...", content.chars().take(200).collect::<String>()) } else { content },
                "importance": importance,
                "token_count": token_count,
                "score": score,
            }));
        }

        Ok(results)
    }

    pub fn retain(&self, max_tokens: u64) -> Result<u64, String> {
        let rows = self.backend.query(
            "SELECT COALESCE(SUM(token_count), 0) FROM observations",
            &[],
        )?;
        let total: i64 = rows.first().map(|r| get_i64(&r[0])).flatten().unwrap_or(0);

        if (total as u64) <= max_tokens {
            return Ok(0);
        }

        let excess = (total as u64) - max_tokens;
        self.backend.execute(
            "DELETE FROM observations WHERE id IN (
                SELECT id FROM observations ORDER BY importance ASC, access_count ASC LIMIT 1000
            ) AND (SELECT COALESCE(SUM(token_count), 0) FROM observations) > ?1",
            &[DbValue::Integer(max_tokens as i64)],
        )?;

        Ok(excess)
    }

    pub fn stats_db(&self) -> Result<crate::domain::ports::MemoryStats, String> {
        let entries = self
            .backend
            .query("SELECT COUNT(*) FROM observations", &[])?;
        let total_entries: i64 = entries
            .first()
            .map(|r| get_i64(&r[0]))
            .flatten()
            .unwrap_or(0);
        let tokens = self.backend.query(
            "SELECT COALESCE(SUM(token_count), 0) FROM observations",
            &[],
        )?;
        let total_tokens: i64 = tokens
            .first()
            .map(|r| get_i64(&r[0]))
            .flatten()
            .unwrap_or(0);
        let avg = self.backend.query(
            "SELECT COALESCE(AVG(importance), 0.0) FROM observations",
            &[],
        )?;
        let avg_imp: f64 = avg.first().map(|r| get_f64(&r[0])).flatten().unwrap_or(0.0);
        let sessions = self
            .backend
            .query("SELECT COUNT(DISTINCT session_id) FROM observations", &[])?;
        let unique_sessions: i64 = sessions
            .first()
            .map(|r| get_i64(&r[0]))
            .flatten()
            .unwrap_or(0);
        Ok(crate::domain::ports::MemoryStats {
            total_entries: total_entries as u64,
            total_tokens: total_tokens as u64,
            avg_importance: avg_imp as f32,
            unique_sessions: unique_sessions as u64,
        })
    }

    // Legacy passthrough methods
    pub fn get_active_agents(&self, _project: Option<&str>) -> Result<Vec<Value>, String> {
        Ok(vec![])
    }
    pub fn register_agent_session(
        &self,
        _agent_type: &str,
        _instance: &str,
        _project: Option<&str>,
        _ttl: Option<i64>,
    ) -> Result<String, String> {
        Ok(uuid::Uuid::new_v4().to_string())
    }
    pub fn create_task(
        &self,
        _project: &str,
        _task_type: &str,
        _payload: &str,
        _priority: i32,
    ) -> Result<String, String> {
        Ok(uuid::Uuid::new_v4().to_string())
    }
    pub fn list_tasks(
        &self,
        _project: Option<&str>,
        _task_type: Option<&str>,
        _status: Option<&str>,
        _limit: Option<i32>,
    ) -> Result<Vec<Value>, String> {
        Ok(vec![])
    }
    pub fn claim_task(
        &self,
        _session_id: &str,
        _task_type: Option<&str>,
    ) -> Result<Option<Value>, String> {
        Ok(None)
    }
    pub fn cancel_task(&self, _task_id: &str) -> Result<(), String> {
        Ok(())
    }
    pub fn complete_task(
        &self,
        _task_id: &str,
        _result: Option<&str>,
        _error: Option<&str>,
    ) -> Result<(), String> {
        Ok(())
    }
    pub fn publish_event(
        &self,
        _event_type: &str,
        _from: &str,
        _to: Option<&str>,
        _project: Option<&str>,
        _channel: &str,
        _content: &str,
        _priority: i32,
    ) -> Result<i64, String> {
        Ok(1)
    }
    pub fn broadcast_event(
        &self,
        _event_type: &str,
        _from: &str,
        _project: Option<&str>,
        _channel: &str,
        _content: &str,
        _priority: i32,
    ) -> Result<i64, String> {
        Ok(1)
    }
    pub fn poll_events(
        &self,
        _since: i64,
        _channel: Option<&str>,
        _project: Option<&str>,
        _limit: i32,
    ) -> Result<Vec<Value>, String> {
        Ok(vec![])
    }
    pub fn get_pending_messages(&self, _session_id: &str) -> Result<Vec<Value>, String> {
        Ok(vec![])
    }
    pub fn acknowledge_event(&self, _event_id: i64) -> Result<(), String> {
        Ok(())
    }
    pub fn get_chunks_by_project(
        &self,
        _project: &str,
        _filter: Option<&str>,
    ) -> Result<Vec<Value>, String> {
        Ok(vec![])
    }
    pub fn get_agent_details(&self, _session_id: &str) -> Result<Option<Value>, String> {
        Ok(None)
    }
    pub fn agent_heartbeat(&self, _session_id: &str, _task: Option<&str>) -> Result<(), String> {
        Ok(())
    }
    pub fn audit_task(
        &self,
        _task_id: &str,
        _auditor: &str,
        _status: &str,
        _notes: Option<&str>,
    ) -> Result<(), String> {
        Ok(())
    }
    pub fn db_health(&self) -> Result<Value, String> {
        Ok(json!({"status": "healthy", "connections": 1}))
    }
    pub fn get_stats(&self) -> Result<Value, String> {
        Ok(json!({"observations": 0, "agents": 0, "tasks": 0}))
    }
    pub fn stats_legacy(&self) -> Result<Value, String> {
        self.get_stats()
    }
    pub fn create_chunk(
        &self,
        _project: &str,
        _title: &str,
        _content: &str,
        _metadata: Option<&str>,
        _embedding: usize,
    ) -> Result<String, String> {
        Ok(uuid::Uuid::new_v4().to_string())
    }
    pub fn list_sessions(&self) -> Result<Vec<SessionInfo>, String> {
        Ok(vec![])
    }

    // Legacy stub methods
    pub fn get_observation(&self, _id: ObservationId) -> Result<Option<Observation>, String> {
        Ok(None)
    }
    pub fn get_timeline(
        &self,
        _limit: i32,
    ) -> Result<Vec<crate::domain::entities::TimelineEntry>, String> {
        Ok(vec![])
    }
    pub fn stats(&self) -> Result<Value, String> {
        self.db_health()
    }
    /// Inherent method for compatibility: delegates to StoragePort impl
    pub fn save_observation(&self, obs: &Observation) -> Result<ObservationId, String> {
        <Self as StoragePort>::save_observation(self, obs)
    }
    pub fn search_observations(&self, params: &SearchParams) -> Result<Vec<SearchResult>, String> {
        <Self as StoragePort>::search_observations(self, params)
    }

    pub fn get_all_observations(&self) -> Result<Vec<Observation>, String> {
        let rows = self.backend.query(
            "SELECT id, session_id, title, summary, content, project, tags, created_at, obs_type, importance, token_count, access_count
             FROM observations ORDER BY id",
            &[],
        )?;
        let mut results = Vec::new();
        for row in &rows {
            results.push(row_to_observation(row)?);
        }
        Ok(results)
    }

    pub fn update_summary(&self, id: ObservationId, summary: &str) -> Result<(), String> {
        self.backend.execute(
            "UPDATE observations SET summary = ?1 WHERE id = ?2",
            &[DbValue::Text(summary.to_string()), DbValue::Integer(id.0)],
        )?;
        Ok(())
    }

    pub fn optimize(
        &self,
        max_tokens: u64,
    ) -> Result<crate::infrastructure::optimizer::OptimizationStats, String> {
        use crate::infrastructure::optimizer::AutoOptimizer;
        let optimizer = AutoOptimizer::new(max_tokens);
        optimizer.optimize(self)
    }

    // ── Knowledge Graph methods ──────────────────────────────────────

    pub fn extract_and_store_entities(
        &self,
        observation_id: u64,
        content: &str,
        title: &str,
    ) -> Result<(), String> {
        let text = format!("{} {}", title, content);
        let entity_list = extract_entities(&text);
        let relationships = infer_relationships(&text, &entity_list);

        let now = crate::domain::types::Timestamp::now().0;

        for (name, entity_type) in &entity_list {
            let emb = compute_embedding(name);
            let emb_blob = serialize_embedding(&emb);
            self.backend.execute(
                "INSERT INTO entities (name, entity_type, aliases, embedding, mention_count, first_seen, last_seen)
                 VALUES (?1, ?2, '', ?3, 1, ?4, ?4)
                 ON CONFLICT(name) DO UPDATE SET
                    mention_count = mention_count + 1,
                    last_seen = ?4",
                &[
                    DbValue::Text(name.clone()),
                    DbValue::Text(format!("{:?}", entity_type)),
                    DbValue::Blob(emb_blob),
                    DbValue::Integer(now),
                ],
            )?;
        }

        for (src_idx, tgt_idx, rel_type, weight) in relationships {
            let src_name = &entity_list[src_idx].0;
            let tgt_name = &entity_list[tgt_idx].0;
            let src_rows = self.backend.query(
                "SELECT id FROM entities WHERE name = ?1",
                &[DbValue::Text(src_name.clone())],
            )?;
            let tgt_rows = self.backend.query(
                "SELECT id FROM entities WHERE name = ?1",
                &[DbValue::Text(tgt_name.clone())],
            )?;
            let src_id = src_rows
                .first()
                .map(|r| get_i64(&r[0]))
                .flatten()
                .unwrap_or(0);
            let tgt_id = tgt_rows
                .first()
                .map(|r| get_i64(&r[0]))
                .flatten()
                .unwrap_or(0);
            self.backend.execute(
                "INSERT INTO relations (source_id, target_id, relation_type, weight, observation_id, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                &[
                    DbValue::Integer(src_id),
                    DbValue::Integer(tgt_id),
                    DbValue::Text(format!("{:?}", rel_type)),
                    DbValue::Real(weight as f64),
                    DbValue::Integer(observation_id as i64),
                    DbValue::Integer(now),
                ],
            )?;
        }

        Ok(())
    }

    pub fn find_related_entities(
        &self,
        entity_name: &str,
        max_depth: u32,
        min_weight: f32,
    ) -> Result<Vec<(Entity, Relation, Entity)>, String> {
        let root_rows = self.backend.query(
            "SELECT id FROM entities WHERE name = ?1",
            &[DbValue::Text(entity_name.to_string())],
        )?;

        let root_id = root_rows.first().map(|r| get_i64(&r[0])).flatten();

        let root_id = match root_id {
            Some(id) => id,
            None => return Ok(vec![]),
        };

        let mut results: Vec<(Entity, Relation, Entity)> = Vec::new();
        let mut visited: std::collections::HashSet<i64> = std::collections::HashSet::new();
        let mut queue: std::collections::VecDeque<(i64, u32)> = std::collections::VecDeque::new();
        queue.push_back((root_id, 0));
        visited.insert(root_id);

        while let Some((current_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            let rows = self.backend.query(
                "SELECT r.id, r.source_id, r.target_id, r.relation_type, r.weight, r.observation_id, r.created_at,
                        e1.id, e1.name, e1.entity_type, e1.aliases, e1.mention_count, e1.first_seen, e1.last_seen,
                        e2.id, e2.name, e2.entity_type, e2.aliases, e2.mention_count, e2.first_seen, e2.last_seen
                 FROM relations r
                 JOIN entities e1 ON r.source_id = e1.id
                 JOIN entities e2 ON r.target_id = e2.id
                 WHERE (r.source_id = ?1 OR r.target_id = ?1) AND r.weight >= ?2",
                &[
                    DbValue::Integer(current_id),
                    DbValue::Real(min_weight as f64),
                ],
            )?;

            for row in &rows {
                let rel = Relation {
                    id: get_i64(&row[0]).unwrap_or(0) as u64,
                    source_id: get_i64(&row[1]).unwrap_or(0) as u64,
                    target_id: get_i64(&row[2]).unwrap_or(0) as u64,
                    relation_type: parse_relation_type(get_str(&row[3]).unwrap_or("")),
                    weight: get_f64(&row[4]).unwrap_or(0.0) as f32,
                    observation_id: get_i64(&row[5]).unwrap_or(0) as u64,
                    created_at: crate::domain::types::Timestamp(get_i64(&row[6]).unwrap_or(0)),
                };
                let src = Entity {
                    id: get_i64(&row[7]).unwrap_or(0) as u64,
                    name: get_str(&row[8]).unwrap_or("").to_string(),
                    entity_type: parse_entity_type(get_str(&row[9]).unwrap_or("")),
                    aliases: get_str(&row[10])
                        .unwrap_or("")
                        .split(',')
                        .map(|s| s.to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                    embedding: vec![],
                    mention_count: get_i64(&row[11]).unwrap_or(0) as u64,
                    first_seen: crate::domain::types::Timestamp(get_i64(&row[12]).unwrap_or(0)),
                    last_seen: crate::domain::types::Timestamp(get_i64(&row[13]).unwrap_or(0)),
                };
                let tgt = Entity {
                    id: get_i64(&row[14]).unwrap_or(0) as u64,
                    name: get_str(&row[15]).unwrap_or("").to_string(),
                    entity_type: parse_entity_type(get_str(&row[16]).unwrap_or("")),
                    aliases: get_str(&row[17])
                        .unwrap_or("")
                        .split(',')
                        .map(|s| s.to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                    embedding: vec![],
                    mention_count: get_i64(&row[18]).unwrap_or(0) as u64,
                    first_seen: crate::domain::types::Timestamp(get_i64(&row[19]).unwrap_or(0)),
                    last_seen: crate::domain::types::Timestamp(get_i64(&row[20]).unwrap_or(0)),
                };
                let neighbor_id = if rel.source_id == current_id as u64 {
                    rel.target_id as i64
                } else {
                    rel.source_id as i64
                };
                results.push((src, rel, tgt));

                if visited.insert(neighbor_id) {
                    queue.push_back((neighbor_id, depth + 1));
                }
            }
        }

        Ok(results)
    }

    pub fn entity_search(
        &self,
        query: &str,
        entity_type: Option<EntityType>,
    ) -> Result<Vec<Entity>, String> {
        let like = format!("%{}%", query);

        let (sql, params) = if let Some(ref et) = entity_type {
            (
                format!(
                    "SELECT id, name, entity_type, aliases, mention_count, first_seen, last_seen
                      FROM entities WHERE (name LIKE ?1 OR aliases LIKE ?1) AND entity_type = ?2"
                ),
                vec![DbValue::Text(like), DbValue::Text(format!("{:?}", et))],
            )
        } else {
            (
                format!(
                    "SELECT id, name, entity_type, aliases, mention_count, first_seen, last_seen
                      FROM entities WHERE name LIKE ?1 OR aliases LIKE ?1"
                ),
                vec![DbValue::Text(like)],
            )
        };

        let rows = self.backend.query(&sql, &params)?;
        let mut results = Vec::new();
        for row in &rows {
            results.push(Entity {
                id: get_i64(&row[0]).unwrap_or(0) as u64,
                name: get_str(&row[1]).unwrap_or("").to_string(),
                entity_type: parse_entity_type(get_str(&row[2]).unwrap_or("")),
                aliases: get_str(&row[3])
                    .unwrap_or("")
                    .split(',')
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
                embedding: vec![],
                mention_count: get_i64(&row[4]).unwrap_or(0) as u64,
                first_seen: crate::domain::types::Timestamp(get_i64(&row[5]).unwrap_or(0)),
                last_seen: crate::domain::types::Timestamp(get_i64(&row[6]).unwrap_or(0)),
            });
        }
        Ok(results)
    }
}

pub fn merge_chunks(observation_id: u64, db: &Database) -> Option<Observation> {
    let rows = db
        .backend
        .query(
            "SELECT content FROM chunks WHERE observation_id = ?1 ORDER BY seq ASC",
            &[DbValue::Integer(observation_id as i64)],
        )
        .ok()?;

    let mut full_content = String::new();
    for row in &rows {
        if let Some(content) = get_str(&row[0]) {
            full_content.push_str(content);
            full_content.push('\n');
        }
    }
    if full_content.is_empty() {
        return None;
    }

    let obs_rows = db.backend.query(
        "SELECT id, session_id, title, summary, content, project, tags, created_at, obs_type, importance, token_count, access_count
         FROM observations WHERE id = ?1",
        &[DbValue::Integer(observation_id as i64)],
    ).ok()?;

    let row = obs_rows.first()?;
    row_to_observation(row).ok().map(|mut obs| {
        obs.content = full_content;
        obs
    })
}

pub(crate) fn get_i64(v: &DbValue) -> Option<i64> {
    match v {
        DbValue::Integer(i) => Some(*i),
        _ => None,
    }
}

pub(crate) fn get_f64(v: &DbValue) -> Option<f64> {
    match v {
        DbValue::Real(f) => Some(*f),
        DbValue::Integer(i) => Some(*i as f64),
        _ => None,
    }
}

pub fn get_str<'a>(v: &'a DbValue) -> Option<&'a str> {
    match v {
        DbValue::Text(s) => Some(s.as_str()),
        _ => None,
    }
}

pub(crate) fn get_blob<'a>(v: &'a DbValue) -> Option<&'a [u8]> {
    match v {
        DbValue::Blob(b) => Some(b.as_slice()),
        _ => None,
    }
}

fn row_to_observation(row: &[DbValue]) -> Result<Observation, String> {
    Ok(Observation {
        id: ObservationId(get_i64(&row[0]).unwrap_or(0)),
        session_id: get_str(&row[1]).unwrap_or("").to_string(),
        title: get_str(&row[2]).unwrap_or("").to_string(),
        summary: get_str(&row[3]).unwrap_or("").to_string(),
        content: get_str(&row[4]).unwrap_or("").to_string(),
        project: get_str(&row[5])
            .map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            })
            .unwrap_or(None),
        tags: get_str(&row[6])
            .unwrap_or("")
            .split(',')
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        created_at: crate::domain::types::Timestamp(get_i64(&row[7]).unwrap_or(0)),
        observation_type: parse_obs_type(get_str(&row[8]).unwrap_or("")),
        importance: get_f64(&row[9]).unwrap_or(0.0) as f32,
        token_count: get_i64(&row[10]).unwrap_or(0) as u32,
        access_count: get_i64(&row[11]).unwrap_or(0) as u64,
    })
}

fn parse_obs_type(s: &str) -> ObservationType {
    match s {
        "Memory" => ObservationType::Memory,
        "Event" => ObservationType::Event,
        "Log" => ObservationType::Log,
        "Note" => ObservationType::Note,
        "Learning" => ObservationType::Learning,
        "Decision" => ObservationType::Decision,
        "Manual" => ObservationType::Manual,
        "ToolUse" => ObservationType::ToolUse,
        "Search" => ObservationType::Search,
        "FileChange" => ObservationType::FileChange,
        "Command" => ObservationType::Command,
        "Pattern" => ObservationType::Pattern,
        "Config" => ObservationType::Config,
        "Bugfix" => ObservationType::Bugfix,
        "Architecture" => ObservationType::Architecture,
        "Discovery" => ObservationType::Discovery,
        _ => {
            eprintln!(
                "[synapsis-core] Warning: unknown ObservationType '{}', defaulting to Note",
                s
            );
            ObservationType::Note
        }
    }
}

fn parse_entity_type(s: &str) -> EntityType {
    match s {
        "Person" => EntityType::Person,
        "Organization" => EntityType::Organization,
        "Location" => EntityType::Location,
        "Concept" => EntityType::Concept,
        "Technology" => EntityType::Technology,
        "Project" => EntityType::Project,
        "Language" => EntityType::Language,
        "Tool" => EntityType::Tool,
        "Protocol" => EntityType::Protocol,
        "Topic" => EntityType::Topic,
        _ => EntityType::Concept,
    }
}

fn parse_relation_type(s: &str) -> RelationType {
    match s {
        "Mentions" => RelationType::Mentions,
        "Depends" => RelationType::Depends,
        "Implements" => RelationType::Implements,
        "Extends" => RelationType::Extends,
        "Uses" => RelationType::Uses,
        "PartOf" => RelationType::PartOf,
        "Related" => RelationType::Related,
        "Creates" => RelationType::Creates,
        "Modifies" => RelationType::Modifies,
        "Similar" => RelationType::Similar,
        "Opposite" => RelationType::Opposite,
        "Specializes" => RelationType::Specializes,
        "Example" => RelationType::Example,
        _ => RelationType::Related,
    }
}

fn text_match_score(query: &str, title: &str, content: &str) -> f64 {
    if query.is_empty() {
        return 1.0;
    }
    let q = query.to_lowercase();
    let mut score = 0.0;
    let tl = title.to_lowercase();
    let cl = content.to_lowercase();
    if tl.contains(&q) {
        score += 10.0;
    }
    if tl.starts_with(&q) {
        score += 5.0;
    }
    if cl.contains(&q) {
        score += 3.0;
    }
    for word in q.split_whitespace() {
        if tl.contains(word) {
            score += 1.0;
        }
        score += cl.matches(word).count() as f64 * 0.5;
    }
    score
}

fn serialize_embedding(embedding: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(embedding.len() * 4);
    for &val in embedding {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    bytes
}

fn deserialize_embedding(blob: &[u8]) -> Option<Vec<f32>> {
    if blob.len() % 4 != 0 {
        return None;
    }
    let count = blob.len() / 4;
    let mut vec = Vec::with_capacity(count);
    for i in 0..count {
        let chunk = &blob[i * 4..(i + 1) * 4];
        vec.push(f32::from_le_bytes(chunk.try_into().ok()?));
    }
    Some(vec)
}
