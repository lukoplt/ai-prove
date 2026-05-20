use crate::error::AppResult;
use keyring::Entry;

const SERVICE: &str = "cz.druhynazor.app";

pub fn set_api_key(account: &str, secret: &str) -> AppResult<()> {
    let entry = Entry::new(SERVICE, account)?;
    entry.set_password(secret)?;
    Ok(())
}

pub fn get_api_key(account: &str) -> AppResult<Option<String>> {
    let entry = Entry::new(SERVICE, account)?;
    match entry.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub fn clear_api_key(account: &str) -> AppResult<()> {
    let entry = Entry::new(SERVICE, account)?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppError;

    fn unique_account() -> String {
        format!("test-{}", uuid::Uuid::now_v7())
    }

    fn keychain_unavailable(error: &AppError) -> bool {
        let message = error.to_string();
        message.contains("-60008") || message.contains("authorization")
    }

    fn unwrap_or_skip<T>(result: AppResult<T>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) if keychain_unavailable(&error) => {
                eprintln!("skipping keychain test: {error}");
                None
            }
            Err(error) => panic!("{error}"),
        }
    }

    #[test]
    fn set_then_get_roundtrip() {
        let account = unique_account();
        if unwrap_or_skip(set_api_key(&account, "secret-value-123")).is_none() {
            return;
        }
        let Some(got) = unwrap_or_skip(get_api_key(&account)) else {
            return;
        };
        assert_eq!(got.as_deref(), Some("secret-value-123"));
        let _ = unwrap_or_skip(clear_api_key(&account));
    }

    #[test]
    fn get_missing_returns_none() {
        let account = unique_account();
        let got = get_api_key(&account).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn clear_is_idempotent() {
        let account = unique_account();
        if unwrap_or_skip(clear_api_key(&account)).is_none() {
            return;
        }
        let _ = unwrap_or_skip(clear_api_key(&account));
    }

    #[test]
    fn overwrite_replaces_value() {
        let account = unique_account();
        if unwrap_or_skip(set_api_key(&account, "first")).is_none() {
            return;
        }
        if unwrap_or_skip(set_api_key(&account, "second")).is_none() {
            return;
        }
        let Some(got) = unwrap_or_skip(get_api_key(&account)) else {
            return;
        };
        assert_eq!(got.as_deref(), Some("second"));
        let _ = unwrap_or_skip(clear_api_key(&account));
    }
}
