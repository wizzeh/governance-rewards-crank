use anchor_client::ClientError;
use log::{debug, error, warn};

#[derive(Debug)]
pub enum Failure<E> {
    Fatal(E),
    PossibleDegradation(E),
    Skip,
}

impl From<ClientError> for Failure<ClientError> {
    fn from(err: ClientError) -> Self {
        match err {
            ClientError::AccountNotFound => Self::Skip,
            ClientError::AnchorError(_) => Self::Fatal(err),
            ClientError::ProgramError(_) => Self::Fatal(err),
            ClientError::SolanaClientError(_) => Self::PossibleDegradation(err),
            ClientError::SolanaClientPubsubError(_) => Self::PossibleDegradation(err),
            ClientError::LogParseError(_) => Self::Fatal(err),
        }
    }
}

impl<E> Failure<E> {
    pub fn must_succeed<T>(result: Result<T, E>) -> Result<T, Failure<E>> {
        result.map_err(|err| Failure::Fatal(err))
    }
}

impl Failure<ClientError> {
    pub fn assess<T>(result: Result<T, ClientError>) -> Result<usize, Failure<ClientError>> {
        if let Err(err) = result {
            match Failure::from(err) {
                Failure::Fatal(e) => {
                    error!("Encountered fatal error: {:?}", e);
                    Err(Failure::Fatal(e))
                }
                Failure::PossibleDegradation(e) => {
                    warn!("Failed to execute noncritical instruction -- possible sign of network degradation: {:?}", e);
                    Ok(1)
                }
                Failure::Skip => {
                    debug!("Encountered nonfatal error -- skipping.");
                    Ok(0)
                }
            }
        } else {
            Ok(0)
        }
    }
}
