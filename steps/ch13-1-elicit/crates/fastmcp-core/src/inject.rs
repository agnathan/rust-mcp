//! Dependencies the client does not send.
//!
//! `Dep<T>` holds a `T` and a `PhantomData<fn() -> T>`. The marker makes the
//! struct invariant in `T` and keeps `T` in the type even when you are
//! reading the signature. `FromContext` builds a `Dep` from the server's
//! map. `Extract` is what a tool asks for: one dependency, or a tuple of them.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

/// Values registered with `FastMcp::provide`, keyed by type.
#[derive(Clone, Default)]
pub struct InjectCtx {
    values: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl InjectCtx {
    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) {
        let mut values = (*self.values).clone();
        values.insert(TypeId::of::<T>(), Arc::new(value));
        self.values = Arc::new(values);
    }

    fn get<T: Send + Sync + 'static>(&self) -> Result<&T, String> {
        self.values
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
            .ok_or_else(|| format!("dependency not provided: {}", std::any::type_name::<T>()))
    }
}

/// A dependency cloned out of [`InjectCtx`].
pub struct Dep<T> {
    value: T,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Dep<T> {
    pub fn get(&self) -> &T {
        &self.value
    }
}

/// Build `Self` from the server's dependency map.
pub trait FromContext: Sized {
    fn from_context(ctx: &InjectCtx) -> Result<Self, String>;
}

impl<T: Clone + Send + Sync + 'static> FromContext for Dep<T> {
    fn from_context(ctx: &InjectCtx) -> Result<Self, String> {
        Ok(Self {
            value: ctx.get::<T>()?.clone(),
            _marker: PhantomData,
        })
    }
}

/// What `tool_inject` pulls in before it calls the function.
pub trait Extract: Sized {
    fn extract(ctx: &InjectCtx) -> Result<Self, String>;
}

impl<T: FromContext> Extract for T {
    fn extract(ctx: &InjectCtx) -> Result<Self, String> {
        T::from_context(ctx)
    }
}

impl<A: Extract, B: Extract> Extract for (A, B) {
    fn extract(ctx: &InjectCtx) -> Result<Self, String> {
        Ok((A::extract(ctx)?, B::extract(ctx)?))
    }
}
