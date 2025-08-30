// QuantumLauncher TUI - Authentication Event Handler

use crate::tui::{AuthEvent, app::App};

/// Handle authentication events from async operations
pub fn handle_auth_event(app: &mut App, event: AuthEvent) {
    match event {
        AuthEvent::LoginStarted => {
            app.status_message = "🔄 Authenticating...".to_string();
            app.is_loading = true;
        }
        AuthEvent::LoginSuccess { account_data } => {
            app.is_loading = false;
            app.login_error = None;
            
            // Save the authenticated account
            match app.save_authenticated_account(&account_data) {
                Ok(()) => {
                    app.status_message = format!("✅ Successfully logged in as {}", account_data.nice_username);
                    app.toggle_add_account_mode(); // Exit add account mode
                    app.load_accounts(); // Refresh account list
                }
                Err(err) => {
                    app.status_message = format!("❌ Login succeeded but failed to save: {}", err);
                    app.login_error = Some(format!("Failed to save account: {}", err));
                }
            }
        }
        AuthEvent::LoginNeedsOtp => {
            app.is_loading = false;
            app.needs_otp = true;
            app.add_account_field_focus = crate::tui::app::AddAccountFieldFocus::Otp;
            app.status_message = "🔢 Two-factor authentication required. Enter OTP code.".to_string();
        }
        AuthEvent::LoginError { error } => {
            app.is_loading = false;
            app.needs_otp = false;
            app.login_error = Some(error.clone());
            app.status_message = format!("❌ Login failed: {}", error);
        }
        AuthEvent::LaunchStarted(message) => {
            app.add_log(message);
            app.needs_forced_refresh = true; // Force refresh to overwrite any stdout spam
        }
        AuthEvent::LaunchSuccess(instance_name, _child_process) => {
            let success_msg = format!("✅ Successfully launched {}", instance_name);
            app.status_message = success_msg.clone();
            app.add_log(success_msg);
        }
        AuthEvent::LaunchError(instance_name, error) => {
            let error_msg = format!("❌ Failed to launch {}: {}", instance_name, error);
            app.status_message = error_msg.clone();
            app.add_log(error_msg);
        }
    }
}
