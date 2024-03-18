use crate::error::*;
use anyhow::Context;
use std::path::Path;

pub(crate) struct SledKVDB {
    db_handle: ::sled::Db,
}

impl SledKVDB {
    pub fn new<T>(location: T, prefixes: Vec<&'static str>) -> Result<Self>
    where
        T: AsRef<Path>,
    {
        let db_handle = sled::open(location)?;

        for prefix in prefixes {
            // Make sure all prefixes exist
            let _ = db_handle.open_tree(prefix)?;
        }

        Ok(Self { db_handle })
    }

    fn get_tree(&self, prefix: &str) -> Result<sled::Tree> {
        self.db_handle
            .open_tree(prefix)
            .context("Failed to open sled tree")
    }

    pub fn get<T>(&self, prefix: &'static str, key: T) -> Result<Option<Vec<u8>>>
    where
        T: AsRef<[u8]>,
    {
        let tree = self.get_tree(prefix)?;

        let result = tree.get(key.as_ref())?;

        Ok(result.map(|v| v.to_vec()))
    }
    pub fn get_all<T, Y>(&self, _keys: T) -> Result<Vec<Result<Option<Vec<u8>>>>>
    where
        T: Iterator<Item = (&'static str, Y)>,
        Y: AsRef<[u8]>,
    {
        todo!()
    }

    pub fn exists<T>(&self, prefix: &'static str, key: T) -> Result<bool>
    where
        T: AsRef<[u8]>,
    {
        let tree = self.get_tree(prefix)?;

        Ok(tree.contains_key(key.as_ref())?)
    }

    pub fn set<T, Y>(&self, prefix: &'static str, key: T, data: Y) -> Result<()>
    where
        T: AsRef<[u8]>,
        Y: AsRef<[u8]>,
    {
        let tree = self.get_tree(prefix)?;

        tree.insert(key.as_ref(), data.as_ref())
            .context(format!("Failed to set key in prefix {:?}", prefix))
            .map(|_| ())
    }
    pub fn set_all<T, Y, Z>(&self, _prefix: &'static str, _values: T) -> Result<()>
    where
        T: Iterator<Item = (Y, Z)>,
        Y: AsRef<[u8]>,
        Z: AsRef<[u8]>,
    {
        todo!()
    }

    pub fn erase<T>(&self, prefix: &'static str, key: T) -> Result<()>
    where
        T: AsRef<[u8]>,
    {
        let tree = self.get_tree(prefix)?;

        tree.remove(key.as_ref())
            .context(format!("Failed to erase key in prefix {:?}", prefix))
            .map(|_| ())
    }

    /// Delete a set of keys
    /// Accepts an [`&[&[u8]]`], in any possible form, as long as it can be dereferenced
    /// all the way to the intended target.
    pub fn erase_keys<T, Y>(&self, prefix: &'static str, keys: T) -> Result<()>
    where
        T: Iterator<Item = Y>,
        Y: AsRef<[u8]>,
    {
        let tree = self.get_tree(prefix)?;

        let mut batch = ::sled::Batch::default();

        for key in keys {
            batch.remove(key.as_ref());
        }

        tree.apply_batch(batch)
            .context(format!("Failed to erase keys in prefix {:?}", prefix))
    }

    pub fn erase_range<T>(&self, prefix: &'static str, start: T, end: T) -> Result<()>
    where
        T: AsRef<[u8]>,
    {
        let tree = self.get_tree(prefix)?;

        let mut batch = sled::Batch::default();

        tree.range(start.as_ref()..end.as_ref()).for_each(|r| {
            let (key, _) = r.unwrap();

            batch.remove(key.as_ref());
        });

        tree.apply_batch(batch)
            .context(format!("Failed to erase range in prefix {:?}", prefix))
    }

    pub fn iter_range<T, Y>(
        &self,
        _prefix: &'static str,
        _start: Option<T>,
        _end: Option<Y>,
    ) -> Result<Box<dyn Iterator<Item = Result<(Box<[u8]>, Box<[u8]>)>> + '_>>
    where
        T: AsRef<[u8]>,
        Y: AsRef<[u8]>,
    {
        todo!()
    }
}
