use crate::{
    error::Error,
    env,
};

use std::fmt::Display;

use firestore::{self, FirestoreDbOptions};
use gcloud_sdk::{TokenSourceType, GCP_DEFAULT_SCOPES};
use serde::{Deserialize, Serialize};

const DEFAULT_MAX_RETRIES: usize = 3;
const DEFAULT_COLLECTION_PATH: &str = "";

pub struct FirestoreDb {
    inner: firestore::FirestoreDb,
    collection: String,
    collection_path: String,
}

impl FirestoreDb {
    pub async fn new(namespace: impl Display) -> Result<Self, Error> {
        let inner = firestore::FirestoreDb::with_options_token_source(
            FirestoreDbOptions {
                google_project_id: env::project_id(&namespace)?,
                max_retries: env::max_retries(
                    &namespace,
                    DEFAULT_MAX_RETRIES,
                )?,
                firebase_api_url: None,
            },
            env::scopes(&namespace, || GCP_DEFAULT_SCOPES.clone())?,
            TokenSourceType::Json(env::credentials(&namespace)?),
        )
            .await
            .map_err(|e| Error::Initialize(e))?;

        let collection = env::collection(&namespace)?;

        let collection_path = env::collection_path(
            &namespace,
            DEFAULT_COLLECTION_PATH,
            inner.get_documents_path(),
        )?;
        
        Ok(Self { inner, collection, collection_path })
    }

    pub async fn read<O>(
        &self,
        document_id: impl AsRef<str> + Send,
    ) -> Result<Option<O>, Error>
    where
        for<'de> O: Deserialize<'de> + Send,
    {
        self.inner
            .fluent()
            .select()
            .by_id_in(&self.collection)
            .parent(&self.collection_path)
            .obj::<O>()
            .one(document_id)
            .await
            .map_err(|e| Error::Read(e))
    }

    pub async fn write<O>(
        &self,
        document_id: impl AsRef<str> + Send,
        object: O,
    ) -> Result<(), Error>
    where
        for<'de> O: Deserialize<'de> + Serialize + Send + Sync,
    {
        self.inner
            .fluent()
            .update()
            .in_col(&self.collection)
            .document_id(document_id)
            .parent(&self.collection_path)
            .object(&object)
            .execute()
            .await
            .map_err(|e| Error::Write(e))
    }
}
