use async_trait::async_trait;
use std::future::Future;
use std::{convert::From, sync::Arc};

use crate::model::Model;

// https://doc.rust-lang.org/nomicon/hrtb.html
#[async_trait]
pub trait Extractor {
    // TODO Should be deleted
    fn extract<T>(self: Arc<Self>) -> Vec<T>
    where
        T: for<'a> From<regex::Captures<'a>> + Model;

    async fn extract_and<T, Fut>(
        &self,
        f: impl FnOnce(T) -> Fut + Send + Copy + 'async_trait,
    ) -> &Self
    where
        T: for<'a> From<regex::Captures<'a>> + Model + Send,
        Fut: Future<Output = ()> + Send;

    async fn extract_then<T, F, Fut>(&self, f: F) -> Vec<Fut>
    where
        T: for<'a> From<regex::Captures<'a>> + Model,
        F: Fn(T) -> Fut + Send,
        Fut: Future<Output = Option<T>> + Send;

    fn extract_fut<T, F, Fut>(self: Arc<Self>, f: F) -> Vec<Fut>
    where
        T: for<'a> From<regex::Captures<'a>> + Model + Send,
        F: Fn(T) -> Fut + Send + Sync,
        Fut: Future<Output = Option<T>> + Send;
}

#[async_trait]
impl Extractor for String {
    fn extract<T>(self: Arc<Self>) -> Vec<T>
    where
        T: for<'a> From<regex::Captures<'a>> + Model,
    {
        T::regex().captures_iter(&self).map(|c| c.into()).collect()
    }

    async fn extract_and<T, Fut>(
        &self,
        f: impl FnOnce(T) -> Fut + Send + Copy + 'async_trait,
    ) -> &Self
    where
        T: for<'a> From<regex::Captures<'a>> + Model + Send,
        Fut: Future<Output = ()> + Send,
    {
        for t in T::regex().captures_iter(self).map(|c| c.into()) {
            f(t).await;
        }
        self
    }

    async fn extract_then<T, F, Fut>(&self, f: F) -> Vec<Fut>
    where
        T: for<'a> From<regex::Captures<'a>> + Model,
        F: Fn(T) -> Fut + Send,
        Fut: Future<Output = Option<T>> + Send,
    {
        T::regex()
            .captures_iter(self)
            .map(|c| c.into())
            .map(|t| f(t))
            .collect()
    }

    fn extract_fut<T, F, Fut>(self: Arc<Self>, f: F) -> Vec<Fut>
    where
        T: for<'a> From<regex::Captures<'a>> + Model + Send,
        F: Fn(T) -> Fut + Send + Sync,
        Fut: futures::Future<Output = Option<T>> + Send,
    {
        T::regex()
            .captures_iter(&self)
            // No deifference parallel or no-parallel
            // .par_bridge()
            .map(|c| c.into())
            .map(|t| f(t))
            .collect()
    }
}
