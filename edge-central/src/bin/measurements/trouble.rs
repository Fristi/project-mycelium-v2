use std::sync::Arc;

use trouble_host::prelude::*;

pub struct TroublePeripheralSyncResultStreamProvider<C>
where
    C: Controller + Send + Sync + 'static,
{
    #[allow(dead_code)]
    controller: Arc<C>,
}

impl<C> TroublePeripheralSyncResultStreamProvider<C>
where
    C: Controller + Send + Sync + 'static,
{
    pub fn new(controller: C) -> Self {
        Self {
            controller: Arc::new(controller),
        }
    }
}
