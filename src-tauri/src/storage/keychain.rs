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
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_account() -> String {
        format!("test-{}", uuid::Uuid::now_v7())
    }

    #[test]
    fn set_then_get_roundtrip() {
        let account = unique_account();
        set_api_key(&account, "secret-value-123").unwrap();
        let got = get_api_key(&account).unwrap();
        assert_eq!(got.as_deref(), Some("secret-value-123"));
        clear_api_key(&account).unwrap();
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
        clear_api_key(&account).unwrap();
        clear_api_key(&account).unwrap();
    }

    #[test]
    fn overwrite_replaces_value() {
        let account = unique_account();
        set_api_key(&account, "first").unwrap();
        set_api_key(&account, "second").unwrap();
        let got = get_api_key(&account).unwrap();
        assert_eq!(got.as_deref(), Some("second"));
        clear_api_key(&account).unwrap();
    }
}
