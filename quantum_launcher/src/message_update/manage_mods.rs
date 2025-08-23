use iced::futures::executor::block_on;
use iced::Task;
use ql_core::{
    err, err_no_log, jarmod::JarMods, InstanceSelection, IntoIoError, IntoStringError, ModId,
    SelectedMod,
};
use ql_mod_manager::store::ModIndex;
use std::{collections::HashSet, path::PathBuf};

use crate::state::{
    Launcher, ManageJarModsMessage, ManageModsMessage, MenuCurseforgeManualDownload,
    MenuEditJarMods, MenuEditMods, Message, ProgressBar, SelectedState, State,
};

impl Launcher {
    pub fn update_manage_mods(&mut self, msg: ManageModsMessage) -> Task<Message> {
        match msg {
            ManageModsMessage::ScreenOpen => match self.go_to_edit_mods_menu() {
                Ok(command) => return command,
                Err(err) => self.set_error(err),
            },
            ManageModsMessage::ScreenOpenWithoutUpdate => {
                return self.go_to_edit_mods_menu_without_update_check();
            }

            ManageModsMessage::ToggleCheckbox((name, id), enable) => {
                if let State::EditMods(menu) = &mut self.state {
                    if enable {
                        menu.selected_mods
                            .insert(SelectedMod::Downloaded { name, id });
                        menu.selected_state = SelectedState::Some;
                    } else {
                        menu.selected_mods
                            .remove(&SelectedMod::Downloaded { name, id });
                        menu.selected_state = if menu.selected_mods.is_empty() {
                            SelectedState::None
                        } else {
                            SelectedState::Some
                        };
                    }
                }
            }
            ManageModsMessage::AddFile => {
                if let Some(paths) = rfd::FileDialog::new()
                    .add_filter("Mod/Modpack", &["jar", "zip", "mrpack", "qmp"])
                    .set_title("Add Mod, Modpack or Preset")
                    .pick_files()
                {
                    let (sender, receiver) = std::sync::mpsc::channel();

                    self.state = State::ImportModpack(ProgressBar::with_recv(receiver));

                    return Task::perform(
                        ql_mod_manager::add_files(
                            self.selected_instance.clone().unwrap(),
                            paths,
                            Some(sender),
                        ),
                        move |n| Message::ManageMods(ManageModsMessage::AddFileDone(n.strerr())),
                    );
                }
            }
            ManageModsMessage::AddFileDone(n) => match n {
                Ok(unsupported) => {
                    if !unsupported.is_empty() {
                        self.state =
                            State::CurseforgeManualDownload(MenuCurseforgeManualDownload {
                                unsupported,
                                is_store: false,
                            });
                    }
                    return self.go_to_edit_mods_menu_without_update_check();
                }
                Err(err) => self.set_error(err),
            },
            ManageModsMessage::ToggleCheckboxLocal(name, enable) => {
                if let State::EditMods(menu) = &mut self.state {
                    if enable {
                        menu.selected_mods
                            .insert(SelectedMod::Local { file_name: name });
                        menu.selected_state = SelectedState::Some;
                    } else {
                        menu.selected_mods
                            .remove(&SelectedMod::Local { file_name: name });
                        menu.selected_state = if menu.selected_mods.is_empty() {
                            SelectedState::None
                        } else {
                            SelectedState::Some
                        };
                    }
                }
            }
            ManageModsMessage::DeleteSelected => {
                if let State::EditMods(menu) = &self.state {
                    let command = Self::get_delete_mods_command(
                        self.selected_instance.clone().unwrap(),
                        menu,
                    );
                    let mods_dir = self.get_selected_dot_minecraft_dir().unwrap().join("mods");
                    let file_paths = menu
                        .selected_mods
                        .iter()
                        .filter_map(|s_mod| {
                            if let SelectedMod::Local { file_name } = s_mod {
                                Some(file_name.clone())
                            } else {
                                None
                            }
                        })
                        .map(|n| mods_dir.join(n))
                        .map(delete_file_wrapper)
                        .map(|n| {
                            Task::perform(n, |n| {
                                Message::ManageMods(ManageModsMessage::LocalDeleteFinished(n))
                            })
                        });
                    let delete_local_command = Task::batch(file_paths);

                    return Task::batch([command, delete_local_command]);
                }
            }
            ManageModsMessage::DeleteFinished(result) => match result {
                Ok(_) => {
                    if let State::EditMods(menu) = &mut self.state {
                        menu.selected_mods.clear();
                    }
                    self.update_mod_index();
                }
                Err(err) => self.set_error(err),
            },
            ManageModsMessage::LocalDeleteFinished(result) => {
                if let Err(err) = result {
                    self.set_error(err);
                }
            }
            ManageModsMessage::LocalIndexLoaded(hash_set) => {
                if let State::EditMods(menu) = &mut self.state {
                    menu.locally_installed_mods = hash_set;
                }
            }
            ManageModsMessage::ToggleSelected => {
                if let State::EditMods(menu) = &mut self.state {
                    let (ids_downloaded, ids_local) = menu.get_kinds_of_ids();
                    let instance_name = self.selected_instance.clone().unwrap();

                    menu.selected_mods.clear();
                    /*menu.selected_mods.retain(|n| {
                        if let SelectedMod::Local { file_name } = n {
                            !ids_local.contains(file_name)
                        } else {
                            true
                        }
                    });

                    menu.selected_mods
                        .extend(ids_local.iter().map(|n| SelectedMod::Local {
                            file_name: ql_mod_manager::store::flip_filename(n),
                        }));*/

                    let toggle_downloaded = Task::perform(
                        ql_mod_manager::store::toggle_mods(ids_downloaded, instance_name.clone()),
                        |n| Message::ManageMods(ManageModsMessage::ToggleFinished(n.strerr())),
                    );
                    let toggle_local = Task::perform(
                        ql_mod_manager::store::toggle_mods_local(ids_local, instance_name.clone()),
                        |n| Message::ManageMods(ManageModsMessage::ToggleFinished(n.strerr())),
                    )
                    .chain(MenuEditMods::update_locally_installed_mods(
                        &menu.mods,
                        &instance_name,
                    ));

                    return Task::batch([toggle_downloaded, toggle_local]);
                }
            }
            ManageModsMessage::ToggleFinished(err) => {
                if let Err(err) = err {
                    self.set_error(err);
                } else {
                    self.update_mod_index();
                }
            }
            ManageModsMessage::UpdateMods => return self.update_mods(),
            ManageModsMessage::UpdateModsFinished(result) => {
                if let Err(err) = result {
                    self.set_error(err);
                } else {
                    self.update_mod_index();
                    if let State::EditMods(menu) = &mut self.state {
                        menu.available_updates.clear();
                    }
                    return Task::perform(
                        ql_mod_manager::store::check_for_updates(
                            self.selected_instance.clone().unwrap(),
                        ),
                        |n| Message::ManageMods(ManageModsMessage::UpdateCheckResult(n.strerr())),
                    );
                }
            }
            ManageModsMessage::UpdateCheckResult(updates) => {
                if let State::EditMods(menu) = &mut self.state {
                    menu.update_check_handle = None;
                    match updates {
                        Ok(updates) => {
                            menu.available_updates =
                                updates.into_iter().map(|(a, b)| (a, b, true)).collect();
                        }
                        Err(err) => {
                            err_no_log!("Could not check for updates: {err}");
                        }
                    }
                }
            }
            ManageModsMessage::UpdateCheckToggle(idx, t) => {
                if let State::EditMods(MenuEditMods {
                    available_updates, ..
                }) = &mut self.state
                {
                    if let Some((_, _, b)) = available_updates.get_mut(idx) {
                        *b = t;
                    }
                }
            }
            ManageModsMessage::SelectAll => {
                if let State::EditMods(menu) = &mut self.state {
                    match menu.selected_state {
                        SelectedState::All => {
                            menu.selected_mods.clear();
                            menu.selected_state = SelectedState::None;
                        }
                        SelectedState::Some | SelectedState::None => {
                            menu.selected_mods = menu
                                .mods
                                .mods
                                .iter()
                                .filter_map(|(id, mod_info)| {
                                    mod_info
                                        .manually_installed
                                        .then_some(SelectedMod::Downloaded {
                                            name: mod_info.name.clone(),
                                            id: ModId::from_index_str(id),
                                        })
                                })
                                .chain(menu.locally_installed_mods.iter().map(|n| {
                                    SelectedMod::Local {
                                        file_name: n.clone(),
                                    }
                                }))
                                .collect();
                            menu.selected_state = SelectedState::All;
                        }
                    }
                }
            }
            ManageModsMessage::ExportMenuOpen => {
                if let State::EditMods(menu) = &mut self.state {
                    // Navigate to the export menu with the current selection and mod data
                    use crate::state::MenuExportMods;

                    self.state = State::ExportMods(MenuExportMods {
                        selected_mods: if menu.selected_mods.is_empty() {
                            menu.mods
                                .mods
                                .iter()
                                .filter_map(|(id, mod_info)| {
                                    mod_info
                                        .manually_installed
                                        .then_some(SelectedMod::Downloaded {
                                            name: mod_info.name.clone(),
                                            id: ModId::from_index_str(id),
                                        })
                                })
                                .chain(menu.locally_installed_mods.iter().map(|n| {
                                    SelectedMod::Local {
                                        file_name: n.clone(),
                                    }
                                }))
                                .collect()
                        } else {
                            menu.selected_mods.clone()
                        },
                    });
                }
            }
        }
        Task::none()
    }

