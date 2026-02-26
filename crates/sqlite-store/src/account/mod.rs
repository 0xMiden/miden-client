// We use const QUERY: &str for SQL queries to increase readability. This style
// triggers this clippy lint error.
#![allow(clippy::items_after_statements)]

mod accounts;
mod helpers;
mod storage;
mod vault;
pub(crate) mod watched;

#[cfg(test)]
mod tests;
