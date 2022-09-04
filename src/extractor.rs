use async_trait::async_trait;
use axum::extract::rejection::TypedHeaderRejection;
use axum::extract::{FromRequest, RequestParts};
use axum::headers::{Header, HeaderMapExt};
use std::ops::Deref;

#[derive(Debug, Clone, Copy)]
pub struct OptionalHeader<T>(pub T);

#[async_trait]
impl<T, B> FromRequest<B> for OptionalHeader<T>
where
    T: Header,
    B: Send,
{
    type Rejection = TypedHeaderRejection;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        match req.headers().typed_try_get::<T>() {
            Ok(Some(value)) => Ok(Self(value)),
            Ok(None) => todo!(),
            Err(err) => todo!(),
        }
    }
}

impl<T> Deref for OptionalHeader<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
