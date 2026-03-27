use std::{collections::HashMap, time::Instant};

use iced::{Task, futures::executor::block_on, widget::scrollable::AbsoluteOffset};
use ql_core::{
    InstanceConfigJson, InstanceSelection, IntoStringError, JsonFileError, ModId, StoreBackendType,
    err, json::VersionDetails,
};
use ql_mod_manager::store::{ModIndex, Query, QueryType, get_description};

use crate::state::{
    InstallModsMessage, Launcher, MenuCurseforgeManualDownload, MenuModsDownload, Message,
    ModOperation, ProgressBar, State,
};

impl Launcher {
    pub fn update_install_mods(&mut self, message: InstallModsMessage) -> Task<Message> {
        let is_server = matches!(&self.selected_instance, Some(InstanceSelection::Server(_)));

        match message {
            InstallModsMessage::LoadData(Err(err))
            | InstallModsMessage::DownloadComplete(Err(err))
            | InstallModsMessage::SearchResult(Err(err))
            | InstallModsMessage::IndexUpdated(Err(err))
            | InstallModsMessage::VersionsLoaded(Err(err))
            | InstallModsMessage::UninstallComplete(Err(err)) => {
                self.set_error(err);
            }

            InstallModsMessage::SearchResult(Ok(search)) => {
                if let State::ModsDownload(menu) = &mut self.state {
                    menu.is_loading_continuation = false;
                    menu.has_continuation_ended = search.reached_end;

                    if search.start_time > menu.latest_load {
                        menu.latest_load = search.start_time;

                        if let (Some(results), true) = (&mut menu.results, search.offset > 0) {
                            results.mods.extend(search.mods);
                        } else {
                            menu.results = Some(search);
                        }
                    }
                }
            }
            InstallModsMessage::Scrolled(viewport) => {
                let total_height =
                    viewport.content_bounds().height - (viewport.bounds().height * 2.0);
                let absolute_offset = viewport.absolute_offset();
                let scroll_px = absolute_offset.y;

                if let State::ModsDownload(menu) = &mut self.state {
                    if menu.results.is_none() {
                        menu.has_continuation_ended = false;
                    }

                    menu.scroll_offset = absolute_offset;
                    if (scroll_px > total_height)
                        && !menu.is_loading_continuation
                        && !menu.has_continuation_ended
                    {
                        menu.is_loading_continuation = true;

                        let offset = if let Some(results) = &menu.results {
                            results.offset + results.mods.len()
                        } else {
                            0
                        };
                        return menu.search_store(is_server, offset);
                    }
                }
            }
            InstallModsMessage::Open => match block_on(self.open_mods_store()) {
                Ok(command) => return command,
                Err(err) => self.set_error(err),
            },
            InstallModsMessage::TickDesc(update_msg) => {
                if let State::ModsDownload(MenuModsDownload {
                    description: Some(description),
                    ..
                }) = &mut self.state
                {
                    description.update(update_msg);
                }
            }
            InstallModsMessage::SearchInput(input) => {
                if let State::ModsDownload(menu) = &mut self.state {
                    menu.query = input;
                    return menu.search_store(is_server, 0);
                }
            }
            InstallModsMessage::Click(i) => {
                if let State::ModsDownload(menu) = &mut self.state {
                    menu.opened_mod = Some(i);
                    menu.reload_description(&mut self.images);
                    if let Some(results) = &menu.results {
                        let hit = results.mods.get(i).unwrap();
                        let id = ModId::from_pair(&hit.id, results.backend);

                        let mut tasks =
                            vec![Task::done(InstallModsMessage::FetchVersions(id.clone()).into())];

                        if !menu.mod_descriptions.contains_key(&id) {
                            tasks.push(Task::perform(get_description(id), |n| {
                                InstallModsMessage::LoadData(n.strerr()).into()
                            }));
                        }

                        return Task::batch(tasks);
                    }
                }
            }
            InstallModsMessage::BackToMainScreen => {
                if let State::ModsDownload(menu) = &mut self.state {
                    menu.opened_mod = None;
                    menu.description = None;
                    menu.mod_versions = None;
                    return iced::widget::scrollable::scroll_to(
                        iced::widget::scrollable::Id::new("MenuModsDownload:main:mods_list"),
                        menu.scroll_offset,
                    );
                }
            }
            InstallModsMessage::LoadData(Ok((id, description))) => {
                if let State::ModsDownload(menu) = &mut self.state {
                    menu.mod_descriptions.insert(id, description);
                    menu.reload_description(&mut self.images);
                }
            }
            InstallModsMessage::Download(index) => {
                return self.mod_download(index);
            }
            InstallModsMessage::DownloadComplete(Ok((id, not_allowed))) => {
                let task = if let State::ModsDownload(menu) = &mut self.state {
                    menu.mods_download_in_progress.remove(&id);
                    Task::none()
                } else {
                    match block_on(self.open_mods_store()) {
                        Ok(n) => n,
                        Err(err) => {
                            self.set_error(err);
                            Task::none()
                        }
                    }
                };

                if not_allowed.is_empty() {
                    return task;
                }
                self.state = State::CurseforgeManualDownload(MenuCurseforgeManualDownload {
                    not_allowed,
                    delete_mods: true,
                });
            }
            InstallModsMessage::IndexUpdated(Ok(idx)) => {
                if let State::ModsDownload(menu) = &mut self.state {
                    menu.mod_index = idx;
                }
            }

            InstallModsMessage::ChangeBackend(backend) => {
                if let State::ModsDownload(menu) = &mut self.state {
                    menu.backend = backend;
                    menu.results = None;
                    menu.scroll_offset = AbsoluteOffset::default();
                    return menu.search_store(is_server, 0);
                }
            }
            InstallModsMessage::ChangeQueryType(query) => {
                if let State::ModsDownload(menu) = &mut self.state {
                    menu.query_type = query;
                    menu.results = None;
                    menu.scroll_offset = AbsoluteOffset::default();
                    return menu.search_store(is_server, 0);
                }
            }
            InstallModsMessage::InstallModpack(id) => {
                let (sender, receiver) = std::sync::mpsc::channel();
                self.state = State::ImportModpack(ProgressBar::with_recv(receiver));

                let selected_instance = self.selected_instance.clone().unwrap();

                return Task::perform(
                    async move {
                        ql_mod_manager::store::download_mod(&id, &selected_instance, Some(sender))
                            .await
                            .map(|not_allowed| (id, not_allowed))
                    },
                    |n| InstallModsMessage::DownloadComplete(n.strerr()).into(),
                );
            }
            InstallModsMessage::Uninstall(index) => {
                let State::ModsDownload(MenuModsDownload {
                    results: Some(results),
                    mods_download_in_progress,
                    ..
                }) = &mut self.state
                else {
                    return Task::none();
                };
                let Some(hit) = results.mods.get(index) else {
                    err!("Couldn't uninstall mod: Index out of range");
                    return Task::none();
                };

                let mod_id = ModId::from_pair(&hit.id, results.backend);
                mods_download_in_progress
                    .insert(mod_id.clone(), (hit.title.clone(), ModOperation::Deleting));
                let selected_instance = self.instance().clone();

                return Task::perform(
                    ql_mod_manager::store::delete_mods(vec![mod_id], selected_instance),
                    |n| InstallModsMessage::UninstallComplete(n.strerr()).into(),
                );
            }
            InstallModsMessage::UninstallComplete(Ok(ids)) => {
                if let State::ModsDownload(menu) = &mut self.state {
                    for id in ids {
                        menu.mods_download_in_progress.remove(&id);
                        menu.mod_index.mods.remove(&id.get_index_str());
                    }
                }
            }
            InstallModsMessage::FetchVersions(id) => {
                use ql_mod_manager::store::Backend;
                return Task::perform(
                    async move {
                        match id {
                            ModId::Modrinth(n) => {
                                ql_mod_manager::store::ModrinthBackend::get_versions(&n).await
                            }
                            ModId::Curseforge(n) => {
                                ql_mod_manager::store::CurseforgeBackend::get_versions(&n).await
                            }
                        }
                    },
                    |n| InstallModsMessage::VersionsLoaded(n.strerr()).into(),
                );
            }
            InstallModsMessage::VersionsLoaded(Ok(versions)) => {
                if let State::ModsDownload(menu) = &mut self.state {
                    if let Some(opened_idx) = menu.opened_mod {
                        if let Some(results) = &menu.results {
                            if let Some(hit) = results.mods.get(opened_idx) {
                                let id = ModId::from_pair(&hit.id, results.backend);
                                menu.mod_versions = Some((id, versions));
                            }
                        }
                    }
                }
            }
            InstallModsMessage::InstallVersion(id, version_id) => {
                let selected_instance = self.instance().clone();
                let State::ModsDownload(menu) = &mut self.state else {
                    return Task::none();
                };

                let title = menu
                    .mod_versions
                    .as_ref()
                    .and_then(|(vid, versions)| (vid == &id).then_some(versions))
                    .and_then(|versions| versions.iter().find(|v| v.id == version_id))
                    .map(|v| v.name.clone())
                    .unwrap_or_else(|| "Mod".to_owned());

                menu.mods_download_in_progress
                    .insert(id.clone(), (title, ModOperation::Downloading));

                return Task::perform(
                    async move {
                        use ql_mod_manager::store::Backend;
                        let res = match &id {
                            ModId::Modrinth(n) => {
                                ql_mod_manager::store::ModrinthBackend::download_version(
                                    n,
                                    &version_id,
                                    &selected_instance,
                                    None,
                                )
                                .await
                            }
                            ModId::Curseforge(n) => {
                                ql_mod_manager::store::CurseforgeBackend::download_version(
                                    n,
                                    &version_id,
                                    &selected_instance,
                                    None,
                                )
                                .await
                            }
                        };
                        res.map(|not_allowed| (id, not_allowed))
                    },
                    |n| InstallModsMessage::DownloadComplete(n.strerr()).into(),
                );
            }
        }
        Task::none()
    }

