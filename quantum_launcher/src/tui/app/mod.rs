// TUI App module root - re-exports state and core types

mod state;

pub use state::{
    AccountType,
    AddAccountFieldFocus,
    App,
    AppResult,
    Instance,
    InstanceSettingsTab,
    SettingsFocus,
    TabId,
    VersionCategory,
};
