use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type InitResult<'a, T> = Pin<Box<dyn Future<Output=Result<T, i32>> + Send + 'a>>;
pub type InitFunction<'a, T> = Box<dyn Fn(T) -> InitResult<'a, T> + Send + 'a>;
pub type WarmUpResult<'a> = Pin<Box<dyn Future<Output=()> + Send + 'a>>;
pub type WarmUpFunction<'a, T> = Box<dyn Fn(Arc::<T>) -> WarmUpResult<'a> + Send + 'a>;
pub type LoadResult<'a> = Pin<Box<dyn Future<Output=Result<(), i32>> + Send + 'a>>;
pub type LoadFunction<'a, T> = Box<dyn Fn(Arc::<T>) -> LoadResult<'a> + Send + 'a>;
pub type CleanUpResult<'a> = Pin<Box<dyn Future<Output=()> + Send + 'a>>;
pub type CleanUpFunction<'a, T> = Box<dyn Fn(T) -> CleanUpResult<'a> + Send + 'a>;