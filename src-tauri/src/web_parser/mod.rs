// Web parser module — fetches and parses d2core.com build links into BuildPlan structs
// via direct HTTP POST to the Tencent CloudBase API.

mod error;
mod extract;
mod client;
mod parse;

pub use error::ParserError;
pub use extract::extract_build_id;
pub use client::D2CoreClient;
