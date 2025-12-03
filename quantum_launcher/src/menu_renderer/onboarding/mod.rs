use std::sync::LazyLock;

use iced::widget::image::Handle;

mod changelog;
mod welcome;

pub use changelog::changelog;

pub static IMG_MANAGE_MODS: LazyLock<Handle> = LazyLock::new(|| {
    Handle::from_bytes(include_bytes!("../../../../assets/screenshots/mod_manage.png").as_slice())
});
pub static IMG_LOGO: LazyLock<Handle> = LazyLock::new(|| {
    Handle::from_bytes(include_bytes!("../../../../assets/icon/ql_logo.png").as_slice())
});
