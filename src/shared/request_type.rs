#[derive(Clone, Debug)]
pub enum RequestType {
    Query,
    Command,
    Macro,
    Unknown,
}