use anchor_client::ClientError;

pub mod claim;
pub mod reclaim;
pub mod register;

fn filter_account_result<T>(result: Result<T, ClientError>) -> Option<Result<T, ClientError>> {
    match result {
        Err(ClientError::AccountNotFound) => None,
        t => Some(t),
    }
}
