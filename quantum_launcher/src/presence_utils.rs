use filthy_rich::types::Activity;

/// Returns a fully-built [`filthy_rich::types::Activity`] object given the details and state.
pub fn bake_activity(details: Option<String>, state: Option<String>) -> Activity {
    let mut activity = Activity::new();

    if let Some(d) = details {
        activity = activity.details(d);
    }
    if let Some(s) = state {
        activity = activity.state(s);
    }

    activity.build()
}
