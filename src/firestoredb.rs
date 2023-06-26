use crate::{env, error::Error};

use std::fmt::Display;

use firestore::{self, FirestoreDbOptions};
use gcloud_sdk::{TokenSourceType, GCP_DEFAULT_SCOPES};
use serde::{Deserialize, Serialize};

const DEFAULT_MAX_RETRIES: usize = 3;

pub struct FirestoreDb {
    inner: firestore::FirestoreDb,
    collection: String,
    collection_path: Option<String>,
}

impl FirestoreDb {
    pub async fn new(namespace: impl Display) -> Result<Self, Error> {
        let inner = firestore::FirestoreDb::with_options_token_source(
            FirestoreDbOptions {
                google_project_id: env::project_id(&namespace)?,
                max_retries: env::max_retries(&namespace, DEFAULT_MAX_RETRIES)?,
                firebase_api_url: None,
            },
            env::scopes(&namespace, || GCP_DEFAULT_SCOPES.clone())?,
            TokenSourceType::Json(env::credentials(&namespace)?),
        )
        .await
        .map_err(|e| Error::Initialize(e))?;

        let collection = env::collection(&namespace)?;

        let collection_path = match env::collection_path(&namespace)? {
            Some(v) if v.len() % 2 != 0 => return Err(Error::InvalidCollectionPath(v)),
            Some(v) => {
                let mut builder = inner
                    .parent_path::<&str>(v[v.len() - 2].as_ref(), v[v.len() - 1].as_ref())
                    .map_err(|e| Error::Initialize(e))?;
                if v.len() > 2 {
                    for tuple in v[..v.len() - 2].rchunks(2) {
                        builder = builder
                            .at::<&str>(tuple[0].as_ref(), tuple[1].as_ref())
                            .map_err(|e| Error::Initialize(e))?;
                    }
                }
                Some(builder.into())
            }
            None => None,
        };

        Ok(Self {
            inner,
            collection,
            collection_path,
        })
    }

    pub async fn read<O>(&self, document_id: impl AsRef<str> + Send) -> Result<Option<O>, Error>
    where
        for<'de> O: Deserialize<'de> + Send,
    {
        let query = self.inner.fluent().select().by_id_in(&self.collection);
        let query = match &self.collection_path {
            Some(path) => query.parent(path),
            None => query,
        };
        query
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
        let query = self
            .inner
            .fluent()
            .update()
            .in_col(&self.collection)
            .document_id(document_id);
        let query = match &self.collection_path {
            Some(path) => query.parent(path),
            None => query,
        };
        query
            .object(&object)
            .execute()
            .await
            .map_err(|e| Error::Write(e))
    }
}
