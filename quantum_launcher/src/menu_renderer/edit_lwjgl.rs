use iced::{widget, Alignment, Length};

use crate::{
    icon_manager,
    state::{EditLwjglMessage, MenuEditLwjgl, Message},
    stylesheet::{color::Color, styles::LauncherTheme},
};

use super::{back_button, button_with_icon, Element};

impl MenuEditLwjgl {
    pub fn view(&'_ self) -> Element<'_> {
        match self {
            MenuEditLwjgl::Loading { .. } => {
                widget::column![
                    back_button().on_press(Message::EditLwjgl(EditLwjglMessage::Back)),
                    widget::text("Loading LWJGL versions...").size(20),
                    widget::text("Fetching version list from Maven repository"),
                ]
                .spacing(10)
                .padding(20)
                .into()
            }
            MenuEditLwjgl::Loaded {
                versions,
                selected,
                initial_version,
                is_applying,
            } => {
                let title = widget::text("LWJGL Version Override").size(24);

                let warning = widget::container(
                    widget::column![
                        widget::text("⚠️ Warning").size(16),
                        widget::text(
                            "Changing the LWJGL version may cause the game to crash or not start. \
                            Only change this if you know what you're doing."
                        ),
                        widget::text(
                            "Setting to 'Default' will use the version bundled with the game."
                        ),
                    ]
                    .spacing(5),
                )
                .padding(10)
                .style(|t: &LauncherTheme| t.style_container_sharp_box(2.0, Color::Mid));

                // Create the dropdown options: "Default" + all versions
                let display_selected = selected
                    .as_ref()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Default (game version)".to_string());

                let picker = widget::column![
                    widget::text("Select LWJGL Version:").size(16),
                    widget::pick_list(
                        std::iter::once("Default (game version)".to_string())
                            .chain(versions.iter().cloned())
                            .collect::<Vec<_>>(),
                        Some(display_selected),
                        |s| {
                            let version = if s == "Default (game version)" {
                                None
                            } else {
                                Some(s)
                            };
                            Message::EditLwjgl(EditLwjglMessage::VersionSelected(version))
                        },
                    )
                    .width(Length::Fixed(300.0)),
                ]
                .spacing(5);

                // Show current setting
                let current_info = widget::text(format!(
                    "Current setting: {}",
                    initial_version
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("Default (game version)")
                ))
                .size(14);

                // Show if changed
                let has_changed = selected != initial_version;
                let changed_indicator: Element = if has_changed {
                    widget::text("* Changes pending")
                        .size(14)
                        .style(|t: &LauncherTheme| t.style_text(Color::Mid))
                        .into()
                } else {
                    widget::Space::new(0, 0).into()
                };

                // Apply button
                let apply_button = if *is_applying {
                    button_with_icon(icon_manager::tick(), "Applying...", 16)
                } else if has_changed {
                    button_with_icon(icon_manager::tick(), "Apply", 16)
                        .on_press(Message::EditLwjgl(EditLwjglMessage::Apply))
                } else {
                    button_with_icon(icon_manager::tick(), "Apply", 16)
                };

                widget::column![
                    back_button().on_press(Message::EditLwjgl(EditLwjglMessage::Back)),
                    title,
                    warning,
                    widget::Space::new(0, 10),
                    picker,
                    current_info,
                    changed_indicator,
                    widget::Space::new(0, 10),
                    apply_button,
                ]
                .spacing(10)
                .padding(20)
                .align_x(Alignment::Start)
                .into()
            }
        }
    }
}
