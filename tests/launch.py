import os
import re
import subprocess
import sys
import time
from typing import List, Tuple

from . import procs

_ANSI_ESCAPE: re.Pattern[str] = re.compile(r'\x1b\[[0-9;]*[mK]')

IS_XWAYLAND = os.environ.get("WAYLAND_DISPLAY") is not None and os.environ.get("DISPLAY") is not None
IS_X11 = os.getenv("XDG_SESSION_TYPE") == "x11"


def _remove_ansi_colors(text):
    return _ANSI_ESCAPE.sub('', text)


def _launch(instance: str) -> int | None:
    process = subprocess.Popen(
        [procs.QL_BIN, "launch", instance, "test`"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    if not process.stdout:
        print("Error: Launcher has no stdout!")
        sys.exit(1)

    pid: str | None = None
    pattern = re.compile(r'\[info\] Launched! PID: (\d+)')

    for line in process.stdout:
        clean_line = _remove_ansi_colors(line)
        if "No ID found!" in clean_line:
            print("Error: Game crashed instantly!")
            process.kill()
            return None
        match = re.search(pattern, clean_line)
        if match:
            pid = match.group(1)
            break

    if not pid:
        print("Error: No PID found!")
        process.kill()
        return None

    return int(pid)


def _is_process_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False  # No such process
    except PermissionError:
        return True  # Process exists, but not owned by you
    else:
        return True  # Process is alive


# ---------------------------
# Windows Implementation
# ---------------------------
if sys.platform.startswith("win"):
    import ctypes
    from ctypes import wintypes

    user32 = ctypes.WinDLL("user32", use_last_error=True)
    WM_CLOSE = 0x0010

    EnumWindows = user32.EnumWindows
    EnumWindowsProc = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
    GetWindowThreadProcessId = user32.GetWindowThreadProcessId
    IsWindowVisible = user32.IsWindowVisible
    GetWindowTextLengthW = user32.GetWindowTextLengthW
    GetWindowTextW = user32.GetWindowTextW
    SendMessageW = user32.SendMessageW
    PostMessageW = user32.PostMessageW

    g_found: List[Tuple[int, str]] = []
    g_pid: int = 0


    def _get_windows_for_pid(pid: int) -> List[Tuple[int, str]]:
        """Return list of (hwnd, title) for windows belonging to pid."""
        found: List[Tuple[int, str]] = []

        @EnumWindowsProc
        def _enum(hwnd, lParam):
            # Check visibility (optional; keeps parity with xdotool which finds top-level windows)
            try:
                if not IsWindowVisible(hwnd):
                    return True  # continue
            except Exception:
                # If any call fails, still continue enumeration
                pass

            lpdwProcessId = wintypes.DWORD()
            GetWindowThreadProcessId(hwnd, ctypes.byref(lpdwProcessId))
            window_pid = lpdwProcessId.value
            if window_pid == pid:
                length = GetWindowTextLengthW(hwnd)
                buffer = ctypes.create_unicode_buffer(length + 1)
                GetWindowTextW(hwnd, buffer, length + 1)
                title = buffer.value
                found.append((hwnd, title))
            return True

        # Start enumeration
        if not EnumWindows(_enum, 0):
            # EnumWindows returns 0 on failure; raise for debugging, but return whatever found
            err = ctypes.get_last_error()
            # Not fatal for runtime; just print for visibility
            print(f"Warning: EnumWindows failed with error {err}")
        return found


    def _close_window_windows(pid: int) -> None:
        wins = _get_windows_for_pid(pid)
        if not wins:
            print(f"No windows found for pid {pid}")
            return
        titles = [f"{hwnd}:'{title}'" for hwnd, title in wins]
        print(f"✅ Window found: {titles} for pid {pid}, sending WM_CLOSE")
        # Send WM_CLOSE to each handle; prefer SendMessage (synchronous) then fallback to PostMessage
        for hwnd, title in wins:
            try:
                # Try SendMessageW first
                SendMessageW(hwnd, WM_CLOSE, 0, 0)
            except Exception:
                try:
                    PostMessageW(hwnd, WM_CLOSE, 0, 0)
                except Exception:
                    # If both fail, continue; we'll still kill process below
                    print(f"    Could not send WM_CLOSE to hwnd {hwnd} ('{title}')")
        # Keep legacy behavior: kill process after closing windows
        procs.kill_process(pid)


    def _backend_windows(pid: int):
        # Windows backend: enumerate windows via ctypes, send WM_CLOSE if found
        try:
            # get windows for pid
            wins = []
            try:
                # call windows helper
                wins = _get_windows_for_pid(pid)
            except Exception as e:
                print(f"    (warning) failed to enumerate windows: {e}")
                wins = []

            if wins:
                titles = [f"{hwnd}:'{title}'" for hwnd, title in wins]
                print(f"    (found windows) {titles}")
                _close_window_windows(pid)
                return True
            else:
                # Also attempt fallback: try to find windows with class/title like "Minecraft"
                # We can enumerate all windows and check title matches "Minecraft" if present.
                all_wins = _get_windows_for_pid(pid)  # should be same; kept for parity
                if all_wins:
                    _close_window_windows(pid)
                    return True
        except Exception as ex:
            print(f"    (warning) windows-check exception: {ex}")


def _close_window_unix(result: bytes, pid: int) -> None:
    window_ids: list[str] = result.decode().strip().splitlines()
    print(f"✅ Window found: {window_ids} for pid {pid}, killing")
    procs.kill_process(pid)


def _wait_for_window(pid: int, timeout: int, name: str) -> bool:
    start_time = time.time()
    check_interval = max(1, timeout // 30)
    print(f"\n\nChecking {name} ({pid}) with interval {check_interval} seconds")

    is_windows = sys.platform.startswith("win")

    while time.time() - start_time < timeout:
        if not _is_process_alive(pid):
            print("Error: Game crashed!")
            return False

        if is_windows:
            if _backend_windows(pid):
                return True
        elif IS_X11 or IS_XWAYLAND:
            try:
                result = subprocess.check_output(["xdotool", "search", "--pid", str(pid)])
                _close_window_unix(result, pid)
                return True
            except subprocess.CalledProcessError:
                try:
                    result = subprocess.check_output(["xdotool", "search", "--classname", "Minecraft*", "windowclose"])
                    print("    (found some \"Minecraft\" window, not sure)")
                    _close_window_unix(result, pid)
                    return True
                except subprocess.CalledProcessError:
                    pass  # No window yet
        else:
            print(
                """Unsupported platform!
- If you're on Wayland try installing xwayland
- If you're on macOS this test suite isn't supported there yet""")

        time.sleep(check_interval)
        print("    ...checking")

    print("Error: Timeout waiting for window!")
    return False


def test(name: str, timeout: int) -> bool:
    pid: int | None = _launch(name)
    if pid:
        if not _wait_for_window(pid, timeout, name):
            print("Test failed (window)!")
            return False
    else:
        print("Test failed (launch)!")
        return False
    return True
