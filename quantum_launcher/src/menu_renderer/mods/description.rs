use frostmark::{MarkState, MarkWidget};
use iced::{
    Alignment, Length,
    widget::{self, column, row, text::Wrapping},
};
use ql_mod_manager::store::{SearchMod, StoreBackendType};

use crate::{
    icons,
    menu_renderer::{Element, FONT_DEFAULT, FONT_MONO, button_with_icon},
    state::{ImageState, InstallModsMessage, ManageModsMessage, MenuModDescription, Message},
    stylesheet::{color::Color, styles::LauncherTheme},
};

impl MenuModDescription {
    pub fn view<'a>(&'a self, images: &'a ImageState, tick_timer: usize) -> Element<'a> {
        let Some(details) = &self.details else {
            let dots = ".".repeat((tick_timer % 3) + 1);
            return column![widget::text!("Loading{dots}")].padding(10).into();
        };

        view_project_description(
            self.description.as_ref(),
            self.mod_id.get_backend(),
            ManageModsMessage::Open,
            details,
            images,
            tick_timer,
        )
    }
}

/// Renders the mod description page
pub fn view_project_description<'a, T: iced::advanced::text::IntoFragment<'a>>(
    description: Result<&'a Option<MarkState>, T>,
    backend: StoreBackendType,
    back_msg: impl Into<Message>,
    hit: &'a SearchMod,
    images: &'a ImageState,
    tick_timer: usize,
) -> Element<'a> {
    // Parses the Markdown description of the mod.
    let markdown_description: Element = match description {
        Ok(Some(desc)) => MarkWidget::new(desc)
            .on_clicking_link(Message::CoreOpenLink)
            .on_drawing_image(|img| images.view(Some(img.url), img.width, img.height))
            .on_updating_state(|n| InstallModsMessage::TickDesc(n).into())
            .font(FONT_DEFAULT)
            .font_mono(FONT_MONO)
            .into(),
        Ok(None) => {
            let dots = ".".repeat((tick_timer % 3) + 1);
            widget::text!("Loading...{dots}").into()
        }
        Err(err) => widget::container(
            column![
                widget::text("Failed to load description").size(16),
                widget::text(err).size(13)
            ]
            .spacing(5)
            .padding(10),
        )
        .into(),
    };

    let url = format!(
        "{}{}/{}",
        match backend {
            StoreBackendType::Modrinth => "https://modrinth.com/",
            StoreBackendType::Curseforge => "https://www.curseforge.com/minecraft/",
        },
        hit.project_type,
        hit.internal_name
    );

    let top_bar = widget::container(
        row![
            button_with_icon(icons::back_s(12), "Back", 13)
                .padding([5, 8])
                .on_press(back_msg.into()),
            widget::Space::with_width(0),
            images.view(hit.icon_url.as_deref(), Some(20.0), Some(20.0)),
            widget::text(&hit.title)
                .shaping(widget::text::Shaping::Advanced)
                .width(Length::Fill)
                .size(16),
            widget::tooltip(
                button_with_icon(icons::globe_s(12), "Open Mod Page", 13)
                    .padding([5, 8])
                    .on_press(Message::CoreOpenLink(url.clone())),
                widget::text(url),
                widget::tooltip::Position::Bottom
            )
            .style(|n| n.style_container_sharp_box(0.0, Color::ExtraDark)),
            widget::button(widget::text("Copy ID").size(13).wrapping(Wrapping::None))
                .padding([5, 8])
                .on_press(Message::CoreCopyText(hit.id.clone())),
        ]
        .align_y(Alignment::Center)
        .spacing(10),
    )
    .style(|n: &LauncherTheme| n.style_container_sharp_box(0.0, Color::ExtraDark))
    .padding([5, 10]);

    column![
        top_bar,
        widget::horizontal_rule(1).style(|t: &LauncherTheme| t.style_rule(Color::SecondDark, 1)),
        widget::scrollable(
            column![
                widget::container(
                    widget::text(&hit.description).shaping(widget::text::Shaping::Advanced)
                )
                .padding(10),
                markdown_description
            ]
            .padding(20)
            .spacing(20),
        )
        .style(LauncherTheme::style_scrollable_flat_extra_dark)
        .width(Length::Fill)
        .height(Length::Fill)
    ]
    .into()
}
