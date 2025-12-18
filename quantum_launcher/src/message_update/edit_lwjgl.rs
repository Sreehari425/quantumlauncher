use iced::Task;

use crate::state::{EditLwjglMessage, Launcher, MenuEditLwjgl, Message, State};

impl Launcher {
    pub fn update_edit_lwjgl(&mut self, message: EditLwjglMessage) -> Task<Message> {
        match message {
            EditLwjglMessage::VersionsLoaded(result) => {
                match result {
                    Ok(versions) => {
                        if let State::EditLwjgl(MenuEditLwjgl::Loading { initial_version, .. }) = &self.state {
                            let initial_version = initial_version.clone();
                            let selected_version = initial_version.clone().unwrap_or_else(|| {
                                versions.lwjgl3.first()
                                    .or_else(|| versions.lwjgl2.first())
                                    .cloned()
                                    .unwrap_or_else(|| "default".to_string())
                            });
                            
                            // Cache the versions for future use
                            self.lwjgl_versions_cache = Some(versions.clone());
                            
                            self.state = State::EditLwjgl(MenuEditLwjgl::Loaded {
                                versions,
                                selected_version,
                                initial_version,
                                is_applying: false,
                            });
                        }
                    }
                    Err(err) => {
                        self.state = State::Error {
                            error: format!("Failed to load LWJGL versions: {}", err),
                        };
                    }
                }
                Task::none()
            }
            EditLwjglMessage::VersionSelected(version) => {
                if let State::EditLwjgl(MenuEditLwjgl::Loaded { selected_version, .. }) = &mut self.state {
                    if let Some(v) = version {
                        *selected_version = v;
                    }
                }
                Task::none()
            }
            EditLwjglMessage::Apply => {
                let instance = self.instance().clone();
                
                if let State::EditLwjgl(MenuEditLwjgl::Loaded {
                    selected_version,
                    is_applying,
                    ..
                }) = &mut self.state
                {
                    *is_applying = true;
                    let version = Some(selected_version.clone()).filter(|v| v != "default");
                    
                    return Task::perform(
                        async move {
                            let mut config = ql_core::json::InstanceConfigJson::read(&instance).await?;
                            config.lwjgl_version = version;
                            config.save(&instance).await?;
                            Ok(())
                        },
                        |result: Result<(), ql_core::JsonFileError>| {
                            match result {
                                Ok(()) => Message::EditLwjgl(EditLwjglMessage::Back),
                                Err(err) => Message::Error(format!("Failed to save LWJGL version: {}", err)),
                            }
                        },
                    );
                }
                Task::none()
            }
            EditLwjglMessage::Back => {
                self.state = State::Launch(Default::default());
                Task::none()
            }
        }
    }
}
