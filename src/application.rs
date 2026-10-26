//! Bind the Symfony application's PHP adapter once, during provider boot.

use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll};

use async_trait::async_trait;
use claviron::ephp::{
    PhpApplicationAdapter, PhpApplicationService, PhpEmbedRuntime,
};
use claviron::http::RouteResponse;
use claviron::runner::AppRequest;
use claviron::runtime::registry::{self, Bootable};
use claviron::runtime::{AppState, Component};
use http_body_util::BodyExt;
use tower::Service;

#[derive(Clone, Default)]
pub(crate) struct Symfony(Arc<OnceLock<PhpApplicationAdapter>>);

impl Service<AppRequest> for Symfony {
    type Response = RouteResponse;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<RouteResponse, Infallible>> + Send>>;

    fn poll_ready(
        &mut self,
        _: &mut Context<'_>,
    ) -> Poll<Result<(), Infallible>> {
        Poll::Ready(Ok(()))
    }

    fn call(
        &mut self,
        request: AppRequest,
    ) -> Self::Future {
        let php = Arc::clone(&self.0);
        Box::pin(async move {
            php.get()
                .expect("PHP adapter binds before ingress starts")
                .handle(request.map(|body| body.boxed_unsync()))
                .await
        })
    }
}

pub(crate) struct Application {
    pub name: String,
    pub service: Symfony,
}

impl Component<AppState> for Application {
    type Deps = Option<Arc<PhpApplicationService>>;

    fn name(&self) -> &'static str {
        "SkeneApplication"
    }

    fn as_bootable(&self) -> Option<&dyn Bootable<AppState, Self::Deps>> {
        Some(self)
    }

    fn validate(
        &self,
        state: &AppState,
    ) -> registry::Result<()> {
        let runtime =
            state.registry_ref().resolve::<PhpEmbedRuntime>().ok_or_else(
                || registry::Error::msg("PHP runtime was not registered"),
            )?;
        if !runtime.has_application(&self.name) {
            return Err(registry::Error::msg(format!(
                "PHP application {:?} is not registered",
                self.name
            )));
        }
        Ok(())
    }
}

#[async_trait]
impl Bootable<AppState, Option<Arc<PhpApplicationService>>> for Application {
    async fn boot(
        &self,
        php: Option<Arc<PhpApplicationService>>,
        state: &AppState,
    ) -> registry::Result<()> {
        if state.is_master() {
            return Ok(());
        }
        let php = php.ok_or_else(|| {
            registry::Error::msg("PHP application service was not registered")
        })?;
        self.service
            .0
            .set(PhpApplicationAdapter::new(php, self.name.clone()))
            .map_err(|_| {
                registry::Error::msg("Symfony adapter was already bound")
            })
    }
}
