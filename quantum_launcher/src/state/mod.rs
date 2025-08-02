use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    str::FromStr,
    sync::{mpsc::Receiver, Arc, Mutex},
};

use iced::{widget::image::Handle, Task};
use ql_core::{
    err, file_utils, GenericProgress, InstanceSelection, IntoIoError, IntoStringError,
    JsonFileError, ListEntry, Progress, LAUNCHER_DIR, LAUNCHER_VERSION_NAME,
};
use ql_instances::{
    auth::{ms::CLIENT_ID, AccountData, AccountType},
    LogLine,
};
use tokio::process::{Child, ChildStdin};

use crate::{
    config::LauncherConfig,
    stylesheet::styles::{LauncherTheme, LauncherThemeColor, LauncherThemeLightness},
    WINDOW_HEIGHT, WINDOW_WIDTH,
};

mod menu;
mod message;
pub use menu::*;
pub use message::*;

pub const OFFLINE_ACCOUNT_NAME: &str = "(Offline)";
pub const NEW_ACCOUNT_NAME: &str = "+ Add Account";

type Res<T = ()> = Result<T, String>;

pub struct InstanceLog {
    pub log: Vec<String>,
    pub has_crashed: bool,
    pub command: String,
}

pub struct Launcher {
    pub state: State,
    pub selected_instance: Option<InstanceSelection>,
    pub config: LauncherConfig,
    pub theme: LauncherTheme,
    pub images: ImageState,

    pub is_log_open: bool,
    pub log_scroll: isize,
    pub tick_timer: usize,

    pub java_recv: Option<ProgressBar<GenericProgress>>,
    pub is_launching_game: bool,

    pub accounts: HashMap<String, AccountData>,
    pub accounts_dropdown: Vec<String>,
    pub accounts_selected: Option<String>,

    pub client_version_list_cache: Option<Vec<ListEntry>>,
    pub server_version_list_cache: Option<Vec<ListEntry>>,
    pub client_list: Option<Vec<String>>,
    pub server_list: Option<Vec<String>>,
    pub client_processes: HashMap<String, ClientProcess>,
    pub server_processes: HashMap<String, ServerProcess>,
    pub client_logs: HashMap<String, InstanceLog>,
    pub server_logs: HashMap<String, InstanceLog>,

    pub window_size: (f32, f32),
    pub mouse_pos: (f32, f32),
    pub keys_pressed: HashSet<iced::keyboard::Key>,
}

#[derive(Default)]
pub struct ImageState {
    pub bitmap: HashMap<String, Handle>,
    pub svg: HashMap<String, iced::widget::svg::Handle>,
    pub downloads_in_progress: HashSet<String>,
    pub to_load: Mutex<HashSet<String>>,
}

pub struct ClientProcess {
    pub child: Arc<Mutex<Child>>,
    pub receiver: Option<Receiver<LogLine>>,
}

pub struct ServerProcess {
    pub child: Arc<Mutex<Child>>,
    pub receiver: Option<Receiver<String>>,
    pub stdin: Option<ChildStdin>,
    pub is_classic_server: bool,
    pub has_issued_stop_command: bool,
}

