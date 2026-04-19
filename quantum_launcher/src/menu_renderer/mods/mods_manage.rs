use crate::{
    icons,
    menu_renderer::{
        CTXI_SIZE, Column, Element, FONT_MONO, back_to_launch_screen, barthin, button_with_icon,
        ctx_button, ctx_button_empty, ctx_button_icon, dots, offsetbox, select_box,
        subbutton_with_icon, tooltip, tsubtitle, view_info_message,
    },
    message_handler::ForgeKind,
    state::{
        EditPresetsMessage, ImageState, InstallFabricMessage, InstallModsMessage,
        InstallOptifineMessage, InstallPaperMessage, ManageJarModsMessage, ManageModsMessage,
        MenuEditMods, MenuEditModsModal, Message, ModDescriptionMessage, ModListEntry,
        SelectedState,
    },
    stylesheet::{color::Color, styles::LauncherTheme, widgets::StyleButton},
};
use iced::{
    Alignment, Length,
    widget::{self, column, row, tooltip::Position},
};
use ql_core::{
    Instance, InstanceKind, Loader,
    json::{InstanceConfigJson, V_LAST_TEXTUREPACK},
};
use ql_mod_manager::store::{QueryType, SelectedMod};

pub const MODS_SIDEBAR_WIDTH: u16 = 190;

