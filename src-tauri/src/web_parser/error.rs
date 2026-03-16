use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("链接无效，请粘贴完整的 d2core.com 构建链接")]
    InvalidUrl(String),

    #[error("无法连接到 d2core API，请检查网络后重试")]
    NetworkError(#[from] reqwest::Error),

    #[error("d2core API 返回错误: {code} - {message}")]
    ApiError { code: String, message: String },

    #[error("构建不存在或已过期，请检查链接")]
    BuildNotFound(String),

    #[error("构建不存在或已过期，请检查链接")]
    BuildDeleted(String),

    #[error("d2core API 已变更，请更新应用")]
    ParseError(String),
}
