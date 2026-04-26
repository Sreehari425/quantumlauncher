use filthy_rich::types::Activity;

/// Returns a fully-built [`filthy_rich::types::Activity`] object given the details and state.
pub fn bake_activity(
    details: Option<String>,
    details_url: Option<String>,
    state: Option<String>,
    state_url: Option<String>,
) -> Activity {
    let mut activity = Activity::new();

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
