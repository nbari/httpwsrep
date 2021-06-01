use anyhow::{Context, Result};
use mysql_async::prelude::*;

/// # Errors
/// return Err if can't get the `wsrep_local_state`
pub async fn state(pool: mysql_async::Pool) -> Result<u8> {
    let conn = pool.get_conn().await?;
    let row: Option<(String, u8)> = "SHOW STATUS LIKE 'wsrep_local_state'".first(conn).await?;
    let (_, value) = row.with_context(|| "could not get wsrep_local_state")?;
    Ok(value)
}
