use std::fmt::Display;

use filthy_rich::types::{Activity, ActivityType};
use serde::{Deserialize, Serialize};

/// Returns a fully-built [`filthy_rich::types::Activity`] object given the details and state.
pub fn bake_activity(
    name: Option<String>,
    sdt: PresenceStatusDisplayType,
    competing: bool,
    details: Option<String>,
    details_url: Option<String>,
    state: Option<String>,
    state_url: Option<String>,
) -> Activity {
    let mut activity = Activity::new()
        .activity_type(if competing {
            ActivityType::Competing
        } else {
            ActivityType::Playing
        })
        .status_display_type(match sdt {
            PresenceStatusDisplayType::Name => filthy_rich::types::StatusDisplayType::Name,
            PresenceStatusDisplayType::Details => filthy_rich::types::StatusDisplayType::Details,
            PresenceStatusDisplayType::State => filthy_rich::types::StatusDisplayType::State,
        });

    if let Some(n) = name {
        activity = activity.name(n)
    }
    if let Some(d) = details {
        activity = activity.details(d);

        if let Some(d_u) = details_url {
            activity = activity.details_url(d_u);
        }
    }
    if let Some(s) = state {
        activity = activity.state(s);

        if let Some(s_u) = state_url {
            activity = activity.state_url(s_u);
        }
    }

    activity.build()
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum PresenceStatusDisplayType {
    #[default]
    Name,
    Details,
    State,
}

impl PresenceStatusDisplayType {
    pub const ALL: &'static [Self] = &[
        PresenceStatusDisplayType::Name,
        PresenceStatusDisplayType::Details,
        PresenceStatusDisplayType::State,
    ];
}

impl Display for PresenceStatusDisplayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            PresenceStatusDisplayType::Name => "Name",
            PresenceStatusDisplayType::Details => "Details",
            PresenceStatusDisplayType::State => "State",
        })
    }
}
