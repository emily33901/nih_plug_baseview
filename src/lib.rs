//! [egui](https://github.com/emilk/egui) editor support for NIH plug.
//!
//! TODO: Proper usage example, for now check out the gain_gui example

// See the comment in the main `nih_plug` crate
#![allow(clippy::type_complexity)]

use baseview::WindowHandler;
use crossbeam::atomic::AtomicCell;
use nih_plug::params::persist::PersistentField;
use nih_plug::prelude::{Editor, GuiContext, ParamSetter};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod editor;

// export baseview for convenience
pub use baseview;

/// Create an [`Editor`] instance using an [`egui`][::egui] GUI. Using the user state parameter is
/// optional, but it can be useful for keeping track of some temporary GUI-only settings. See the
/// `gui_gain` example for more information on how to use this. The [`EguiState`] passed to this
/// function contains the GUI's intitial size, and this is kept in sync whenever the GUI gets
/// resized. You can also use this to know if the GUI is open, so you can avoid performing
/// potentially expensive calculations while the GUI is not open. If you want this size to be
/// persisted when restoring a plugin instance, then you can store it in a `#[persist = "key"]`
/// field on your parameters struct.
///
/// See [`EguiState::from_size()`].
pub fn create_baseview_editor<T, B, H>(
    baseview_state: Arc<BaseviewState>,
    user_state: T,
    build: B,
    // update: U,
) -> Option<Box<dyn Editor>>
where
    T: 'static + Send + Sync,
    B: Fn(&baseview::Window, Arc<dyn GuiContext>, &mut T) -> H + 'static + Send + Sync,
    H: WindowHandler + Send + Sync + 'static, // U: Fn(&Context, &ParamSetter, &mut T) + 'static + Send + Sync,
{
    Some(Box::new(editor::BaseviewEditor {
        baseview_state,
        user_state: Arc::new(RwLock::new(user_state)),
        build: Arc::new(build),
        // render: Arc::new(update),

        // TODO: We can't get the size of the window when baseview does its own scaling, so if the
        //       host does not set a scale factor on Windows or Linux we should just use a factor of
        //       1. That may make the GUI tiny but it also prevents it from getting cut off.
        #[cfg(target_os = "macos")]
        scaling_factor: AtomicCell::new(None),
        #[cfg(not(target_os = "macos"))]
        scaling_factor: AtomicCell::new(Some(1.0)),
    }))
}

/// State for an `nih_plug_egui` editor.
#[derive(Debug, Serialize, Deserialize)]
pub struct BaseviewState {
    /// The window's size in logical pixels before applying `scale_factor`.
    #[serde(with = "nih_plug::params::persist::serialize_atomic_cell")]
    size: AtomicCell<(u32, u32)>,
    /// Whether the editor's window is currently open.
    #[serde(skip)]
    open: AtomicBool,
}

impl<'a> PersistentField<'a, BaseviewState> for Arc<BaseviewState> {
    fn set(&self, new_value: BaseviewState) {
        self.size.store(new_value.size.load());
    }

    fn map<F, R>(&self, f: F) -> R
    where
        F: Fn(&BaseviewState) -> R,
    {
        f(self)
    }
}

impl BaseviewState {
    /// Initialize the GUI's state. This value can be passed to [`create_egui_editor()`]. The window
    /// size is in logical pixels, so before it is multiplied by the DPI scaling factor.
    pub fn from_size(width: u32, height: u32) -> Arc<BaseviewState> {
        Arc::new(BaseviewState {
            size: AtomicCell::new((width, height)),
            open: AtomicBool::new(false),
        })
    }

    /// Returns a `(width, height)` pair for the current size of the GUI in logical pixels.
    pub fn size(&self) -> (u32, u32) {
        self.size.load()
    }

    /// Whether the GUI is currently visible.
    // Called `is_open()` instead of `open()` to avoid the ambiguity.
    pub fn is_open(&self) -> bool {
        self.open.load(Ordering::Acquire)
    }
}