    async fn open_mods_store(&mut self) -> Result<Task<Message>, JsonFileError> {
        let selection = self.instance();

        let config = InstanceConfigJson::read(selection).await?;
        let version_json = if let State::EditMods(menu) = &self.state {
            menu.version_json.clone()
        } else {
            Box::new(VersionDetails::load(selection).await?)
        };
        let mod_index = ModIndex::load(selection).await?;

        let mut menu = MenuModsDownload {
            scroll_offset: AbsoluteOffset::default(),
            config,
            version_json,
            latest_load: Instant::now(),
            query: String::new(),
            results: None,
            opened_mod: None,
            mod_descriptions: HashMap::new(),
            mods_download_in_progress: HashMap::new(),
            mod_index,
            is_loading_continuation: false,
            has_continuation_ended: false,
            description: None,

            backend: StoreBackendType::Modrinth,
            query_type: QueryType::Mods,

            mod_versions: None,
        };
        let command = menu.search_store(
            matches!(&self.selected_instance, Some(InstanceSelection::Server(_))),
            0,
        );
        self.state = State::ModsDownload(menu);
        Ok(command)
    }

    fn mod_download(&mut self, index: usize) -> Task<Message> {
        let selected_instance = self.instance().clone();
        let State::ModsDownload(menu) = &mut self.state else {
            return Task::none();
        };
        let Some(results) = &menu.results else {
            err!("Couldn't download mod: Search results empty");
            return Task::none();
        };
        let Some(hit) = results.mods.get(index) else {
            err!("Couldn't download mod: Not present in results");
            return Task::none();
        };

        menu.mods_download_in_progress.insert(
            ModId::from_pair(&hit.id, results.backend),
            (hit.title.clone(), ModOperation::Downloading),
        );

        let project_id = hit.id.clone();
        let backend = menu.backend;
        let id = ModId::from_pair(&project_id, backend);

        if let QueryType::ModPacks = menu.query_type {
            self.state = State::ConfirmAction {
                msg1: format!("install the modpack: {}", hit.title),
                msg2: "This might take a while, install many files, and use a lot of network..."
                    .to_owned(),
                yes: InstallModsMessage::InstallModpack(id).into(),
                no: InstallModsMessage::Open.into(),
            };
            Task::none()
        } else {
            Task::perform(
                async move {
                    ql_mod_manager::store::download_mod(&id, &selected_instance, None)
                        .await
                        .map(|not_allowed| (id, not_allowed))
                },
                |n| InstallModsMessage::DownloadComplete(n.strerr()).into(),
            )
        }
    }
}

impl MenuModsDownload {
    pub fn search_store(&mut self, is_server: bool, offset: usize) -> Task<Message> {
        let query = Query {
            name: self.query.clone(),
            version: self.version_json.get_id().to_owned(),
            loader: self.config.mod_type,
            server_side: is_server,
            // open_source: false, // TODO: Add Open Source filter
        };
        let backend = self.backend;
        Task::perform(
            ql_mod_manager::store::search(query, offset, backend, self.query_type),
            |n| InstallModsMessage::SearchResult(n.strerr()).into(),
        )
    }
}
