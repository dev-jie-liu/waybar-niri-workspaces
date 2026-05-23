use crate::global::SharedState;
use std::{cell::RefCell, path::PathBuf};
use waybar_cffi::gtk::{
    self as gtk,
    gdk_pixbuf::Pixbuf,
    prelude::{
        BoxExt, ButtonExt, ContainerExt, CssProviderExt, GdkPixbufExt, IconThemeExt,
        StyleContextExt, WidgetExt,
    },
    CssProvider, IconLookupFlags, IconSize, IconTheme, Orientation, ReliefStyle,
};

#[derive(Clone)]
pub struct WindowButton {
    app_id: Option<String>,
    gtk_button: gtk::Button,
    state: SharedState,
    window_id: u64,
}

impl std::fmt::Debug for WindowButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowButton")
            .field("app_id", &self.app_id)
            .field("window_id", &self.window_id)
            .finish_non_exhaustive()
    }
}

thread_local! {
    static BUTTON_STYLES: CssProvider = {
        let provider = CssProvider::new();
        if let Err(e) = provider.load_from_data(include_bytes!("styles.css")) {
            tracing::error!(%e, "failed to load CSS");
        }
        provider
    };

    static ICON_THEME_INSTANCE: IconTheme = IconTheme::default().unwrap_or_default();
}

fn query_process_name(pid: i32) -> Option<String> {
    if pid <= 0 {
        return None;
    }

    let exe_path = format!("/proc/{}/exe", pid);
    if let Ok(path) = std::fs::read_link(&exe_path) {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            return Some(name.to_owned());
        }
    }

    // fallback to comm
    let comm_path = format!("/proc/{}/comm", pid);
    if let Ok(comm) = std::fs::read_to_string(&comm_path) {
        let name = comm.trim_end().to_owned(); // 去除末尾换行等
        if !name.is_empty() {
            return Some(name);
        }
    }

    None
}

impl WindowButton {
    #[tracing::instrument(level = "TRACE", fields(app_id = &window.app_id))]
    pub fn create(state: &SharedState, window: &niri_ipc::Window) -> Self {
        let state_clone = state.clone();
        let icon_dimension = state.settings().icon_size();

        let layout_box = gtk::Box::new(Orientation::Horizontal, 0);

        let gtk_button = gtk::Button::new();
        gtk_button.set_always_show_image(true);
        gtk_button.set_relief(ReliefStyle::None);
        gtk_button.add(&layout_box);

        BUTTON_STYLES.with(|provider| {
            gtk_button
                .style_context()
                .add_provider(provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        });

        let app_id = window
            .app_id
            .clone()
            .filter(|id| !id.is_empty())
            .or_else(|| query_process_name(window.pid.unwrap()));
        let icon_location = app_id
            .as_deref()
            .and_then(|id| state_clone.icon_resolver().resolve(id));

        let button = Self {
            app_id,
            gtk_button,
            state: state_clone,
            window_id: window.id,
        };

        button.setup_click_handler(window.id);
        button.setup_icon_rendering(layout_box, icon_location, icon_dimension);

        let tooltip_text = window
            .title
            .as_deref()
            .filter(|t| !t.is_empty())
            .or(button.app_id.as_deref())
            .unwrap_or("Unknown");
        button.gtk_button.set_tooltip_text(Some(tooltip_text));

        button
    }

    #[tracing::instrument(level = "TRACE")]
    pub fn update_focus(&self, is_focused: bool) {
        let style_ctx = self.gtk_button.style_context();
        if is_focused {
            style_ctx.add_class("focused");
        } else {
            style_ctx.remove_class("focused");
        }
        self.gtk_button.queue_draw();
    }

    pub fn get_widget(&self) -> &gtk::Button {
        &self.gtk_button
    }

    fn setup_click_handler(&self, window_id: u64) {
        let state = self.state.clone();
        self.gtk_button.connect_clicked(move |_| {
            if let Err(e) = state.compositor().focus_window(window_id) {
                tracing::warn!(%e, "focus window failed");
            }
        });
    }

    fn setup_icon_rendering(
        &self,
        container: gtk::Box,
        icon_path: Option<PathBuf>,
        icon_dimension: i32,
    ) {
        let last_allocation = RefCell::new(None);

        self.gtk_button
            .connect_size_allocate(move |button, allocation| {
                let mut needs_render = container.children().is_empty();

                if !needs_render {
                    if let Some(prev_alloc) = last_allocation.take() {
                        if &prev_alloc != allocation {
                            needs_render = true;
                        }
                    } else {
                        needs_render = true;
                    }
                    last_allocation.replace(Some(*allocation));
                }

                if needs_render {
                    let icon_image =
                        Self::load_icon_image(icon_path.as_ref(), button, icon_dimension)
                            .unwrap_or_else(|| {
                                static FALLBACK: &str = "application-x-executable";
                                ICON_THEME_INSTANCE
                                    .with(|theme| {
                                        theme.lookup_icon_for_scale(
                                            FALLBACK,
                                            icon_dimension,
                                            button.scale_factor(),
                                            IconLookupFlags::empty(),
                                        )
                                    })
                                    .and_then(|info| {
                                        Self::load_icon_image(
                                            info.filename().as_ref(),
                                            button,
                                            icon_dimension,
                                        )
                                    })
                                    .unwrap_or_else(|| {
                                        gtk::Image::from_icon_name(Some(FALLBACK), IconSize::Button)
                                    })
                            });

                    let container_copy = container.clone();
                    let button_copy = button.clone();
                    gtk::glib::source::idle_add_local_once(move || {
                        for child in container_copy.children() {
                            container_copy.remove(&child);
                        }
                        container_copy.pack_start(&icon_image, false, false, 0);
                        container_copy.show_all();
                        button_copy.show_all();
                    });
                }
            });
    }

    fn load_icon_image(
        path: Option<&PathBuf>,
        button: &gtk::Button,
        size: i32,
    ) -> Option<gtk::Image> {
        let scaled_size = size * button.scale_factor();

        path.and_then(
            |p| match Pixbuf::from_file_at_scale(p, scaled_size, scaled_size, true) {
                Ok(pixbuf) => Some(pixbuf),
                Err(e) => {
                    tracing::info!(%e, ?p, "icon load failed");
                    None
                }
            },
        )
        .and_then(|pixbuf| pixbuf.create_surface(0, button.window().as_ref()))
        .map(|surface| gtk::Image::from_surface(Some(&surface)))
    }
}