impl Launcher {
    pub fn load_new(
        message: Option<String>,
        is_new_user: bool,
        config: Result<LauncherConfig, JsonFileError>,
    ) -> Result<Self, JsonFileError> {
        if let Err(err) = file_utils::get_launcher_dir() {
            err!("Could not get launcher dir (This is a bug):");
            return Ok(Self::with_error(format!(
                "Could not get launcher dir: {err}"
            )));
        }

        let mut config = config?;
        let theme = get_theme(&config);

        let mut launch = if let Some(message) = message {
            MenuLaunch::with_message(message)
        } else {
            MenuLaunch::default()
        };

        if let Some(sidebar_width) = config.sidebar_width {
            launch.sidebar_width = sidebar_width as u16;
        }

        let launch = State::Launch(launch);

        // The version field was added in 0.3
        let version = config.version.as_deref().unwrap_or("0.3.0");

        let state = if is_new_user {
            State::Welcome(MenuWelcome::P1InitialScreen)
        } else if version == LAUNCHER_VERSION_NAME {
            launch
        } else {
            config.version = Some(LAUNCHER_VERSION_NAME.to_owned());
            State::ChangeLog
        };

        let mut accounts = HashMap::new();

        let mut accounts_dropdown =
            vec![OFFLINE_ACCOUNT_NAME.to_owned(), NEW_ACCOUNT_NAME.to_owned()];

        if let Some(config_accounts) = config.accounts.as_mut() {
            let mut accounts_to_remove = Vec::new();

            for (username, account) in config_accounts.iter_mut() {
                load_account(
                    &mut accounts,
                    &mut accounts_dropdown,
                    &mut accounts_to_remove,
                    username,
                    account,
                );
            }

            for rem in accounts_to_remove {
                config_accounts.remove(&rem);
            }
        }

        let selected_account = accounts_dropdown
            .first()
            .cloned()
            .unwrap_or_else(|| OFFLINE_ACCOUNT_NAME.to_owned());

        Ok(Self {
            client_list: None,
            server_list: None,
            java_recv: None,
            is_log_open: false,
            log_scroll: 0,
            state,
            client_processes: HashMap::new(),
            config,
            client_logs: HashMap::new(),
            selected_instance: None,
            images: ImageState::default(),
            theme,
            is_launching_game: false,
            client_version_list_cache: None,
            server_version_list_cache: None,
            server_processes: HashMap::new(),
            server_logs: HashMap::new(),
            mouse_pos: (0.0, 0.0),
            window_size: (WINDOW_WIDTH, WINDOW_HEIGHT),
            accounts,
            accounts_dropdown,
            accounts_selected: Some(selected_account),
            keys_pressed: HashSet::new(),
            tick_timer: 0,
        })
    }

