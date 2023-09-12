use async_trait::async_trait;
use sqlx;

use super::{Database, Error};

/// A database entity.
#[async_trait]
pub trait Entity:
    for<'a> sqlx::FromRow<'a, <Database as sqlx::Database>::Row> + Unpin
{
    /// The type of the key value.
    type Key: Sync + for<'a> sqlx::Encode<'a, Database> + sqlx::Type<Database>;

    /// A description of an entity.
    ///
    /// This is not necessarily complete, and it does not contain a key.
    type Description;

    /// The SQL statement used to insert an item of this kind.
    const CREATE: &'static str;

    /// The SQL statement used to read a single item of this kind.
    const READ: &'static str;

    /// The SQL statement used to update an item of this kind.
    const UPDATE: &'static str;

    /// The SQL statement used to delete an item of this kind.
    const DELETE: &'static str;

    /// Inserts this item to the database.
    ///
    /// # Arguments
    /// *  `e` - The database executor.
    async fn create<'a, E>(&self, e: E) -> Result<(), Error>
    where
        E: ::sqlx::Executor<'a, Database = Database>;

    /// Loads an item of this kind from the database.
    ///
    /// If no item corresponding to the keys exists, `Ok(None)` is
    /// returned.
    ///
    /// # Arguments
    /// *  `e` - The database executor.
    async fn read<'a, E>(e: E, key: &Self::Key) -> Result<Option<Self>, Error>
    where
        E: ::sqlx::Executor<'a, Database = Database>,
    {
        sqlx::query_as(Self::READ).bind(key).fetch_optional(e).await
    }

    /// Updates this item in the database.
    ///
    /// # Arguments
    /// *  `e` - The database executor.
    async fn update<'a, E>(&self, e: E) -> Result<(), Error>
    where
        E: ::sqlx::Executor<'a, Database = Database>;

    /// Deletes this item from the database.
    ///
    /// # Arguments
    /// *  `e` - The database executor.
    async fn delete<'a, E>(&self, e: E) -> Result<(), Error>
    where
        E: sqlx::Executor<'a, Database = Database>,
    {
        let count = ::sqlx::query(Self::DELETE)
            .bind(self.key())
            .execute(e)
            .await?
            .rows_affected();
        if count > 0 {
            Ok(())
        } else {
            Err(Error::RowNotFound)
        }
    }

    /// Merges a description into this entity.
    ///
    /// All values present in the description are set on this item.
    ///
    /// # Arguments
    /// *  `description` - The description to merge.
    fn merge(self, description: Self::Description) -> Self;

    /// The key of this item.
    fn key(&self) -> &Self::Key;
}
