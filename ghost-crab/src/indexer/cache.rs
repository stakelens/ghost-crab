use super::error::{Error, Result};
use rocksdb::DB;

pub fn load_cache(network: &str) -> Result<DB> {
    let current_dir = std::env::current_dir().map_err(|e| Error::CacheFileNotFound(e))?;
    let cache_path = current_dir.join("cache").join(network);
    let db = DB::open_default(cache_path).map_err(|e| Error::DB(e))?;

    Ok(db)
}
