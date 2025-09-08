## PR Title: Add Comprehensive Terminal User Interface (TUI) to QuantumLauncher

### TLDR

This PR introduces a full-featured Terminal User Interface (TUI) for QuantumLauncher, enabling users to manage Minecraft instances, accounts, and settings entirely from the command line using a modern, keyboard-driven interface built with Ratatui.

### 🎯 Key Features Added

#### **Core TUI Infrastructure**

- **Ratatui-based UI Framework**: Clean, responsive terminal interface with proper mouse and keyboard support
- **Async Event System**: Real-time handling of authentication, launches, and log streaming
- **Periodic Refresh**: Automatic UI updates to prevent stdout/stderr interference during gameplay
- **State Management**: Robust app state with tab navigation and popup overlays

#### **Tab-Based Navigation**

1. **Instances Tab**

   - List all Minecraft instances with version, loader, and running status
   - Navigate with ↑/↓ keys, launch with Enter
   - Visual indicators for running instances
   - Empty state guidance for new users

2. **Create Tab**

   - Interactive instance creation with name input (Ctrl+N to edit)
   - Version selection from available Minecraft versions
   - Download assets toggle
   - Real-time creation progress with async feedback

3. **Settings Tab**

   - **General Settings**: Core launcher configuration
   - **Java Settings**: JVM path, memory allocation, arguments
   - **UI/Theme**: Terminal theme customization
   - **Launch Options**: Pre/post-launch commands
   - **About & Licenses**: Built-in license viewer and app information

4. **Accounts Tab**

   - List all Minecraft accounts (Microsoft, Offline)
   - Add new accounts with guided flow
   - Login/logout functionality with OTP support
   - Default account selection
   - Account type indicators and status

5. **Logs Tab**
   - Real-time game log streaming
   - Auto-follow with manual scroll control
   - Word-wrapped text for readability
   - Persistent log buffer (2000 lines)

#### **Instance-Specific Management**

- **Overview Sub-tab**: Instance details, launch button, quick actions
- **Mods Sub-tab**: Mod management interface (integrated with existing mod system)
- **Settings Sub-tab**: Per-instance configuration (memory, JVM args, etc.)
- **Logs Sub-tab**: Instance-specific log viewing

#### **Interactive Features**

- **Popups & Modals**:
  - Loading indicators during async operations
  - Confirmation dialogs for destructive actions
  - Help popup with keyboard shortcuts
  - Rename instance popup with live editing
  - Memory allocation editor
- **Keyboard Shortcuts**:
  - Tab navigation (Tab/Shift+Tab)
  - Instance selection (↑/↓/Enter)
  - Quick actions (F5 refresh, Ctrl+N edit name, etc.)
  - Popup controls (Esc cancel, Enter confirm)

#### **Authentication & Launch Integration**

- **ELY Account Login**: Full OAuth flow with OTP support
- **Offline Accounts**: Local account creation
- **Launch Monitoring**: Real-time status updates and error handling

#### **Technical Enhancements**

- **Stdio Management**: Automatic logging disable/enable to prevent TUI corruption
- **Error Handling**: Graceful error display and recovery
- **Performance**: Efficient rendering with minimal redraws
- **Accessibility**: Keyboard-only navigation, clear visual hierarchy

### 🔧 Implementation Details

- **6593 lines added, 775 lines modified** across 79 files
- **New Dependencies**: Ratatui, Crossterm for terminal UI
- **Architecture**: Modular design with separate app state, UI rendering, and event handling
- **Integration**: Seamlessly integrated with existing QuantumLauncher backend
- **Testing**: Comprehensive async testing support maintained

### 🎨 User Experience

- **Intuitive Navigation**: Tab-based workflow familiar to GUI users
- **Real-time Feedback**: Immediate visual responses to all actions
- **Robust Error Handling**: Clear error messages and recovery options
- **Performance**: Smooth operation even during intensive operations like mod downloads
- **Accessibility**: Full keyboard control, no mouse required

This TUI brings the full power of QuantumLauncher to the terminal, making it perfect for headless servers, SSH sessions, and users who prefer keyboard-driven workflows while maintaining all the features of the GUI version. 🚀
