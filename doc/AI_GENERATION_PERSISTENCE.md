# AI generation persistence

**AI generations are expensive, non-reproducible assets. Never discard them.**

Every call to an LLM costs real money and produces unique output that cannot be exactly reproduced. Treat generations like database records — assign an ID, persist immediately, and make them retrievable.

## Core Rules

1. **Generate an ID before the LLM call** — use `nanoid::nanoid!()` or `uuid::Uuid::new_v4()`
2. **Persist every generation** — text and metadata to local SQLite, images to filesystem
3. **Make every generation addressable** — file path pattern: `generations/[id].json`
4. **Track metadata** — model name, token usage, estimated cost, timestamp
5. **Never process without saving** — if the app crashes, the generation must survive

## Generate-Then-Save Pattern

The standard flow for AI features: create the record first, then update with results.

```rust
// src/generation.rs
use nanoid::nanoid;
use rusqlite::{params, Connection};
use anyhow::{Context, Result};

pub struct Generation {
    pub id: String,
    pub prompt: String,
    pub model: String,
    pub status: String,
    pub created_at: i64,
}

pub fn create_generation(conn: &Connection, prompt: &str, model: &str) -> Result<String> {
    let id = nanoid!(10); // Short ID suitable for filenames
    
    // Create the record BEFORE generation starts
    conn.execute(
        "INSERT INTO generations (id, prompt, model, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![&id, prompt, model, "pending", chrono::Utc::now().timestamp()],
    ).context("Failed to insert generation record")?;
    
    Ok(id)
}

pub fn complete_generation(
    conn: &Connection,
    id: &str,
    result: &str,
    token_usage: &TokenUsage,
) -> Result<()> {
    let estimated_cost_cents = estimate_cost(&token_usage.model, token_usage);
    
    conn.execute(
        "UPDATE generations 
         SET result = ?1, status = ?2, token_usage = ?3, 
             estimated_cost_cents = ?4, completed_at = ?5
         WHERE id = ?6",
        params![
            result,
            "complete",
            serde_json::to_string(token_usage)?,
            estimated_cost_cents,
            chrono::Utc::now().timestamp(),
            id
        ],
    ).context("Failed to update generation record")?;
    
    Ok(())
}
```

## Persistence Schema

```rust
// migrations/001_generations.sql
CREATE TABLE generations (
    id TEXT PRIMARY KEY,
    model TEXT NOT NULL,
    prompt TEXT,
    result TEXT,
    image_paths TEXT,        -- JSON array of file paths
    token_usage TEXT,        -- JSON: {"prompt_tokens": N, "completion_tokens": M}
    estimated_cost_cents INTEGER,
    status TEXT DEFAULT 'pending',  -- pending | complete | error
    created_at INTEGER NOT NULL,    -- Unix timestamp
    completed_at INTEGER
);

CREATE INDEX idx_generations_created_at ON generations(created_at);
CREATE INDEX idx_generations_status ON generations(status);
```

```rust
// src/db.rs
use rusqlite::Connection;
use anyhow::Result;

pub fn init_db(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(include_str!("../migrations/001_generations.sql"))?;
    Ok(conn)
}
```

## Storage Strategy

| Data Type                | Storage                      | Why                                    |
|--------------------------|------------------------------|----------------------------------------|
| Text, metadata, history  | SQLite via `rusqlite`        | Queryable, local-first, zero config    |
| Generated images & files | Local filesystem             | No network, permanent, user owns data  |
| Prompt dedup cache       | SQLite or in-memory LRU      | Fast lookup, survives restarts         |

## Image Persistence

Never hold generated images only in memory. Save to filesystem immediately:

```rust
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use anyhow::{Context, Result};

pub fn save_generation_images(
    generation_id: &str,
    images: &[ImageData],
    base_dir: &PathBuf,
) -> Result<Vec<PathBuf>> {
    let gen_dir = base_dir.join("generations").join(generation_id);
    fs::create_dir_all(&gen_dir)
        .with_context(|| format!("Failed to create dir: {}", gen_dir.display()))?;
    
    let mut paths = Vec::new();
    
    for (idx, img) in images.iter().enumerate() {
        let ext = match img.format {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Webp => "webp",
        };
        let path = gen_dir.join(format!("image_{}.{}", idx, ext));
        
        let mut file = File::create(&path)
            .with_context(|| format!("Failed to create file: {}", path.display()))?;
        file.write_all(&img.data)
            .with_context(|| format!("Failed to write image: {}", path.display()))?;
        
        paths.push(path);
    }
    
    Ok(paths)
}
```

## Cost Tracking

Extract usage from every generation and store it. This enables budgeting and optimization:

```rust
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub model: String,
}

pub fn estimate_cost(model: &str, usage: &TokenUsage) -> i64 {
    // Rough estimates in cents per 1K tokens
    let rates: std::collections::HashMap<&str, (f64, f64)> = [
        ("gpt-4", (3.0, 6.0)),      // (input, output) cents per 1K
        ("gpt-4-turbo", (1.0, 3.0)),
        ("gpt-3.5-turbo", (0.5, 1.5)),
        ("claude-3-opus", (1.5, 7.5)),
        ("claude-3-sonnet", (0.3, 1.5)),
    ].into_iter().collect();
    
    let (input_rate, output_rate) = rates.get(model).copied().unwrap_or((1.0, 2.0));
    
    let input_cost = (usage.prompt_tokens as f64 / 1000.0) * input_rate;
    let output_cost = (usage.completion_tokens as f64 / 1000.0) * output_rate;
    
    ((input_cost + output_cost) * 100.0).round() as i64 // Return cents
}

// After generation
let usage = TokenUsage {
    prompt_tokens: result.usage.prompt_tokens,
    completion_tokens: result.usage.completion_tokens,
    total_tokens: result.usage.total_tokens,
    model: model.to_string(),
};

complete_generation(&conn, &id, &result.text, &usage)?;
```

## Prompt Dedup / Caching

Avoid paying for identical generations. Cache by content hash:

```rust
use sha2::{Sha256, Digest};
use rusqlite::params;
use anyhow::Result;

fn hash_prompt(model: &str, prompt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(model.as_bytes());
    hasher.update(b":");
    hasher.update(prompt.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn check_cache(conn: &Connection, model: &str, prompt: &str) -> Result<Option<String>> {
    let hash = hash_prompt(model, prompt);
    
    let mut stmt = conn.prepare(
        "SELECT id FROM generations 
         WHERE prompt_hash = ?1 AND status = 'complete' 
         AND created_at > strftime('%s', 'now', '-1 hour')"
    )?;
    
    let id: Option<String> = stmt.query_row([&hash], |row| row.get(0)).ok();
    Ok(id)
}

// Before generating
if let Some(cached_id) = check_cache(&conn, model, prompt)? {
    return Ok(cached_id); // Return cached result
}

// After generation, store the hash
conn.execute(
    "UPDATE generations SET prompt_hash = ?1 WHERE id = ?2",
    params![hash_prompt(model, prompt), &id],
)?;
```

## Anti-Patterns

- **Processing without saving** — generation lost on crash. Always write to SQLite before starting and update after completion.
- **No ID assignment** — processing without a stable ID means no history tracking. Generate ID first.
- **Re-generating identical prompts** — check cache first. Same prompt + same model = same cost for no new value.
- **Images only in memory** — generated images held as `Vec<u8>` are lost on exit. Save to filesystem immediately.
- **Missing metadata** — always store model name, token counts, and timestamp. You need this for cost tracking and debugging.
- **No error persistence** — failed generations should be recorded with error status for debugging.
