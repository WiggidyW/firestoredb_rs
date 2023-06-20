use crate::error::Error;

use std::fmt::Display;

use env_util;

const PROJECT_ID: &str = "FS_PROJECT_ID";
const CREDENTIALS: &str = "FS_CREDENTIALS";
const MAX_RETRIES: &str = "FS_MAX_RETRIES";
const COLLECTION_PATH: &str = "FS_COLLECTION_PATH";
const COLLECTION: &str = "FS_COLLECTION";
const SCOPES: &str = "FS_SCOPES";

pub fn project_id(namespace: impl Display) -> Result<String, Error> {
    let key = format!("{namespace}_{PROJECT_ID}");
    Ok(env_util::get(&key).required_checked()?.into_inner())
}

pub fn credentials(namespace: impl Display) -> Result<String, Error> {
    let key = format!("{namespace}_{CREDENTIALS}");
    Ok(env_util::get(&key).required_checked()?.into_inner())
}

pub fn collection(namespace: impl Display) -> Result<String, Error> {
    let key = format!("{namespace}_{COLLECTION}");
    Ok(env_util::get(&key).required_checked()?.into_inner())
}

pub fn collection_path(
    namespace: impl Display,
    default: &str,
    doc_path: &String,
) -> Result<String, Error> {
    let key = format!("{namespace}_{COLLECTION_PATH}");
    Ok(env_util::get(&key)
        .with_default_checked(default)?
        .then_fn_str_into(|s| format!("{doc_path}/{s}"))
        .into_inner()
    )
}

pub fn max_retries(
    namespace: impl Display,
    default: usize,
) -> Result<usize, Error> {
    let key = format!("{namespace}_{MAX_RETRIES}");
    Ok(match env_util::get(&key).optional_checked()? {
        Some(v) => v.then_try_fromstr_into()?.into_inner(),
        None => default,
    })
}

pub fn scopes(
    namespace: impl Display,
    default: fn() -> Vec<String>,
) -> Result<Vec<String>, Error> {
    let key = format!("{namespace}_{SCOPES}");
    Ok(match env_util::get(&key).optional_checked()? {
        Some(v) => v
            .then_fn_str_into(|s| s
                .split(',')
                .map(|s| s.trim())
                .map(|s| s.to_string())
                .collect()
            )
            .into_inner(),
        None => default(),
    })
}
