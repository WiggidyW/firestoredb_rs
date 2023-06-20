use std::{error::Error as StdError, fmt::Display};

use env_util;

use firestore;

#[derive(Debug)]
pub enum Error {
    Initialize(firestore::errors::FirestoreError),
    Write(firestore::errors::FirestoreError),
    Read(firestore::errors::FirestoreError),
    Env(env_util::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Initialize(e) => write!(f, "Error initializing FirestoreDb: {}", e,),
            Error::Write(e) => write!(f, "Error Writing to Firestore: {}", e),
            Error::Read(e) => write!(f, "Error Reading from Firestore: {}", e),
            Error::Env(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Initialize(e) => Some(e),
            Error::Write(e) => Some(e),
            Error::Read(e) => Some(e),
            Error::Env(e) => Some(e),
        }
    }
}

impl From<env_util::Error> for Error {
    fn from(e: env_util::Error) -> Self {
        Error::Env(e)
    }
}
