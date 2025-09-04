use miden_client::store::StoreError;

pub(crate) trait SqlResultExt<T> {
    fn as_store_error(self) -> Result<T, StoreError>;
}

impl<T> SqlResultExt<T> for Result<T, rusqlite::Error> {
    fn as_store_error(self) -> Result<T, StoreError> {
        self.map_err(|value| match value {
            rusqlite::Error::FromSqlConversionFailure(..)
            | rusqlite::Error::IntegralValueOutOfRange(..)
            | rusqlite::Error::InvalidColumnIndex(_)
            | rusqlite::Error::InvalidColumnType(..) => StoreError::ParsingError(value.to_string()),
            rusqlite::Error::InvalidParameterName(_)
            | rusqlite::Error::InvalidColumnName(_)
            | rusqlite::Error::StatementChangedRows(_)
            | rusqlite::Error::ExecuteReturnedResults
            | rusqlite::Error::InvalidQuery
            | rusqlite::Error::MultipleStatement
            | rusqlite::Error::InvalidParameterCount(..)
            | rusqlite::Error::QueryReturnedNoRows => StoreError::QueryError(value.to_string()),
            _ => StoreError::DatabaseError(value.to_string()),
        })
    }
}