    fn get_delete_mods_command(
        selected_instance: InstanceSelection,
        menu: &MenuEditMods,
    ) -> Task<Message> {
        let ids: Vec<ModId> = menu
            .selected_mods
            .iter()
            .filter_map(|s_mod| {
                if let SelectedMod::Downloaded { id, .. } = s_mod {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();

        Task::perform(
            ql_mod_manager::store::delete_mods(ids, selected_instance),
            |n| Message::ManageMods(ManageModsMessage::DeleteFinished(n.strerr())),
        )
    }

    fn update_mod_index(&mut self) {
        if let State::EditMods(menu) = &mut self.state {
            match block_on(ModIndex::load(self.selected_instance.as_ref().unwrap())).strerr() {
                Ok(idx) => menu.mods = idx,
                Err(err) => self.set_error(err),
            }
        }
    }

    pub fn update_manage_jar_mods(&mut self, msg: ManageJarModsMessage) -> Task<Message> {
        match msg {
            ManageJarModsMessage::Open => {
                self.state = State::EditJarMods(MenuEditJarMods {
                    jarmods: None,
                    selected_state: SelectedState::None,
                    selected_mods: HashSet::new(),
                    drag_and_drop_hovered: false,
                    free_for_autosave: true,
                });

                let instance = self.selected_instance.clone().unwrap();
                return Task::perform(async move { JarMods::get(&instance).await }, |n| {
                    Message::ManageJarMods(ManageJarModsMessage::Loaded(n.strerr()))
                });
            }
            ManageJarModsMessage::Loaded(Err(err)) => {
                self.set_error(err);
            }
            ManageJarModsMessage::Loaded(Ok(jarmods)) => {
                if let State::EditJarMods(menu) = &mut self.state {
                    menu.jarmods = Some(jarmods);
                }
            }
            ManageJarModsMessage::AddFile => {
                self.manage_jarmods_add_file_from_picker();
            }
            ManageJarModsMessage::ToggleCheckbox(name, enable) => {
                self.manage_jarmods_toggle_checkbox(name, enable);
            }
            ManageJarModsMessage::DeleteSelected => {
                self.manage_jarmods_delete_selected();
            }
            ManageJarModsMessage::ToggleSelected => {
                self.manage_jarmods_toggle_selected();
            }
            ManageJarModsMessage::SelectAll => {
                self.manage_jarmods_select_all();
            }
            ManageJarModsMessage::AutosaveFinished((res, jarmods)) => {
                if let Err(err) = res {
                    self.set_error(format!("While autosaving jarmods index: {err}"));
                } else if let State::EditJarMods(menu) = &mut self.state {
                    // Some cleanup of jarmods state may happen during autosave
                    menu.jarmods = Some(jarmods);
                    menu.free_for_autosave = true;
                }
            }

            ManageJarModsMessage::MoveUp | ManageJarModsMessage::MoveDown => {
                self.manage_jarmods_move_up_or_down(&msg);
            }
        }
        Task::none()
    }

    fn manage_jarmods_move_up_or_down(&mut self, msg: &ManageJarModsMessage) {
        if let State::EditJarMods(MenuEditJarMods {
            jarmods: Some(jarmods),
            selected_mods,
            ..
        }) = &mut self.state
        {
            let mut selected: Vec<usize> = selected_mods
                .iter()
                .filter_map(|selected_name| {
                    jarmods
                        .mods
                        .iter()
                        .enumerate()
                        .find_map(|(i, n)| (n.filename == *selected_name).then_some(i))
                })
                .collect();
            selected.sort_unstable();
            if let ManageJarModsMessage::MoveDown = msg {
                selected.reverse();
            }

            for i in selected {
                if i < jarmods.mods.len() {
                    match msg {
                        ManageJarModsMessage::MoveUp => {
                            if i > 0 {
                                let removed = jarmods.mods.remove(i);
                                jarmods.mods.insert(i - 1, removed);
                            }
                        }
                        ManageJarModsMessage::MoveDown => {
                            if i + 1 < jarmods.mods.len() {
                                let removed = jarmods.mods.remove(i);
                                jarmods.mods.insert(i + 1, removed);
                            }
                        }
                        _ => {}
                    }
                } else {
                    err!(
                        "Out of bounds in jarmods move up/down: !({i} < len:{})",
                        jarmods.mods.len()
                    );
                }
            }
        }
    }

    fn manage_jarmods_select_all(&mut self) {
        if let State::EditJarMods(MenuEditJarMods {
            jarmods: Some(jarmods),
            selected_state,
            selected_mods,
            ..
        }) = &mut self.state
        {
            match selected_state {
                SelectedState::All => {
                    selected_mods.clear();
                    *selected_state = SelectedState::None;
                }
                SelectedState::Some | SelectedState::None => {
                    *selected_mods = jarmods
                        .mods
                        .iter()
                        .map(|mod_info| mod_info.filename.clone())
                        .collect();
                    *selected_state = SelectedState::All;
                }
            }
        }
    }

    fn manage_jarmods_toggle_selected(&mut self) {
        if let State::EditJarMods(MenuEditJarMods {
            jarmods: Some(jarmods),
            selected_mods,
            ..
        }) = &mut self.state
        {
            for selected in selected_mods.iter() {
                if let Some(jarmod) = jarmods.mods.iter_mut().find(|n| n.filename == *selected) {
                    jarmod.enabled = !jarmod.enabled;
                }
            }
        }
    }

    fn manage_jarmods_delete_selected(&mut self) {
        if let State::EditJarMods(MenuEditJarMods {
            jarmods: Some(jarmods),
            selected_mods,
            ..
        }) = &mut self.state
        {
            let jarmods_path = self
                .selected_instance
                .as_ref()
                .unwrap()
                .get_instance_path()
                .join("jarmods");

            for selected in selected_mods.iter() {
                if let Some(n) = jarmods
                    .mods
                    .iter()
                    .enumerate()
                    .find_map(|(i, n)| (n.filename == *selected).then_some(i))
                {
                    jarmods.mods.remove(n);
                }

                let path = jarmods_path.join(selected);
                if path.is_file() {
                    _ = std::fs::remove_file(&path);
                }
            }

            selected_mods.clear();
        }
    }

    fn manage_jarmods_toggle_checkbox(&mut self, name: String, enable: bool) {
        if let State::EditJarMods(menu) = &mut self.state {
            if enable {
                menu.selected_mods.insert(name);
                menu.selected_state = SelectedState::Some;
            } else {
                menu.selected_mods.remove(&name);
                menu.selected_state = if menu.selected_mods.is_empty() {
                    SelectedState::None
                } else {
                    SelectedState::Some
                };
            }
        }
    }

    fn export_mods_markdown(selected_mods: &HashSet<SelectedMod>) -> String {
        let mut markdown_lines = Vec::new();

        for selected_mod in selected_mods {
            match selected_mod {
                SelectedMod::Downloaded { name, id } => {
                    let url = match id {
                        ModId::Modrinth(mod_id) => {
                            format!("https://modrinth.com/mod/{mod_id}")
                        }
                        ModId::Curseforge(mod_id) => {
                            format!("https://www.curseforge.com/projects/{mod_id}")
                        }
                    };
                    markdown_lines.push(format!("- [{name}]({url})"));
                }
                SelectedMod::Local { file_name } => {
                    let display_name = file_name
                        .strip_suffix(".jar")
                        .or_else(|| file_name.strip_suffix(".zip"))
                        .unwrap_or(file_name);
                    markdown_lines.push(display_name.to_string());
                }
            }
        }

        markdown_lines.join("\n")
    }

    fn export_mods_html(selected_mods: &HashSet<SelectedMod>) -> String {
        let mut html_content = String::new();
        
        // HTML document structure
        html_content.push_str("<!DOCTYPE html>\n");
        html_content.push_str("<html lang=\"en\">\n");
        html_content.push_str("<head>\n");
        html_content.push_str("    <meta charset=\"UTF-8\">\n");
        html_content.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html_content.push_str("    <title>Mod List</title>\n");
        html_content.push_str("    <style>\n");
        html_content.push_str("        body { font-family: Arial, sans-serif; margin: 20px; background-color: #f5f5f5; }\n");
        html_content.push_str("        .container { max-width: 800px; margin: 0 auto; background-color: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }\n");
        html_content.push_str("        h1 { color: #333; text-align: center; border-bottom: 2px solid #4CAF50; padding-bottom: 10px; }\n");
        html_content.push_str("        .mod-list { list-style: none; padding: 0; }\n");
        html_content.push_str("        .mod-item { margin: 10px 0; padding: 12px; background-color: #f9f9f9; border-radius: 4px; border-left: 4px solid #4CAF50; }\n");
        html_content.push_str("        .mod-link { color: #2196F3; text-decoration: none; font-weight: bold; }\n");
        html_content.push_str("        .mod-link:hover { text-decoration: underline; }\n");
        html_content.push_str("        .local-mod { color: #666; font-style: italic; }\n");
        html_content.push_str("        .mod-count { color: #888; text-align: center; margin-bottom: 20px; }\n");
        html_content.push_str("    </style>\n");
        html_content.push_str("</head>\n");
        html_content.push_str("<body>\n");
        html_content.push_str("    <div class=\"container\">\n");
        html_content.push_str("        <h1>Minecraft Mod List</h1>\n");
        html_content.push_str(&format!("        <p class=\"mod-count\">{} mod{} installed</p>\n", 
            selected_mods.len(), 
            if selected_mods.len() == 1 { "" } else { "s" }
        ));
        html_content.push_str("        <ul class=\"mod-list\">\n");

        for selected_mod in selected_mods {
            match selected_mod {
                SelectedMod::Downloaded { name, id } => {
                    let url = match id {
                        ModId::Modrinth(mod_id) => {
                            format!("https://modrinth.com/mod/{mod_id}")
                        }
                        ModId::Curseforge(mod_id) => {
                            format!("https://www.curseforge.com/minecraft/mc-mods/{mod_id}")
                        }
                    };
                    html_content.push_str(&format!(
                        "            <li class=\"mod-item\"><a href=\"{}\" class=\"mod-link\" target=\"_blank\">{}</a></li>\n",
                        url, name
                    ));
                }
                SelectedMod::Local { file_name } => {
                    let display_name = file_name
                        .strip_suffix(".jar")
                        .or_else(|| file_name.strip_suffix(".zip"))
                        .unwrap_or(file_name);
                    html_content.push_str(&format!(
                        "            <li class=\"mod-item\"><span class=\"local-mod\">{} (local)</span></li>\n",
                        display_name
                    ));
                }
            }
        }

        html_content.push_str("        </ul>\n");
        html_content.push_str("        <p style=\"text-align: center; color: #888; margin-top: 30px; font-size: 0.9em;\">Generated by QuantumLauncher</p>\n");
        html_content.push_str("    </div>\n");
        html_content.push_str("</body>\n");
        html_content.push_str("</html>");

        html_content
    }

    fn export_plain_text_to_file(content: String) -> Task<Message> {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Save mod list as Plain Text")
            .add_filter("Text files", &["txt"])
            .set_file_name("mod_list.txt")
            .save_file()
        {
            match std::fs::write(&path, content) {
                Ok(()) => Task::none(),
                Err(_err) => Task::none(),
            }
        } else {
            Task::none()
        }
    }

    fn export_markdown_to_file(content: String) -> Task<Message> {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Save mod list as Markdown")
            .add_filter("Markdown files", &["md"])
            .set_file_name("mod_list.md")
            .save_file()
        {
            match std::fs::write(&path, content) {
                Ok(()) => Task::none(),
                Err(_err) => Task::none(),
            }
        } else {
            Task::none()
        }
    }

    fn export_html_to_file(content: String) -> Task<Message> {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Save mod list as HTML")
            .add_filter("HTML files", &["html"])
            .set_file_name("mod_list.html")
            .save_file()
        {
            match std::fs::write(&path, content) {
                Ok(()) => Task::none(),
                Err(_err) => Task::none(),
            }
        } else {
            Task::none()
        }
    }

    fn manage_jarmods_add_file_from_picker(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("jar/zip", &["jar", "zip"])
            .set_title("Pick a Jar Mod Patch (.jar/.zip)")
            .pick_file()
        {
            if let Some(filename) = path.file_name() {
                let dest = self
                    .selected_instance
                    .as_ref()
                    .unwrap()
                    .get_instance_path()
                    .join("jarmods")
                    .join(filename);
                if let Err(err) = std::fs::copy(&path, dest) {
                    self.set_error(format!("While picking jar mod to be added: {err}"));
                }
            }
        }
    }

    pub fn update_export_mods(&mut self, msg: crate::state::ExportModsMessage) -> Task<Message> {
        use crate::state::ExportModsMessage;
        match msg {
            ExportModsMessage::ExportAsPlainText => {
                if let State::ExportMods(menu) = &self.state {
                    return Self::export_plain_text_to_file(Self::export_mods_plain_text(&menu.selected_mods));
                }
            }
            ExportModsMessage::ExportAsMarkdown => {
                if let State::ExportMods(menu) = &self.state {
                    return Self::export_markdown_to_file(Self::export_mods_markdown(&menu.selected_mods));
                }
            }
            ExportModsMessage::ExportAsHtml => {
                if let State::ExportMods(menu) = &self.state {
                    return Self::export_html_to_file(Self::export_mods_html(&menu.selected_mods));
                }
            }
            ExportModsMessage::CopyMarkdownToClipboard => {
                if let State::ExportMods(menu) = &self.state {
                    return iced::clipboard::write(Self::export_mods_markdown(&menu.selected_mods));
                }
            }
            ExportModsMessage::CopyPlainTextToClipboard => {
                if let State::ExportMods(menu) = &self.state {
                    return iced::clipboard::write(Self::export_mods_plain_text(
                        &menu.selected_mods,
                    ));
                }
            }
            ExportModsMessage::CopyHtmlToClipboard => {
                if let State::ExportMods(menu) = &self.state {
                    return iced::clipboard::write(Self::export_mods_html(&menu.selected_mods));
                }
            }
        }
        Task::none()
    }

    fn export_mods_plain_text(selected_mods: &HashSet<SelectedMod>) -> String {
        let mut lines = Vec::new();

        for selected_mod in selected_mods {
            match selected_mod {
                SelectedMod::Downloaded { name, .. } => {
                    lines.push(name.clone());
                }
                SelectedMod::Local { file_name } => {
                    // Remove file extension for cleaner display
                    let display_name = file_name
                        .strip_suffix(".jar")
                        .or_else(|| file_name.strip_suffix(".zip"))
                        .unwrap_or(file_name);
                    lines.push(display_name.to_string());
                }
            }
        }
        lines.join("\n")
    }
}

async fn delete_file_wrapper(path: PathBuf) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    tokio::fs::remove_file(&path).await.path(path).strerr()
}
