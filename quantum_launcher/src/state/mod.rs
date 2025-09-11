use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    path::Path,
    str::FromStr,
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
};

use iced::{widget::image::Handle, Task};
use notify::Watcher;
use ql_core::{
    err, file_utils, GenericProgress, InstanceSelection, IntoIoError, IntoStringError, IoError,
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
};

mod menu;
mod message;
pub use menu::*;
pub use message::*;

pub const OFFLINE_ACCOUNT_NAME: &str = "(Offline)";
pub const NEW_ACCOUNT_NAME: &str = "+ Add Account";

pub const ADD_JAR_NAME: &str = "+ Add JAR";
pub const REMOVE_JAR_NAME: &str = "- Remove Selected";
pub const NONE_JAR_NAME: &str = "(None)";

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
    pub is_launching_game: bool,

    pub java_recv: Option<ProgressBar<GenericProgress>>,
    pub custom_jar: Option<CustomJarState>,

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

pub struct CustomJarState {
    pub choices: Vec<String>,
    pub recv: Receiver<notify::Event>,
    pub _watcher: notify::RecommendedWatcher,
}

impl CustomJarState {
    pub fn load() -> Task<Message> {
        Task::perform(load_custom_jars(), |n| {
            Message::EditInstance(EditInstanceMessage::CustomJarLoaded(n.strerr()))
        })
    }
}

#[derive(Default)]
pub struct ImageState {
    pub bitmap: HashMap<String, Handle>,
    pub svg: HashMap<String, iced::widget::svg::Handle>,
    pub downloads_in_progress: HashSet<String>,
    pub to_load: Mutex<HashSet<String>>,
}

#[derive(Debug)]
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

        let selected_account = config.account_selected.clone().unwrap_or(
            accounts_dropdown
                .first()
                .cloned()
                .unwrap_or_else(|| OFFLINE_ACCOUNT_NAME.to_owned()),
        );

        let (window_width, window_height) = config.read_window_size();

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
            window_size: (window_width, window_height),
            accounts,
            accounts_dropdown,
            accounts_selected: Some(selected_account),
            keys_pressed: HashSet::new(),
            tick_timer: 0,
            custom_jar: None,
        })
    }

    pub fn with_error(error: impl Display) -> Self {
        let error = error.to_string();
        let launcher_dir = if error.contains("Could not get launcher dir") {
            None
        } else {
            Some(LAUNCHER_DIR.clone())
        };

        let (mut config, theme) = launcher_dir
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

        let (window_width, window_height) = config.read_window_size();

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
            window_size: (window_width, window_height),
            accounts: HashMap::new(),
            accounts_dropdown: vec![OFFLINE_ACCOUNT_NAME.to_owned(), NEW_ACCOUNT_NAME.to_owned()],
            accounts_selected: Some(OFFLINE_ACCOUNT_NAME.to_owned()),
            keys_pressed: HashSet::new(),
            tick_timer: 0,
            custom_jar: None,
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
        Task::perform(get_entries(false), Message::CoreListLoaded)
    }
}

fn load_account(
    accounts: &mut HashMap<String, AccountData>,
    accounts_dropdown: &mut Vec<String>,
    accounts_to_remove: &mut Vec<String>,
    username: &str,
    account: &mut crate::config::ConfigAccount,
) {
    fn get_refresh_token_for_account_type(
        account_type: AccountType,
        username: &str,
        keyring_identifier: Option<&str>,
    ) -> Result<String, String> {
        let keyring_username = if let Some(keyring_id) = keyring_identifier {
            keyring_id
        } else {
            // Fallback to old behavior for backwards compatibility
            match account_type {
                AccountType::ElyBy => username.strip_suffix(" (elyby)").unwrap_or(username),
                AccountType::LittleSkin => {
                    username.strip_suffix(" (littleskin)").unwrap_or(username)
                }
                AccountType::Microsoft => username,
            }
        };
        ql_instances::auth::read_refresh_token(keyring_username, account_type).strerr()
    }

    let (account_type, refresh_token) =
        if account.account_type.as_deref() == Some("ElyBy") || username.ends_with(" (elyby)") {
            (
                AccountType::ElyBy,
                get_refresh_token_for_account_type(
                    AccountType::ElyBy,
                    username,
                    account.keyring_identifier.as_deref(),
                ),
            )
        } else if account.account_type.as_deref() == Some("LittleSkin")
            || username.ends_with(" (littleskin)")
        {
            (
                AccountType::LittleSkin,
                get_refresh_token_for_account_type(
                    AccountType::LittleSkin,
                    username,
                    account.keyring_identifier.as_deref(),
                ),
            )
        } else {
            (
                AccountType::Microsoft,
                get_refresh_token_for_account_type(
                    AccountType::Microsoft,
                    username,
                    account.keyring_identifier.as_deref(),
                ),
            )
        };

    let keyring_username = if let Some(keyring_id) = &account.keyring_identifier {
        keyring_id.clone()
    } else {
        // Fallback to old behavior for backwards compatibility
        match account_type {
            AccountType::ElyBy => username
                .strip_suffix(" (elyby)")
                .unwrap_or(username)
                .to_owned(),
            AccountType::LittleSkin => username
                .strip_suffix(" (littleskin)")
                .unwrap_or(username)
                .to_owned(),
            AccountType::Microsoft => username.to_owned(),
        }
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

                    username: keyring_username.clone(),
                    nice_username: account
                        .username_nice
                        .clone()
                        .unwrap_or(keyring_username.clone()),
                },
            );
        }
        Err(err) => {
            err!(
                "Could not load account: {err}\nUsername: {keyring_username}, Account Type: {}",
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

pub async fn get_entries(is_server: bool) -> Res<(Vec<String>, bool)> {
    let dir_path = file_utils::get_launcher_dir().strerr()?.join(if is_server {
        "servers"
    } else {
        "instances"
    });
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
            .strerr()?
            .into_iter()
            .filter(|n| !n.is_file)
            .map(|n| n.name)
            .collect(),
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

pub async fn load_custom_jars() -> Result<Vec<String>, IoError> {
    let names = file_utils::read_filenames_from_dir(LAUNCHER_DIR.join("custom_jars")).await?;
    let mut list: Vec<String> = names
        .into_iter()
        .filter(|n| n.is_file)
        .map(|n| n.name)
        .collect();

    list.insert(0, NONE_JAR_NAME.to_owned());
    list.push(ADD_JAR_NAME.to_owned());
    list.push(REMOVE_JAR_NAME.to_owned());

    Ok(list)
}

pub fn dir_watch<P: AsRef<Path>>(
    path: P,
) -> notify::Result<(mpsc::Receiver<notify::Event>, notify::RecommendedWatcher)> {
    let (tx, rx) = mpsc::channel();

    // `notify` runs callbacks in its own thread.
    let mut watcher: notify::RecommendedWatcher = notify::recommended_watcher(move |res| {
        if let Ok(event) = res {
            _ = tx.send(event);
        }
    })?;
    watcher.watch(path.as_ref(), notify::RecursiveMode::NonRecursive)?;

    Ok((rx, watcher))
}
