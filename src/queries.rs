use mysql_async::prelude::*;
use std::error::Error;

/// # Errors
/// return Err if can't get the `wsrep_local_state`
pub async fn state(pool: mysql_async::Pool) -> Result<u8, Box<dyn Error>> {
    let conn = pool.get_conn().await?;
    let (_, row) = conn
        .first::<_, (String, u8)>("SHOW STATUS LIKE 'wsrep_local_state'")
        .await?;
    let (_, value) = row.ok_or("expected a row")?;
    Ok(value)
}