    pub fn with_error(error: impl std::fmt::Display) -> Self {
        let error = error.to_string();
        let launcher_dir = if error.contains("Could not get launcher dir") {
            None
        } else {
            Some(LAUNCHER_DIR.clone())
        };

        let (config, theme) = launcher_dir
            .as_ref()
            .and_then(|_| {
                match LauncherConfig::load_s().map(|n| {
                    let theme = get_theme(&n);
                    (n, theme)
                }) {
                    Ok(n) => Some(n),
                    Err(err) => {
                        err!("Error loading config: {err}");
                        None
                    }
                }
            })
            .unwrap_or((LauncherConfig::default(), LauncherTheme::default()));

        Self {
            state: State::Error { error },
            is_log_open: false,
            log_scroll: 0,
            java_recv: None,
            is_launching_game: false,
            client_list: None,
            server_list: None,
            config,
            client_processes: HashMap::new(),
            client_logs: HashMap::new(),
            selected_instance: None,
            images: ImageState::default(),
            theme,
            client_version_list_cache: None,
            server_processes: HashMap::new(),
            server_logs: HashMap::new(),
            server_version_list_cache: None,
            mouse_pos: (0.0, 0.0),
            window_size: (WINDOW_WIDTH, WINDOW_HEIGHT),
            accounts: HashMap::new(),
            accounts_dropdown: vec![OFFLINE_ACCOUNT_NAME.to_owned(), NEW_ACCOUNT_NAME.to_owned()],
            accounts_selected: Some(OFFLINE_ACCOUNT_NAME.to_owned()),
            keys_pressed: HashSet::new(),
            tick_timer: 0,
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn set_error(&mut self, error: impl ToString) {
        let error = error.to_string().replace(CLIENT_ID, "[CLIENT ID]");
        err!("{error}");
        self.state = State::Error { error }
    }

    pub fn go_to_launch_screen<T: Display>(&mut self, message: Option<T>) -> Task<Message> {
        let mut menu_launch = match message {
            Some(message) => MenuLaunch::with_message(message.to_string()),
            None => MenuLaunch::default(),
        };
        if let Some(width) = self.config.sidebar_width {
            menu_launch.sidebar_width = width as u16;
        }
        self.state = State::Launch(menu_launch);
        Task::perform(
            get_entries("instances".to_owned(), false),
            Message::CoreListLoaded,
        )
    }
}

fn load_account(
    accounts: &mut HashMap<String, AccountData>,
    accounts_dropdown: &mut Vec<String>,
    accounts_to_remove: &mut Vec<String>,
    username: &str,
    account: &mut crate::config::ConfigAccount,
) {
    let (account_type, refresh_token) =
        if account.account_type.as_deref() == Some("ElyBy") || username.ends_with(" (elyby)") {
            let username_stripped = username.strip_suffix(" (elyby)").unwrap_or(username);
            (
                AccountType::ElyBy,
                ql_instances::auth::elyby::read_refresh_token(username_stripped).strerr(),
            )
        } else if account.account_type.as_deref() == Some("LittleSkin")
            || username.ends_with(" (littleskin)")
        {
            let username_stripped = username.strip_suffix(" (littleskin)").unwrap_or(username);
            (
                AccountType::LittleSkin,
                ql_instances::auth::littleskin::read_refresh_token(username_stripped).strerr(),
            )
        } else if account.account_type.as_deref() == Some("BlessingSkin")
            || username.ends_with(" (blessing)")
        {
            let username_stripped = username.strip_suffix(" (blessing)").unwrap_or(username);
            // For blessing skin, we need the custom auth URL to read the refresh token
            if let Some(base_url) = &account.custom_auth_url {
                (
                    AccountType::BlessingSkin,
                    ql_instances::auth::blessing_skin::read_refresh_token(username_stripped, base_url).strerr(),
                )
            } else {
                // No auth URL stored, can't load this account
                return;
            }
        } else {
            let username_stripped = username;
            (
                AccountType::Microsoft,
                ql_instances::auth::ms::read_refresh_token(username_stripped).strerr(),
            )
        };

    let username_stripped = match account_type {
        AccountType::ElyBy => username.strip_suffix(" (elyby)").unwrap_or(username),
        AccountType::LittleSkin => username.strip_suffix(" (littleskin)").unwrap_or(username),
        AccountType::BlessingSkin => username.strip_suffix(" (blessing)").unwrap_or(username),
        AccountType::Microsoft => username,
    };

    match refresh_token {
        Ok(refresh_token) => {
            accounts_dropdown.insert(0, username.to_owned());
            accounts.insert(
                username.to_owned(),
                AccountData {
                    access_token: None,
                    uuid: account.uuid.clone(),
                    refresh_token,
                    needs_refresh: true,
                    account_type,

                    username: username_stripped.to_owned(),
                    nice_username: account
                        .username_nice
                        .clone()
                        .unwrap_or(username_stripped.to_owned()),
                    custom_auth_url: if account_type == AccountType::BlessingSkin {
                        account.custom_auth_url.clone()
                    } else {
                        None
                    },
                },
            );
        }
        Err(err) => {
            err!(
                "Could not load account: {err}\nUsername: {username_stripped}, Account Type: {}",
                account_type.to_string()
            );
            accounts_to_remove.push(username.to_owned());
        }
    }
}

fn get_theme(config: &LauncherConfig) -> LauncherTheme {
    let theme = match config.theme.as_deref() {
        Some("Dark") => LauncherThemeLightness::Dark,
        Some("Light") => LauncherThemeLightness::Light,
        None => LauncherThemeLightness::default(),
        _ => {
            err!("Unknown style: {:?}", config.theme);
            LauncherThemeLightness::default()
        }
    };
    let style = config
        .style
        .as_deref()
        .and_then(|n| LauncherThemeColor::from_str(n).ok())
        .unwrap_or_default();
    LauncherTheme::from_vals(style, theme)
}

pub async fn get_entries(path: String, is_server: bool) -> Res<(Vec<String>, bool)> {
    let dir_path = file_utils::get_launcher_dir().strerr()?.join(path);
    if !dir_path.exists() {
        tokio::fs::create_dir_all(&dir_path)
            .await
            .path(&dir_path)
            .strerr()?;
        return Ok((Vec::new(), is_server));
    }

    Ok((
        file_utils::read_filenames_from_dir(&dir_path)
            .await
            .strerr()?,
        is_server,
    ))
}

pub struct ProgressBar<T: Progress> {
    pub num: f32,
    pub message: Option<String>,
    pub receiver: Receiver<T>,
    pub progress: T,
}

impl<T: Default + Progress> ProgressBar<T> {
    pub fn with_recv(receiver: Receiver<T>) -> Self {
        Self {
            num: 0.0,
            message: None,
            receiver,
            progress: T::default(),
        }
    }

    pub fn with_recv_and_msg(receiver: Receiver<T>, msg: String) -> Self {
        Self {
            num: 0.0,
            message: Some(msg),
            receiver,
            progress: T::default(),
        }
    }
}

impl<T: Progress> ProgressBar<T> {
    pub fn tick(&mut self) -> bool {
        let mut has_ticked = false;
        while let Ok(progress) = self.receiver.try_recv() {
            self.num = progress.get_num();
            self.message = progress.get_message();
            self.progress = progress;
            has_ticked = true;
        }
        has_ticked
    }
}
