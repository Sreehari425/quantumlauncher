import os
import re
import subprocess
import sys
import time

from . import procs

_ANSI_ESCAPE = re.compile(r'\x1b\[[0-9;]*[mK]')
def _remove_ansi_colors(text: str) -> str:
    return _ANSI_ESCAPE.sub('', text)

def _launch(instance: str) -> str | None:
    process = subprocess.Popen(
        [procs.QL_BIN, "launch", instance, "test`"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    if not process.stdout:
        print("Error: Launcher has no stdout!")
        sys.exit(1)

    pid = None
    pattern = r'\[info\] Launched! PID: (\d+)'

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

    return pid

def _is_process_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False  # No such process
    except PermissionError:
        return True   # Process exists, but not owned by you
    else:
        return True   # Process is alive

def _close_window(result: bytes, pid: int):
    window_ids = result.decode().strip().splitlines()
    print(f"✅ Window found: {window_ids} for pid {pid}, killing")
    procs.kill_process(pid)


def _wait_for_window(pid: int, timeout: int, name: str) -> bool:
    start_time = time.time()
    check_interval = max(1, timeout // 30)
    print(f"\n\nChecking {name} ({pid}) with interval {check_interval} seconds")

    while time.time() - start_time < timeout:
        if not _is_process_alive(pid):
            print("Error: Game crashed!")
            return False

        try:
            result = subprocess.check_output(["xdotool", "search", "--pid", str(pid)])
            _close_window(result, pid)
            return True
        except subprocess.CalledProcessError:
            try:
                result = subprocess.check_output(["xdotool", "search", "--classname", "Minecraft*", "windowclose"])
                print("    (found some \"Minecraft\" window, not sure)")
                _close_window(result, pid)
                return True
            except subprocess.CalledProcessError:
                pass  # No window yet
        time.sleep(check_interval)
        print("    ...checking")
    else:
        print("Error: Timeout waiting for window!")
        return False

def test(name: str, timeout: int) -> bool:
    pid = _launch(name)
    if pid:
        if not _wait_for_window(int(pid), timeout, name):
            print("Test failed (window)!")
            return False
    else:
        print("Test failed (launch)!")
        return False
    return True
