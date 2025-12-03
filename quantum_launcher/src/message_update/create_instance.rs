use iced::Task;
use ql_core::{
    pt, DownloadProgress, InstanceSelection, IntoStringError, JsonDownloadError, ListEntry,
};

use crate::state::{
    CreateInstanceMessage, Launcher, MenuCreateInstance, Message, ProgressBar, State,
};

impl Launcher {
    pub fn update_create_instance(&mut self, message: CreateInstanceMessage) -> Task<Message> {
        match message {
            CreateInstanceMessage::ScreenOpen { is_server } => {
                return self.go_to_create_screen(is_server)
            }
            CreateInstanceMessage::VersionsLoaded(result, is_server) => {
                self.create_instance_finish_loading_versions_list(result, is_server);
            }
            CreateInstanceMessage::VersionSelected(selected_version) => {
                self.select_created_instance_version(selected_version);
            }
            CreateInstanceMessage::NameInput(name) => self.update_created_instance_name(name),
            CreateInstanceMessage::Start => return self.create_instance(),
            CreateInstanceMessage::End(result) => match result {
                Ok(instance) => {
                    let is_server = instance.is_server();
                    self.selected_instance = Some(instance);
                    return if is_server {
                        self.go_to_server_manage_menu(Some("Created Server".to_owned()))
                    } else {
                        self.go_to_launch_screen(Some("Created Instance"))
                    };
                }
                Err(n) => self.set_error(n),
            },
            CreateInstanceMessage::ChangeAssetToggle(t) => {
                if let State::Create(MenuCreateInstance::Choosing {
                    download_assets, ..
                }) = &mut self.state
                {
                    *download_assets = t;
                }
            }
            CreateInstanceMessage::Cancel => {
                return self.go_to_launch_screen(Option::<String>::None)
            }
            CreateInstanceMessage::Import => {
                if let Some(file) = rfd::FileDialog::new()
                    .set_title("Select an instance...")
                    .pick_file()
                {
                    let (send, recv) = std::sync::mpsc::channel();
                    let progress = ProgressBar::with_recv(recv);

                    // I know this doesn't look necessary but there's
                    // a weird untrackable bug where importing instance
                    // screen just doesn't appear, and the Task runs
                    // silently in the background.
                    //
                    // I hope I manage to fix it in the future.
                    pt!("(Internal): Setting state to ImportingInstance...");

                    self.state = State::Create(MenuCreateInstance::ImportingInstance(progress));

                    return Task::perform(
                        ql_packager::import_instance(file.clone(), true, Some(send)),
                        |n| {
                            Message::CreateInstance(CreateInstanceMessage::ImportResult(n.strerr()))
                        },
                    );
                }
            }
            CreateInstanceMessage::ImportResult(res) => match res {
                Ok(instance) => {
                    let is_valid_modpack = instance.is_some();
                    self.selected_instance = instance;
                    if is_valid_modpack {
                        return self.go_to_launch_screen(None::<String>);
                    }
                    self.set_error(
                        r#"the file you imported isn't a valid QuantumLauncher/MultiMC instance.

If you meant to import a Modrinth/Curseforge/Preset pack,
create a instance with the matching version,
then go to "Mods->Add File""#,
                    );
                }
                Err(err) => self.set_error(err),
            },
        }
        Task::none()
    }

    fn create_instance_finish_loading_versions_list(
        &mut self,
        result: Result<(Vec<ListEntry>, String), String>,
        is_server: bool,
    ) {
        match result {
            Ok((versions, latest)) => {
                if is_server {
                    self.version_list_cache.server = Some(versions.clone());
                } else {
                    self.version_list_cache.client = Some(versions.clone());
                }
                self.version_list_cache.latest_stable = Some(latest);
                let combo_state = iced::widget::combo_box::State::new(versions.clone());
                if let State::Create(MenuCreateInstance::LoadingList { .. }) = &self.state {
                    self.state = State::Create(MenuCreateInstance::Choosing {
                        instance_name: String::new(),
                        selected_version: None,
                        download_assets: true,
                        combo_state: Box::new(combo_state),
                        is_server,
                    });
                }
            }
            Err(n) => self.set_error(n),
        }
    }

    pub fn go_to_create_screen(&mut self, is_server: bool) -> Task<Message> {
        if let Some(versions) = self.version_list_cache.client.clone() {
            let combo_state = iced::widget::combo_box::State::new(versions.clone());
            self.state = State::Create(MenuCreateInstance::Choosing {
                instance_name: String::new(),
                selected_version: None,
                download_assets: true,
                combo_state: Box::new(combo_state),
                is_server,
            });
            Task::none()
        } else {
            let msg = move |n: Result<(Vec<ListEntry>, String), JsonDownloadError>| {
                Message::CreateInstance(CreateInstanceMessage::VersionsLoaded(
                    n.strerr(),
                    is_server,
                ))
            };
            let (task, handle) = if is_server {
                Task::perform(ql_servers::list(), msg)
            } else {
                Task::perform(ql_instances::list_versions(), msg)
            }
            .abortable();

            self.state = State::Create(MenuCreateInstance::LoadingList {
                _handle: handle.abort_on_drop(),
            });

            task
        }
    }

    fn select_created_instance_version(&mut self, entry: ListEntry) {
        if let State::Create(MenuCreateInstance::Choosing {
            selected_version, ..
        }) = &mut self.state
        {
            *selected_version = Some(entry);
        }
    }

    fn update_created_instance_name(&mut self, name: String) {
        if let State::Create(MenuCreateInstance::Choosing { instance_name, .. }) = &mut self.state {
            *instance_name = name;
        }
    }

    fn create_instance(&mut self) -> Task<Message> {
        if let State::Create(MenuCreateInstance::Choosing {
            instance_name,
            download_assets,
            selected_version,
            is_server,
            ..
        }) = &mut self.state
        {
            let is_server = *is_server;
            let (sender, receiver) = std::sync::mpsc::channel::<DownloadProgress>();
            let progress = ProgressBar {
                num: 0.0,
                message: Some("Started download".to_owned()),
                receiver,
                progress: DownloadProgress::DownloadingJsonManifest,
            };

            let version = selected_version
                .clone()
                .or(self
                    .version_list_cache
                    .latest_stable
                    .clone()
                    .map(|name| ListEntry {
                        name,
                        is_server,
                        is_snapshot: false,
                    }))
                .unwrap();
            let instance_name = if instance_name.trim().is_empty() {
                version.name.clone()
            } else {
                instance_name.clone()
            };
            let download_assets = *download_assets;

            self.state = State::Create(MenuCreateInstance::DownloadingInstance(progress));

            return if is_server {
                Task::perform(
                    async move {
                        let sender = sender;
                        ql_servers::create_server(instance_name.clone(), version, Some(&sender))
                            .await
                            .strerr()
                            .map(InstanceSelection::Server)
                    },
                    |n| Message::CreateInstance(CreateInstanceMessage::End(n)),
                )
            } else {
                Task::perform(
                    ql_instances::create_instance(
                        instance_name.clone(),
                        version,
                        Some(sender),
                        download_assets,
                    ),
                    |n| {
                        Message::CreateInstance(CreateInstanceMessage::End(
                            n.strerr().map(InstanceSelection::Instance),
                        ))
                    },
                )
            };
        }
        Task::none()
    }
}
