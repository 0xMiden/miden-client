#[napi(string_enum)]
pub enum NoteFilterType {
    All,
    Consumed,
    Committed,
    Expected,
    Processing,
    Unverified,
}
