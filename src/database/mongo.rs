use crate::model::Model;

use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::bson::Document;
use mongodb::options::FindOptions;
use mongodb::Cursor;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub async fn insert<M>(id: String, parent: String)
where
    M: Unpin + Send + Sync + DeserializeOwned + Serialize + Model + Clone,
{
    update(M::new(id, parent)).await;
}

pub async fn update<M>(mut model: M) -> Option<M>
where
    M: Unpin + Send + Sync + DeserializeOwned + Serialize + Model + Clone,
{
    let mut new = None;

    // Set options for update query
    let options = mongodb::options::UpdateOptions::builder()
        .upsert(true)
        .build();

    // Define collections
    let collection = super::get_db().await.collection::<M>(&M::ident());

    // Find document
    let cursor = collection.find_one(model.id_query(), None).await.unwrap();

    // Edit existed fields
    if let Some(doc) = cursor {
        model = model.merge(doc).await;
    } else {
        new = Some(model.clone());
    }

    let doc = mongodb::bson::to_document(&model).unwrap();
    let doc = doc! {"$set":doc};
    collection
        .update_one(model.id_query(), doc, options.clone())
        .await
        .unwrap();

    new
}

fn normalaize(
    filter: Option<String>,
    limit: Option<String>,
    sort: Option<String>,
) -> (Document, i64, Document) {
    let filter: Document =
        serde_json::from_str(&filter.unwrap_or("{}".to_string()).replace("'", "\"")).unwrap();
    let limit = limit.unwrap_or("0".to_string()).parse::<i64>().unwrap();
    let sort: Document =
        serde_json::from_str(&sort.unwrap_or("{}".to_string()).replace("'", "\"")).unwrap();
    (filter, limit, sort)
}

pub async fn find<M, D>(
    filter: Option<String>,
    limit: Option<String>,
    sort: Option<String>,
) -> Cursor<D>
where
    M: Unpin + Send + Sync + DeserializeOwned + Model,
    D: Unpin + Send + Sync + DeserializeOwned,
{
    let (filter, limit, sort) = normalaize(filter, limit, sort);
    let find_options = FindOptions::builder().limit(limit).sort(sort).build();
    let db = super::get_db().await;

    db.collection::<D>(&M::ident())
        .find(filter, find_options)
        .await
        .unwrap()
}
pub async fn find_as_vec<M>(
    filter: Option<String>,
    limit: Option<String>,
    sort: Option<String>,
) -> Vec<M>
where
    M: Unpin + Send + Sync + DeserializeOwned + Model,
{
    let cursor = find::<M, M>(filter, limit, sort).await;
    cursor.try_collect::<Vec<M>>().await.unwrap()
}

pub async fn find_as_string<M>(
    filter: Option<String>,
    limit: Option<String>,
    sort: Option<String>,
    field: Option<String>,
) -> Vec<String>
where
    M: Unpin + Send + Sync + DeserializeOwned + Model,
{
    let cursor = find::<M, Document>(filter, limit, sort).await;

    if let Some(field) = field {
        cursor
            .map_ok(|d| d.get_str(&field).unwrap().to_string())
            .try_collect::<Vec<String>>()
            .await
            .unwrap()
    } else {
        cursor
            .map_ok(|d| format!("{:#?}", d))
            .try_collect::<Vec<String>>()
            .await
            .unwrap()
    }
}
