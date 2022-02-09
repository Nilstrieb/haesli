#[derive(Debug, thiserror::Error)]
pub enum TransError {
    #[error("{0}")]
    Invalid(#[from] ProtocolError),
    #[error("connection error: `{0}`")]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("fatal error")]
    Fatal,
    #[error("{0}")]
    ConException(#[from] ConException),
    #[error("{0}")]
    ChannelException(#[from] ChannelException),
    #[error("closing connection")]
    OtherCloseConnection,
}

#[derive(Debug, thiserror::Error)]
pub enum ConException {
    #[error("501 Frame error")]
    FrameError,
    #[error("503 Command invalid")]
    CommandInvalid,
    #[error("503 Syntax error")]
    SyntaxError,
    #[error("504 Channel error")]
    ChannelError,
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelException {}