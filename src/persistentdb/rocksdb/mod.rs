use anyhow::{anyhow, Context};
use std::fmt::format;
use std::path::Path;

use crate::Err;
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, CompactOptions, DBWithThreadMode, Direction,
    IteratorMode, Options, SingleThreaded, WriteBatchWithTransaction, DB,
};

use crate::error::*;
use crate::persistentdb::{KeyValueEntry, PersStorage};

pub(crate) struct RocksKVDB {
    db: DBWithThreadMode<SingleThreaded>,
}

impl RocksKVDB {
    pub fn new<T>(db_location: T, prefixes: Vec<&'static str>) -> Result<Self>
    where
        T: AsRef<Path>,
    {
        let mut cfs = Vec::with_capacity(prefixes.len());

        for cf in prefixes {
            let cf_opts = Options::default();

            cfs.push(ColumnFamilyDescriptor::new(cf, cf_opts));
        }

        let mut db_opts = Options::default();
        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);

        let db = DB::open_cf_descriptors(&db_opts, db_location, cfs).unwrap();

        Ok(RocksKVDB { db })
    }

    fn get_handle(&self, prefix: &'static str) -> Result<&ColumnFamily> {
        let handle = self.db.cf_handle(prefix);

        if let Some(handle) = handle {
            Ok(handle)
        } else {
            Err!(PersStorage::NoPrefix(prefix))
        }
    }

    pub fn get<T>(&self, prefix: &'static str, key: T) -> Result<Option<Vec<u8>>>
    where
        T: AsRef<[u8]>,
    {
        let handle = self.get_handle(prefix)?;

        self.db
            .get_cf(handle, key)
            .with_context(|| format!("Failed to get for prefix {:?}", prefix))
    }

    pub fn get_all<T, Y>(&self, keys: T) -> Result<Vec<Result<Option<Vec<u8>>>>>
    where
        T: Iterator<Item = (&'static str, Y)>,
        Y: AsRef<[u8]>,
    {
        let final_keys: Result<Vec<_>> = keys
            .map(|(prefix, key)| {
                if let Ok(handle) = self.get_handle(prefix) {
                    Ok((handle, key))
                } else {
                    Err(anyhow!(""))
                }
            })
            .collect();

        Ok(self
            .db
            .multi_get_cf(final_keys?)
            .into_iter()
            .map(|r| {
                if let Ok(result) = r {
                    Ok(result)
                } else {
                    Err(anyhow!(""))
                }
            })
            .collect())
    }

    pub fn exists<T>(&self, prefix: &'static str, key: T) -> Result<bool>
    where
        T: AsRef<[u8]>,
    {
        let handle = self.get_handle(prefix)?;

        Ok(self.db.key_may_exist_cf(handle, key))
    }

    pub fn set<T, Y>(&self, prefix: &'static str, key: T, data: Y) -> Result<()>
    where
        T: AsRef<[u8]>,
        Y: AsRef<[u8]>,
    {
        let handle = self.get_handle(prefix)?;

        self.db
            .put_cf(handle, key, data)
            .context(format!("Failed to set in prefix {:?}", prefix))
    }

    pub fn set_all<T, Y, Z>(&self, prefix: &'static str, values: T) -> Result<()>
    where
        T: Iterator<Item = (Y, Z)>,
        Y: AsRef<[u8]>,
        Z: AsRef<[u8]>,
    {
        let handle = self.get_handle(prefix)?;

        let mut batch = WriteBatchWithTransaction::<false>::default();

        for (key, value) in values {
            batch.put_cf(handle, key, value)
        }

        self.db.write(batch).context(format!("Failed to set keys"))
    }

    pub fn erase<T>(&self, prefix: &'static str, key: T) -> Result<()>
    where
        T: AsRef<[u8]>,
    {
        let handle = self.get_handle(prefix)?;

        self.db
            .delete_cf(handle, key)
            .context(format!("Failed to erase key in prefix {:?}", prefix))
    }

    /// Delete a set of keys
    /// Accepts an [`&[&[u8]]`], in any possible form, as long as it can be dereferenced
    /// all the way to the intended target.
    pub fn erase_keys<T, Y>(&self, prefix: &'static str, keys: T) -> Result<()>
    where
        T: Iterator<Item = Y>,
        Y: AsRef<[u8]>,
    {
        let handle = self.get_handle(prefix)?;

        let mut batch = WriteBatchWithTransaction::<false>::default();

        for key in keys {
            batch.delete_cf(handle, key)
        }

        self.db
            .write(batch)
            .context(format!("Failed to erase in prefix {:?}", prefix))
    }

    pub fn erase_range<T>(&self, prefix: &'static str, start: T, end: T) -> Result<()>
    where
        T: AsRef<[u8]>,
    {
        let handle = self.get_handle(prefix)?;

        self.db
            .delete_range_cf(handle, start, end)
            .with_context(|| format!("Failed to erase in prefix {:?}", prefix))
    }

    pub fn compact_range<T, Y>(
        &self,
        prefix: &'static str,
        start: Option<T>,
        end: Option<Y>,
    ) -> Result<()>
    where
        T: AsRef<[u8]>,
        Y: AsRef<[u8]>,
    {
        let handle = self.get_handle(prefix)?;

        Ok(self
            .db
            .compact_range_cf_opt(handle, start, end, &CompactOptions::default()))
    }

    pub fn iter_range<T, Y>(
        &self,
        prefix: &'static str,
        start: Option<T>,
        end: Option<Y>,
    ) -> Result<Box<dyn Iterator<Item = Result<KeyValueEntry>> + '_>>
    where
        T: AsRef<[u8]>,
        Y: AsRef<[u8]>,
    {
        let handle = self.get_handle(prefix)?;

        let mut iterator = if let Some(start) = start {
            self.db.iterator_cf(
                handle,
                IteratorMode::From(start.as_ref(), Direction::Forward),
            )
        } else {
            self.db.iterator_cf(handle, IteratorMode::Start)
        };

        if let Some(end) = end {
            iterator.set_mode(IteratorMode::From(end.as_ref(), Direction::Reverse));
        }

        let mut bytes = vec![];

        for futures in iterator {
            bytes.push(Ok(futures?));
        }

        Ok(Box::new(bytes.into_iter()))
    }
}
