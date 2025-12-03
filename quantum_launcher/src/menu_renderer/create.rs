use iced::{
    widget::{self, column, row, tooltip::Position},
    Alignment, Length,
};
use ql_core::{ListEntry, ListEntryKind};

use crate::{
    icon_manager,
    menu_renderer::{back_button, button_with_icon, sidebar, sidebar_button, tooltip, Element},
    state::{CreateInstanceMessage, MenuCreateInstance, Message},
    stylesheet::{color::Color, styles::LauncherTheme, widgets::StyleButton},
};

impl MenuCreateInstance {
    pub fn view<'a>(
        &'a self,
        existing_instances: Option<&[String]>,
        versions: Option<&'a [ListEntry]>,
    ) -> Element<'a> {
        match self {
            MenuCreateInstance::LoadingList { .. } => column![
                row![
                    back_button().on_press(Message::CreateInstance(CreateInstanceMessage::Cancel)),
                    button_with_icon(icon_manager::folder_with_size(14), "Import Instance", 14)
                        .on_press(Message::CreateInstance(CreateInstanceMessage::Import)),
                ]
                .spacing(5),
                widget::text("Loading version list...").size(20),
            ]
            .padding(10)
            .spacing(10)
            .into(),
            MenuCreateInstance::Choosing {
                instance_name,
                selected_version,
                download_assets,
                search_box,
                is_server,
                show_category_dropdown,
                ..
            } => {
                let pb = iced::Padding::new(7.0).left(10.0).right(10.0);
                let opened_search = search_box.is_some();
                let opened_controls = *show_category_dropdown;
                let header = column![row![
                    back_button()
                        .style(|t: &LauncherTheme, s| t.style_button(s, StyleButton::RoundDark))
                        .on_press(Message::LaunchScreenOpen {
                            message: None,
                            clear_selection: false,
                            is_server: Some(*is_server),
                        }),
                    widget::button(icon_manager::three_lines())
                        .padding(pb)
                        .style(move |t: &LauncherTheme, s| t.style_button(
                            s,
                            if opened_search {
                                StyleButton::Round
                            } else {
                                StyleButton::RoundDark
                            }
                        )),
                    widget::button(icon_manager::three_lines())
                        .padding(pb)
                        .style(move |t: &LauncherTheme, s| t.style_button(
                            s,
                            if opened_controls {
                                StyleButton::Round
                            } else {
                                StyleButton::RoundDark
                            }
                        ))
                ]
                .spacing(5)]
                .spacing(5)
                .push_maybe(
                    search_box
                        .as_deref()
                        .map(|t| widget::text_input("Search...", t)),
                );

                let sidebar = Self::get_sidebar_contents(
                    versions,
                    selected_version,
                    *is_server,
                    header.into(),
                );

                row![
                    sidebar,
                    Self::get_main_page(
                        selected_version,
                        instance_name,
                        *download_assets,
                        existing_instances
                    )
                ]
                .width(Length::Fill)
                .into()
            }
            MenuCreateInstance::DownloadingInstance(progress) => column![
                widget::text("Downloading Instance..").size(20),
                progress.view()
            ]
            .padding(10)
            .spacing(5)
            .into(),
            MenuCreateInstance::ImportingInstance(progress) => column![
                widget::text("Importing Instance..").size(20),
                progress.view()
            ]
            .padding(10)
            .spacing(5)
            .into(),
        }
    }

    fn get_main_page(
        selected_version: &ListEntry,
        instance_name: &String,
        download_assets: bool,
        existing_instances: Option<&[String]>,
    ) -> widget::Column<'static, Message, LauncherTheme> {
        let already_exists = existing_instances.is_some_and(|n| n.contains(instance_name));

        column![
            button_with_icon(icon_manager::folder_with_size(14), "Import Instance", 14)
                .on_press(Message::CreateInstance(CreateInstanceMessage::Import)),
            row![
                widget::text("Name:").width(100).size(18),
                {
                    let placeholder = selected_version.name.as_str();
                    widget::text_input(placeholder, instance_name)
                        .on_input(|n| Message::CreateInstance(CreateInstanceMessage::NameInput(n)))
                }
            ].align_y(Alignment::Center),

            tooltip(
                row![
                    widget::Space::with_width(5),
                    widget::checkbox("Download assets?", download_assets).text_size(14).size(14).on_toggle(|t| Message::CreateInstance(CreateInstanceMessage::ChangeAssetToggle(t)))
                ],
                widget::text("If disabled, creating instance will be MUCH faster, but no sound or music will play in-game").size(12),
                Position::Bottom
            ),
            get_create_button(already_exists),
            widget::horizontal_rule(1),
            column![
                widget::text("- To install Fabric/Forge/OptiFine/etc and mods, click on Mods after installing the instance").size(12),
                row!(
                    widget::text("- To sideload your own custom JARs, create an instance with a similar version, then go to").size(12),
                    widget::text(" \"Edit->Custom Jar File\"").size(12)
                ).wrap(),
            ].spacing(5)
        ].push_maybe({
            let real_platform = if cfg!(target_arch = "x86") { "x86_64" } else { "aarch64" };
            (cfg!(target_os = "linux") && (cfg!(target_arch = "x86") || cfg!(target_arch = "arm")))
            .then_some(column![
                // WARN: Linux i686 and arm32
                widget::text("Warning: On your platform (Linux 32 bit) only Minecraft 1.16.5 and below are supported.").size(20),
                widget::text!("If your computer isn't outdated, you might have wanted to download QuantumLauncher 64 bit ({real_platform})"),
            ])
        }).spacing(10).padding(10)
    }

    fn get_sidebar_contents<'a>(
        versions: Option<&'a [ListEntry]>,
        selected_version: &'a ListEntry,
        is_server: bool,
        header: Element<'static>,
    ) -> widget::Container<'a, Message, LauncherTheme> {
        sidebar(
            Some(header),
            versions
                .into_iter()
                .flatten()
                .filter(|n| n.supports_server || !is_server)
                .map(|n| {
                    let label = widget::text(&n.name)
                        .size(if n.kind == ListEntryKind::Snapshot {
                            14
                        } else {
                            16
                        })
                        .style(|t: &LauncherTheme| {
                            t.style_text(if n.kind == ListEntryKind::Snapshot {
                                Color::SecondLight
                            } else {
                                Color::Light
                            })
                        });

                    sidebar_button(
                        n,
                        selected_version,
                        label,
                        Message::CreateInstance(CreateInstanceMessage::VersionSelected(n.clone())),
                    )
                }),
        )
    }
}

fn get_create_button(already_exists: bool) -> Element<'static> {
    let create_button = button_with_icon(icon_manager::create(), "Create Instance", 16)
        .on_press_maybe(
            (!already_exists).then_some(Message::CreateInstance(CreateInstanceMessage::Start)),
        );

    if already_exists {
        tooltip(
            create_button,
            "An instance with that name already exists!",
            Position::FollowCursor,
        )
        .into()
    } else {
        create_button.into()
    }
}
