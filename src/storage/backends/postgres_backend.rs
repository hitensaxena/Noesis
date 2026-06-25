use async_trait::async_trait;
use anyhow::Result;
use serde_json::Value;
use tracing;

use super::super::store::Storage;

/// Postgres storage backend.
///
/// Connects to the existing curlyos-core Postgres container (:54321).
/// Uses a simple key-value table (namespace, key, value) for field state persistence.
/// The curlyos-core schema is NOT modified — Noesis uses its own table namespace.
pub struct PostgresBackend {
    pool: deadpool_postgres::Pool,
}

impl std::fmt::Debug for PostgresBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresBackend").finish()
    }
}

impl PostgresBackend {
    /// Connect to Postgres and ensure the noesis_kv table exists.
    ///
    /// # Arguments
    /// * `url` - Postgres URL (e.g., "host=127.0.0.1 port=54321 dbname=curlyos user=curlyos password=...")
    pub async fn connect(config: &tokio_postgres::Config) -> Result<Self> {
        let pool = deadpool_postgres::Manager::new(
            config.clone(),
            tokio_postgres::NoTls,
        );
        let pool = deadpool_postgres::Pool::builder(pool)
            .max_size(4)
            .build()
            .unwrap();

        // Create the noesis namespace table if it doesn't exist
        let client = pool.get().await?;
        client
            .batch_execute(
                "CREATE TABLE IF NOT EXISTS noesis_kv (
                    namespace TEXT NOT NULL,
                    key TEXT NOT NULL,
                    value JSONB NOT NULL,
                    updated_at TIMESTAMPTZ DEFAULT NOW(),
                    PRIMARY KEY (namespace, key)
                );",
            )
            .await?;
        tracing::info!("[PostgresBackend] connected and schema ready");
        Ok(Self { pool })
    }
}

#[async_trait]
impl Storage for PostgresBackend {
    async fn get(&self, namespace: &str, key: &str) -> Result<Option<Value>> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT value FROM noesis_kv WHERE namespace = $1 AND key = $2",
                &[&namespace.to_string(), &key.to_string()],
            )
            .await?;
        if let Some(row) = rows.first() {
            let val: serde_json::Value = row.get(0);
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    async fn set(&self, namespace: &str, key: &str, value: Value) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO noesis_kv (namespace, key, value, updated_at)
                 VALUES ($1, $2, $3, NOW())
                 ON CONFLICT (namespace, key)
                 DO UPDATE SET value = $3, updated_at = NOW()",
                &[&namespace.to_string(), &key.to_string(), &value],
            )
            .await?;
        Ok(())
    }

    async fn delete(&self, namespace: &str, key: &str) -> Result<()> {
        let client = self.pool.get().await?;
        client
            .execute(
                "DELETE FROM noesis_kv WHERE namespace = $1 AND key = $2",
                &[&namespace.to_string(), &key.to_string()],
            )
            .await?;
        Ok(())
    }

    async fn list(&self, namespace: &str) -> Result<Vec<String>> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT key FROM noesis_kv WHERE namespace = $1 ORDER BY key",
                &[&namespace.to_string()],
            )
            .await?;
        Ok(rows.iter().map(|r| r.get(0)).collect())
    }
}
