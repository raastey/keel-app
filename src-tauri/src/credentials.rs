use crate::error::{KeelError, Result};

#[cfg(target_os = "macos")]
pub fn set(engine_id: &str, secret: &str) -> Result<()> {
    use security_framework::passwords::{delete_generic_password, set_generic_password};
    let _ = delete_generic_password("com.sixtwelve.keel", engine_id);
    set_generic_password("com.sixtwelve.keel", engine_id, secret.as_bytes())
        .map_err(|error| KeelError::Credential(error.to_string()))
}

#[cfg(target_os = "macos")]
pub fn get(engine_id: &str) -> Result<Option<String>> {
    use security_framework::passwords::get_generic_password;
    match get_generic_password("com.sixtwelve.keel", engine_id) {
        Ok(value) => String::from_utf8(value)
            .map(Some)
            .map_err(|error| KeelError::Credential(error.to_string())),
        Err(_) => Ok(None),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn set(_engine_id: &str, _secret: &str) -> Result<()> {
    Err(KeelError::Credential(
        "secure credential storage is not implemented on this platform".into(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn get(_engine_id: &str) -> Result<Option<String>> {
    Ok(None)
}
