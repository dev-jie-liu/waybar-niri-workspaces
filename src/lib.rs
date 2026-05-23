use std::collections::BTreeMap;

use futures::StreamExt;
use settings::Settings;
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};
use waybar_cffi::{
    Module,
    gtk::{self, Orientation, prelude::{BoxExt, Cast, ContainerExt, LabelExt, StyleContextExt, WidgetExt}},
    waybar_module,
};

mod compositor;
mod errors;
mod global;
mod icons;
mod screen;
mod settings;
mod widget;

use compositor::WorkspaceSnapshot;
use errors::ModuleError;
use global::{EventMessage, SharedState};
use widget::WindowButton;

static LOGGING: std::sync::LazyLock<()> = std::sync::LazyLock::new(|| {
    if let Err(e) = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::CLOSE)
        .try_init()
    {
        eprintln!("tracing subscriber initialization failed: {e}");
    }
});

struct WindowButtonsModule;

impl Module for WindowButtonsModule {
    type Config = Settings;

    fn init(info: &waybar_cffi::InitInfo, settings: Settings) -> Self {
        *LOGGING;

        let shared_state = SharedState::create(settings);
        let context = waybar_cffi::gtk::glib::MainContext::default();

        if let Err(e) = context.block_on(initialize_module(info, shared_state)) {
            tracing::error!(%e, "module initialization failed");
        }

        Self
    }
}

waybar_module!(WindowButtonsModule);

async fn initialize_module(info: &waybar_cffi::InitInfo, state: SharedState) -> Result<(), ModuleError> {
    let root = info.get_root_widget();

    let main_container = gtk::Box::new(Orientation::Horizontal, 4);
    main_container.style_context().add_class("niri-workspaces");

    root.add(&main_container);

    let context = waybar_cffi::gtk::glib::MainContext::default();
    context.spawn_local(async move {
        ModuleInstance::create(state, main_container).run_event_loop().await
    });

    Ok(())
}

struct WorkspaceBox {
    event_box: gtk::EventBox,
    inner: gtk::Box,
    label: gtk::Label,
}

struct ModuleInstance {
    buttons: BTreeMap<u64, WindowButton>,
    workspaces: BTreeMap<u64, WorkspaceBox>,
    container: gtk::Box,
    state: SharedState,
    previous_snapshot: Option<WorkspaceSnapshot>,
}

impl ModuleInstance {
    fn create(state: SharedState, container: gtk::Box) -> Self {
        Self {
            buttons: BTreeMap::new(),
            workspaces: BTreeMap::new(),
            container,
            state,
            previous_snapshot: None,
        }
    }

    async fn run_event_loop(&mut self) {
        let mut event_stream = Box::pin(self.state.create_event_stream());

        while let Some(event) = event_stream.next().await {
            match event {
                EventMessage::WindowUpdate(snapshot) => {
                    self.handle_window_update(snapshot).await;
                }
                EventMessage::Workspaces => {}
            }
        }
    }

    async fn handle_window_update(&mut self, snapshot: WorkspaceSnapshot) {
        let config = self.state.settings();

        let mut removed_windows: std::collections::BTreeSet<u64> = self.buttons.keys().copied().collect();
        let mut removed_workspaces: std::collections::BTreeSet<u64> = self.workspaces.keys().copied().collect();

        for (position, workspace_view) in snapshot.iter().enumerate() {
            let ws = &workspace_view.workspace;
            removed_workspaces.remove(&ws.id);

            let ws_box = self.workspaces.entry(ws.id).or_insert_with(|| {
                let event_box = gtk::EventBox::new();
                let inner = gtk::Box::new(Orientation::Horizontal, 0);
                inner.style_context().add_class("workspace");

                let label_text = &ws.idx.to_string();
                let label = gtk::Label::new(Some(&label_text));
                label.style_context().add_class("workspace-label");
                inner.pack_start(&label, false, false, 6);

                event_box.add(&inner);
                event_box.show_all();

                let state = self.state.clone();
                let ws_id = ws.id;
                event_box.connect_button_press_event(move |_, event| {
                    if event.button() == 1 {
                        if let Err(e) = state.compositor().focus_workspace(ws_id) {
                            tracing::warn!(%e, "focus workspace failed");
                        }
                    }
                    waybar_cffi::gtk::glib::Propagation::Stop
                });

                self.container.add(&event_box);
                WorkspaceBox { event_box, inner, label }
            });

            self.container.reorder_child(&ws_box.event_box, position as i32);

            // 更新 workspace 序号标签（当 workspace 被销毁/创建后序号可能变化）
            ws_box.label.set_text(&ws.idx.to_string());

            if ws.is_active {
                ws_box.inner.style_context().add_class("active");
            } else {
                ws_box.inner.style_context().remove_class("active");
            }

            for window in &workspace_view.windows {
                if window.app_id.is_some() && config.should_ignore(window.app_id.as_deref(), window.title.as_deref(), window.workspace_id) {
                    continue;
                }

                removed_windows.remove(&window.id);

                let button = self.buttons.entry(window.id).or_insert_with(|| {
                    let btn = WindowButton::create(&self.state, window);
                    ws_box.inner.add(btn.get_widget());
                    btn.get_widget().show_all();
                    btn
                });

                // 确保按钮在正确的 workspace 容器中
                if let Some(current_parent) = button.get_widget().parent() {
                    if let Ok(current_box) = current_parent.downcast::<gtk::Box>() {
                        if current_box != ws_box.inner {
                            current_box.remove(button.get_widget());
                            ws_box.inner.add(button.get_widget());
                        }
                    }
                }

                button.update_focus(window.is_focused);
            }
        }

        for window_id in removed_windows {
            if let Some(button) = self.buttons.remove(&window_id) {
                if let Some(parent) = button.get_widget().parent() {
                    if let Ok(container) = parent.downcast::<gtk::Box>() {
                        container.remove(button.get_widget());
                    }
                }
            }
        }

        for ws_id in removed_workspaces {
            if let Some(ws_box) = self.workspaces.remove(&ws_id) {
                self.container.remove(&ws_box.event_box);
            }
        }

        self.previous_snapshot = Some(snapshot);
    }
}
