// TUI App module root - re-exports state and core types

mod state;
mod instances_ctrl;
mod settings_ctrl;
mod config_ctrl;
mod logs_ctrl;
mod instance_settings_ctrl;
mod accounts_ctrl;
mod create_ctrl;
mod launch_ctrl;
pub(crate) mod licenses;
pub(crate) mod logs;
mod events;

pub use state::{
    AccountType,
    AddAccountFieldFocus,
    App,
    AppResult,
    Instance,
    InstanceSettingsTab,
    InstanceSettingsPage,
    ArgsEditKind,
    SettingsFocus,
    TabId,
    VersionCategory,
};

// Internal modules; callers should use App methods or import modules explicitly.
