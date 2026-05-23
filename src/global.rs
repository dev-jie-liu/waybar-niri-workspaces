use std::sync::Arc;
use niri_ipc::Workspace;
use waybar_cffi::gtk::glib;
use crate::{
    compositor::{self, CompositorClient, WorkspaceSnapshot},
    icons::IconResolver,
    settings::Settings,
};

#[derive(Debug, Clone)]
pub struct SharedState(Arc<StateInner>);

#[derive(Debug)]
struct StateInner {
    settings: Arc<Settings>,
    icon_resolver: IconResolver,
    compositor: CompositorClient,
}

impl SharedState {
    pub fn create(settings: Settings) -> Self {
        let settings = Arc::new(settings);
        Self(Arc::new(StateInner {
            compositor: CompositorClient::create(Arc::clone(&settings)),
            icon_resolver: IconResolver::new(),
            settings,
        }))
    }

    pub fn settings(&self) -> Arc<Settings> {
        Arc::clone(&self.0.settings)
    }

    pub fn icon_resolver(&self) -> &IconResolver {
        &self.0.icon_resolver
    }

    pub fn compositor(&self) -> &CompositorClient {
        &self.0.compositor
    }

    pub fn create_event_stream(&self) -> async_channel::Receiver<EventMessage> {
        let (tx, rx) = async_channel::unbounded();

        let window_tx = tx.clone();
        let (glib_win_tx, glib_win_rx) = async_channel::unbounded::<WorkspaceSnapshot>();
        glib::MainContext::default().spawn_local(async move {
            while let Ok(snapshot) = glib_win_rx.recv().await {
                if let Err(e) = window_tx.try_send(EventMessage::WindowUpdate(snapshot)) {
                    tracing::error!(%e, "failed to forward window update");
                }
            }
        });
        compositor::start_window_stream(glib_win_tx, self.compositor().only_current_workspace());

        let workspace_tx = tx;
        let (glib_ws_tx, glib_ws_rx) = async_channel::unbounded::<Vec<Workspace>>();
        glib::MainContext::default().spawn_local(async move {
            while let Ok(_workspaces) = glib_ws_rx.recv().await {
                if let Err(e) = workspace_tx.try_send(EventMessage::Workspaces) {
                    tracing::error!(%e, "failed to forward workspace change");
                }
            }
        });
        compositor::start_workspace_stream(glib_ws_tx);

        rx
    }
}

pub enum EventMessage {
    WindowUpdate(WorkspaceSnapshot),
    Workspaces,
}