impl MenuEditMods {
    pub fn view<'a>(
        &'a self,
        selected_instance: &'a Instance,
        tick_timer: usize,
        images: &'a ImageState,
        window_height: f32,
    ) -> Element<'a> {
        if let Some(progress) = &self.mod_update_progress {
            return column![widget::text("Updating mods").size(20), progress.view()]
                .padding(10)
                .spacing(10)
                .into();
        }

        let menu_main = widget::Column::new()
            .push_maybe(
                self.info_message
                    .as_ref()
                    .map(|n| view_info_message(n, ManageModsMessage::SetInfoMessage(None).into())),
            )
            .push_maybe(self.info_message.as_ref().map(|_| {
                widget::horizontal_rule(2)
                    .style(|t: &LauncherTheme| t.style_rule(Color::SecondDark, 1))
            }))
            .push(row![
                self.get_sidebar(selected_instance, tick_timer),
                self.get_mod_list(images)
            ]);

        if self.drag_and_drop_hovered {
            return widget::stack!(
                menu_main,
                widget::center(widget::button(
                    widget::text("Drag and drop mod files to add them").size(20)
                ))
            )
            .into();
        }

        match &self.modal {
            Some(MenuEditModsModal::Submenu) => {
                let submenu = column![
                    ctx_button_icon(icons::refresh_s(CTXI_SIZE), "Check for updates")
                        .on_press(ManageModsMessage::UpdateCheck.into()),
                    ctx_button_icon(icons::file_info_s(CTXI_SIZE), "Export list as text")
                        .on_press(ManageModsMessage::ExportMenuOpen.into()),
                    ctx_button_icon(icons::file_zip_s(CTXI_SIZE), "Export QMP Preset")
                        .on_press(EditPresetsMessage::Open.into()),
                    widget::horizontal_rule(1).style(barthin),
                    ctx_button_icon(icons::download_s(CTXI_SIZE), "See recommended mods").on_press(
                        Message::RecommendedMods(crate::state::RecommendedModMessage::Open)
                    ),
                ]
                .spacing(4);

                offsetbox(menu_main, submenu, MODS_SIDEBAR_WIDTH + 30, 40, 200).into()
            }
            Some(MenuEditModsModal::RightClick(id, (x, y))) => offsetbox(
                menu_main,
                column![
                    ctx_button_icon(icons::toggleon_s(CTXI_SIZE), "Toggle")
                        .on_press(ManageModsMessage::ToggleSelected.into()),
                    ctx_button_icon(icons::bin_s(CTXI_SIZE), "Delete")
                        .on_press(ManageModsMessage::DeleteSelected.into()),
                    ctx_button_icon(icons::file_info_s(CTXI_SIZE), "Mod Details")
                        .on_press_with(|| ModDescriptionMessage::Open(id.clone()).into()),
                ]
                .spacing(4),
                *x,
                y.clamp(0.0, window_height - 130.0),
                150,
            )
            .into(),
            Some(MenuEditModsModal::FolderMenu) => {
                let folder_menu = column![
                    ctx_button("Mods Folder").on_press_with(|| {
                        Message::CoreOpenPath(
                            selected_instance.get_dot_minecraft_path().join("mods"),
                        )
                    }),
                    ctx_button("Resource Packs Folder").on_press_with(|| Message::CoreOpenPath(
                        selected_instance.get_dot_minecraft_path().join(
                            if self.version_json.is_before_or_eq(V_LAST_TEXTUREPACK) {
                                "texturepacks"
                            } else {
                                "resourcepacks"
                            }
                        )
                    )),
                    ctx_button("Shaders Folder").on_press_with(|| {
                        Message::CoreOpenPath(
                            selected_instance
                                .get_dot_minecraft_path()
                                .join("shaderpacks"),
                        )
                    }),
                    ctx_button("Datapacks Folder").on_press_with(|| {
                        Message::CoreOpenPath(
                            selected_instance.get_dot_minecraft_path().join("datapacks"),
                        )
                    }),
                ]
                .spacing(4);

                offsetbox(menu_main, folder_menu, 30, 40, 200).into()
            }
            Some(MenuEditModsModal::AddFile) => {
                let addfile_msg = |q| Message::ManageMods(ManageModsMessage::AddFile(false, q));

                let menu = column![
                    ctx_button_icon(icons::file_jar_s(CTXI_SIZE), "Mod")
                        .on_press(addfile_msg(QueryType::Mods)),
                    ctx_button_empty("Shader Pack").on_press(addfile_msg(QueryType::Shaders)),
                    ctx_button_empty("Resource Pack")
                        .on_press(addfile_msg(QueryType::ResourcePacks)),
                    ctx_button_empty("Datapack").on_press(addfile_msg(QueryType::DataPacks)),
                    ctx_button_empty("Modpack/QMP").on_press(addfile_msg(QueryType::ModPacks)),
                ]
                .spacing(4);

                offsetbox(menu_main, menu, 30, 70, 150).into()
            }
            None => menu_main.into(),
        }
    }

    fn get_sidebar<'a>(
        &'a self,
        selected_instance: &'a Instance,
        tick_timer: usize,
    ) -> widget::Scrollable<'a, Message, LauncherTheme> {
        const TOP_PADDING: iced::Padding = iced::Padding {
            top: 5.0,
            right: 8.0,
            bottom: 5.0,
            left: 8.0,
        };

        let open_folders_btn = button_with_icon(icons::folder_s(14), "Open...", 14)
            .padding(TOP_PADDING)
            .on_press(
                ManageModsMessage::SetModal(
                    if let Some(MenuEditModsModal::FolderMenu) = self.modal {
                        None
                    } else {
                        Some(MenuEditModsModal::FolderMenu)
                    },
                )
                .into(),
            );

        // .on_press(ManageModsMessage::AddFile(false).into())
        let add_file_btn = button_with_icon(icons::file_s(14), "Add File...", 14)
            .padding(TOP_PADDING)
            .on_press(
                ManageModsMessage::SetModal(if let Some(MenuEditModsModal::AddFile) = self.modal {
                    None
                } else {
                    Some(MenuEditModsModal::AddFile)
                })
                .into(),
            );

        widget::scrollable(
            column![
                column![
                    row![
                        button_with_icon(icons::back_s(13), "Back", 14)
                            .padding(TOP_PADDING)
                            .on_press(back_to_launch_screen(None)),
                        open_folders_btn,
                    ]
                    .spacing(5),
                    add_file_btn,
                ]
                .spacing(5),
                self.get_mod_installer_buttons(selected_instance.kind),
                column![
                    button_with_icon(icons::download_s(15), "Download Content...", 14)
                        .on_press(InstallModsMessage::Open.into()),
                    button_with_icon(icons::file_jar(), "Jarmod Patches", 14)
                        .on_press(ManageJarModsMessage::Open.into()),
                ]
                .spacing(5),
                self.get_mod_update_pane(tick_timer),
            ]
            .padding(10)
            .spacing(10),
        )
        .style(LauncherTheme::style_scrollable_flat_dark)
        .height(Length::Fill)
    }

    fn get_mod_update_pane(&'_ self, tick_timer: usize) -> Column<'_> {
        if self.update_check_handle.is_some() {
            column![widget::text!("Checking for mod updates{}", dots(tick_timer)).size(12)]
        } else if self.available_updates.is_empty() {
            column![]
        } else {
            column![
                widget::horizontal_rule(1),
                widget::text("Mod Updates Available!").size(15),
                widget::column(self.available_updates.iter().enumerate().map(
                    |(i, (id, update_name, is_enabled))| {
                        let title = self.mods.mods.get(id).map(|n| &*n.name).unwrap_or_default();

                        let toggle = move |b| ManageModsMessage::UpdateCheckToggle(i, b).into();

                        widget::mouse_area(row![
                            widget::checkbox("", *is_enabled).on_toggle(toggle),
                            column![
                                widget::text(title).size(12),
                                widget::text!("{update_name}").size(10).style(tsubtitle)
                            ]
                        ])
                        .on_press(toggle(!*is_enabled))
                        .into()
                    }
                ))
                .spacing(5),
                button_with_icon(icons::version_download(), "Update", 16)
                    .on_press(ManageModsMessage::UpdatePerform.into()),
            ]
            .padding(5)
            .spacing(10)
            .width(MODS_SIDEBAR_WIDTH)
        }
    }

    fn get_mod_installer_buttons(&'_ self, kind: InstanceKind) -> Element<'_> {
        match self.config.mod_type {
            Loader::Vanilla => match kind {
                InstanceKind::Client => column![
                    "Install:",
                    row![
                        install_ldr("Fabric")
                            .on_press(InstallFabricMessage::ScreenOpen { is_quilt: false }.into()),
                        install_ldr("Quilt")
                            .on_press(InstallFabricMessage::ScreenOpen { is_quilt: true }.into()),
                    ]
                    .spacing(5),
                    row![
                        install_ldr("Forge").on_press(Message::InstallForge(ForgeKind::Normal)),
                        install_ldr("NeoForge")
                            .on_press(Message::InstallForge(ForgeKind::NeoForge))
                    ]
                    .spacing(5),
                    install_ldr("OptiFine").on_press(InstallOptifineMessage::ScreenOpen.into())
                ]
                .spacing(5)
                .into(),
                InstanceKind::Server => column![
                    "Install:",
                    row![
                        install_ldr("Fabric")
                            .on_press(InstallFabricMessage::ScreenOpen { is_quilt: false }.into()),
                        install_ldr("Quilt")
                            .on_press(InstallFabricMessage::ScreenOpen { is_quilt: true }.into()),
                    ]
                    .spacing(5),
                    row![
                        install_ldr("Forge").on_press(Message::InstallForge(ForgeKind::Normal)),
                        install_ldr("NeoForge")
                            .on_press(Message::InstallForge(ForgeKind::NeoForge))
                    ]
                    .spacing(5),
                    row![
                        widget::button("Bukkit").width(97),
                        widget::button("Spigot").width(97)
                    ]
                    .spacing(5),
                    install_ldr("Paper")
                        .on_press(Message::InstallPaper(InstallPaperMessage::ScreenOpen)),
                ]
                .spacing(5)
                .into(),
            },

            Loader::Forge => widget::Column::new()
                .push_maybe(
                    matches!(kind, InstanceKind::Client)
                        .then(|| Self::get_optifine_install_button(&self.config)),
                )
                .push(Self::get_uninstall_panel(self.config.mod_type))
                .spacing(5)
                .into(),
            Loader::OptiFine => column![
                widget::button(widget::text("Install Forge with OptiFine").size(14))
                    .on_press(Message::InstallForge(ForgeKind::OptiFine)),
                Self::get_uninstall_panel(self.config.mod_type),
            ]
            .spacing(5)
            .into(),

            Loader::Neoforge | Loader::Fabric | Loader::Quilt | Loader::Paper => {
                Self::get_uninstall_panel(self.config.mod_type).into()
            }

            _ => widget::text!("Unknown mod type: {}", self.config.mod_type).into(),
        }
    }

    fn get_optifine_install_button(
        config: &InstanceConfigJson,
    ) -> widget::Button<'static, Message, LauncherTheme> {
        if let Some(optifine) = config
            .mod_type_info
            .as_ref()
            .and_then(|n| n.optifine_jar.clone())
        {
            widget::button(
                row![
                    icons::bin_s(14),
                    widget::text("Uninstall OptiFine").size(14)
                ]
                .align_y(Alignment::Center)
                .spacing(11)
                .padding(2),
            )
            .on_press_with(move || {
                Message::UninstallLoaderConfirm(
                    Box::new(ManageModsMessage::DeleteOptiforge(optifine.clone()).into()),
                    Loader::OptiFine,
                )
            })
        } else {
            widget::button(widget::text("Install OptiFine with Forge").size(14))
                .on_press(InstallOptifineMessage::ScreenOpen.into())
        }
    }

    fn get_uninstall_panel(mod_type: Loader) -> widget::Button<'static, Message, LauncherTheme> {
        widget::button(
            row![
                icons::bin_s(14),
                widget::text!("Uninstall {mod_type}").size(14)
            ]
            .align_y(Alignment::Center)
            .spacing(11)
            .padding(2),
        )
        .on_press(Message::UninstallLoaderConfirm(
            Box::new(Message::UninstallLoaderStart),
            mod_type,
        ))
    }

    fn get_mod_list<'a>(&'a self, images: &'a ImageState) -> Element<'a> {
        if self.sorted_mods_list.is_empty() {
            return column![
                "Download some mods to get started",
                widget::button(widget::text("View Recommended Mods").size(14))
                    .on_press(crate::state::RecommendedModMessage::Open.into())
            ]
            .spacing(10)
            .padding(10)
            .width(Length::Fill)
            .into();
        }

        let hamburger_dropdown = widget::button(
            row![icons::lines_s(12)]
                .align_y(Alignment::Center)
                .padding(1),
        );

        let hamburger_dropdown = if let Some(MenuEditModsModal::Submenu) = self.modal {
            hamburger_dropdown.on_press(ManageModsMessage::SetModal(None).into())
        } else {
            hamburger_dropdown
                .on_press(ManageModsMessage::SetModal(Some(MenuEditModsModal::Submenu)).into())
                .style(|t: &LauncherTheme, s| t.style_button(s, StyleButton::RoundDark))
        };

        widget::container(
            column![
                widget::Column::new()
                    .push_maybe(
                        (self.config.mod_type.is_vanilla() && !self.sorted_mods_list.is_empty())
                        .then_some(
                            widget::container(
                                widget::text(
                                    // WARN: No loader installed
                                    "You haven't installed any mod loader! Install Fabric/Forge/Quilt/NeoForge as per your mods"
                                ).size(12)
                            ).padding(10).width(Length::Fill).style(|n: &LauncherTheme| n.style_container_sharp_box(0.0, Color::ExtraDark)),
                        )
                    )
                    .push(
                        row![
                            hamburger_dropdown,

                            // Search button
                            widget::button(
                                row![icons::search_s(12)]
                                    .align_y(Alignment::Center)
                                    .padding(1),
                            )
                            .style(|t: &LauncherTheme, s| {
                                t.style_button(s, if self.search.is_some() {
                                    StyleButton::Round
                                } else {
                                    StyleButton::RoundDark
                                })
                            }).on_press(
                                if self.search.is_some() {
                                    ManageModsMessage::SetSearch(None).into()
                                } else {
                                    Message::Multiple(vec![
                                        ManageModsMessage::SetSearch(
                                            Some(String::new())
                                        ).into(),
                                        Message::CoreFocusNext
                                    ])
                                }
                            ),

                            subbutton_with_icon(icons::bin_s(12), "Delete")
                            .on_press_maybe((!self.selected_mods.is_empty()).then_some(ManageModsMessage::DeleteSelected.into())),
                            subbutton_with_icon(icons::toggleoff_s(12), "Toggle")
                                .on_press(ManageModsMessage::ToggleSelected.into()),
                            subbutton_with_icon(icons::deselectall_s(12), if matches!(self.selected_state, SelectedState::All) {
                                "Unselect All"
                            } else {
                                "Select All"
                            })
                            .on_press(ManageModsMessage::SelectAll.into()),
                        ]
                        .spacing(5)
                        .wrap()
                    )
                    // Content filter row
                    .push(
                        self.get_content_filters()
                    )
                    .push(
                        if self.selected_mods.is_empty() {
                            widget::text("Select some content to perform actions on them")
                        } else {
                            widget::text!("{} items selected", self.selected_mods.len())
                        }
                        .size(10)
                        .style(|t: &LauncherTheme| t.style_text(Color::Mid))
                    )
                    .push_maybe(self.search.as_ref().map(|search|
                        widget::text_input("Search...", search).size(14).on_input(|msg|
                            ManageModsMessage::SetSearch(Some(msg)).into()
                        )
                    ))
                    .padding(10)
                    .spacing(5),
                widget::responsive(|s| self.get_mod_list_contents(s, images)),
            ],
        )
        .style(|n| n.style_container_sharp_box(0.0, Color::ExtraDark))
        .into()
    }

    fn get_content_filters(&self) -> Element<'_> {
        fn query_button(
            label: widget::Text<'_, LauncherTheme>,
            is_selected: bool,
            filter: Option<QueryType>,
        ) -> widget::Button<'_, Message, LauncherTheme> {
            widget::button(label.size(10))
                .padding([2, 4])
                .style(move |t: &LauncherTheme, s| {
                    t.style_button(
                        s,
                        if is_selected {
                            StyleButton::Round
                        } else {
                            StyleButton::RoundDark
                        },
                    )
                })
                .on_press(Message::ManageMods(ManageModsMessage::SetContentFilter(
                    filter,
                )))
        }

        row![query_button(
            widget::text("All"),
            self.content_filter.is_none(),
            None
        )]
        .extend(QueryType::ALL.iter().map(|filter| {
            let is_selected = self.content_filter.is_some_and(|n| n == *filter);
            query_button(widget::text(filter.to_string()), is_selected, Some(*filter)).into()
        }))
        .spacing(4)
        .wrap()
        .into()
    }

    fn get_mod_list_contents<'a>(
        &'a self,
        size: iced::Size,
        images: &'a ImageState,
    ) -> Element<'a> {
        widget::scrollable(widget::column(
            self.sorted_mods_list
                .iter()
                .filter(|n| {
                    if self.content_filter.is_some_and(|f| f != n.project_type()) {
                        return false;
                    }
                    let Some(search) = &self.search else {
                        return true;
                    };
                    n.name().to_lowercase().contains(&search.to_lowercase())
                })
                .map(|mod_list_entry| self.get_mod_entry(mod_list_entry, size, images)),
        ))
        .direction(widget::scrollable::Direction::Both {
            vertical: widget::scrollable::Scrollbar::new(),
            horizontal: widget::scrollable::Scrollbar::new(),
        })
        .id(widget::scrollable::Id::new("MenuEditMods:mods"))
        .on_scroll(|viewport| ManageModsMessage::ListScrolled(viewport.absolute_offset()).into())
        .style(LauncherTheme::style_scrollable_flat_extra_dark)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn get_mod_entry<'a>(
        &'a self,
        entry: &'a ModListEntry,
        size: iced::Size,
        images: &'a ImageState,
    ) -> Element<'a> {
        const PADDING: iced::Padding = iced::Padding {
            top: 4.0,
            bottom: 6.0,
            right: 15.0,
            left: 20.0,
        };
        const ICON_SIZE: f32 = 18.0;
        const SPACING: u16 = 16;

        let no_icon = widget::Column::new()
            .width(ICON_SIZE)
            .height(ICON_SIZE)
            .into();

        match entry {
            ModListEntry::Downloaded { id, config } => {
                if config.manually_installed {
                    let is_enabled = config.enabled;
                    let is_selected = self.selected_mods.contains(&SelectedMod::Downloaded {
                        name: config.name.clone(),
                        id: (*id).clone(),
                    });

                    let image: Element = if let Some(url) = &config.icon_url {
                        images.view(Some(url), Some(ICON_SIZE), Some(ICON_SIZE))
                    } else {
                        no_icon
                    };

                    let toggle: Element = if entry.project_type().is_toggleable() {
                        widget::toggler(is_enabled)
                            .on_toggle(move |_| ManageModsMessage::ToggleOne(id.clone()).into())
                            .size(14)
                            .into()
                    } else {
                        widget::Space::with_width(10).into()
                    };

                    let entry = select_box(
                        widget::row![
                            toggle,
                            image,
                            widget::Space::with_width(1),
                            widget::text(&*config.name)
                                .shaping(widget::text::Shaping::Advanced)
                                .style(move |t: &LauncherTheme| {
                                    t.style_text(if is_enabled {
                                        Color::SecondLight
                                    } else {
                                        Color::Mid
                                    })
                                })
                                .size(14)
                                .width(self.width_name),
                            widget::text(&config.installed_version)
                                .style(|t: &LauncherTheme| t.style_text(Color::Mid))
                                .font(FONT_MONO)
                                .size(12)
                        ]
                        .push_maybe({
                            // Measure the length of the text
                            // then from there measure the space it would occupy
                            // (only possible because monospace font)

                            // This is for finding the filler space
                            //
                            // ║ Some Mod         v0.0.1                ║
                            // ║ Some other mod   2.4.1-fabric          ║
                            //
                            //  ╙═╦══════════════╜            ╙═╦══════╜
                            //  Measured by:                   What we want
                            //  `self.width_name`              to find

                            let measured: f32 = (config.installed_version.len() as f32) * 7.2;
                            let occupied =
                                measured + self.width_name + PADDING.left + PADDING.right + 150.0;
                            let space = size.width - occupied;
                            (space > -10.0).then_some(widget::Space::with_width(space))
                        })
                        .align_y(Alignment::Center)
                        .padding(PADDING)
                        .spacing(SPACING),
                        is_selected,
                        ManageModsMessage::SelectMod(
                            config.name.clone(),
                            Some(id.clone()),
                            config.project_type,
                        )
                        .into(),
                    )
                    .padding(0);

                    let rightclick = ManageModsMessage::RightClick(id.clone()).into();

                    widget::mouse_area(entry)
                        .on_right_press(if self.selected_mods.len() > 1 && self.is_selected(id) {
                            rightclick
                        } else {
                            Message::Multiple(vec![
                                ManageModsMessage::SelectEnsure(
                                    config.name.clone(),
                                    Some(id.clone()),
                                    config.project_type,
                                )
                                .into(),
                                rightclick,
                            ])
                        })
                        .into()
                } else {
                    row![
                        widget::text("(dependency) ")
                            .size(12)
                            .style(|t: &LauncherTheme| t.style_text(Color::Mid)),
                        widget::text(&*config.name)
                            .shaping(widget::text::Shaping::Advanced)
                            .size(13)
                            .style(tsubtitle)
                    ]
                    .padding(PADDING)
                    .into()
                }
            }
            ModListEntry::Local(local) => {
                let file_name = &*local.0;
                let project_type = local.1;

                let is_enabled = !file_name.ends_with(".disabled");
                let is_selected = self
                    .selected_mods
                    .contains(&SelectedMod::Local(local.clone()));

                let toggle: Element = if project_type.is_toggleable() {
                    widget::toggler(is_enabled)
                        .on_toggle(move |_| ManageModsMessage::ToggleOneLocal(local.clone()).into())
                        .size(14)
                        .into()
                } else {
                    widget::Space::with_width(36).into()
                };

                let label = format!(
                    "{}{}",
                    file_name.strip_suffix(".disabled").unwrap_or(file_name),
                    match project_type {
                        QueryType::Mods => "",
                        QueryType::Shaders => " (shader)",
                        QueryType::ModPacks => " (modpack)",
                        QueryType::DataPacks => " (datapack)",
                        QueryType::ResourcePacks => " (resourcepack)",
                    }
                );
                let label_len = label.len();

                let checkbox = select_box(
                    row![
                        toggle,
                        widget::text(label)
                            .font(FONT_MONO)
                            .shaping(widget::text::Shaping::Advanced)
                            .style(move |t: &LauncherTheme| {
                                t.style_text(if is_enabled {
                                    Color::SecondLight
                                } else {
                                    Color::Mid
                                })
                            })
                            .size(13)
                    ]
                    .push_maybe({
                        // Measure the length of the text
                        // then from there measure the space it would occupy
                        // (only possible because monospace font)

                        // This is for finding the filler space
                        //
                        // ║ some_mod.jar              ║
                        // ║ some_other_mod.jar        ║
                        //
                        //  ╙═╦═══════════════╜  ╙═╦═══╜
                        //  Measured by:         What we want
                        //  `label_len`          to find

                        let measured: f32 = (label_len as f32) * 7.2;
                        let occupied = measured + PADDING.left + PADDING.right + 100.0;
                        let space = size.width - occupied;
                        (space > 0.0).then_some(widget::Space::with_width(space))
                    })
                    .padding(PADDING)
                    .spacing(SPACING),
                    is_selected,
                    ManageModsMessage::SelectMod(local.0.clone(), None, project_type).into(),
                )
                .padding(0);

                if is_enabled {
                    checkbox.into()
                } else {
                    tooltip(checkbox, "Disabled", Position::FollowCursor).into()
                }
            }
        }
    }
}

fn install_ldr(loader: &str) -> widget::Button<'_, Message, LauncherTheme> {
    widget::button(widget::text(loader).size(14)).width(90)
}
