import os
import subprocess
import sys
import shutil
import signal
from concurrent.futures import ThreadPoolExecutor, as_completed


QL_BIN: str = "tests/qlbin.exe" if sys.platform == "Windows" else "tests/qlbin"
OLD_QL_BIN: str = (
    "target/debug/quantum_launcher.exe"
    if sys.platform == "Windows"
    else "target/debug/quantum_launcher"
)

def prepare_ql_bin():
    shutil.copy(OLD_QL_BIN, QL_BIN)
    with open('tests/qldir.txt', 'w') as f:
        f.write('')

def run(args: list[str]):
    try: subprocess.run(args)
    except subprocess.CalledProcessError as e:
        print(f"Error: Process failed with exit code {e.returncode}")
        sout = e.stdout.decode()
        if len(sout) > 0:
            print(f"Stdout:\n{sout}")
        sout = e.stderr.decode()
        if len(sout) > 0:
            print(f"Stderr:\n{sout}")
        sys.exit(1)

def run_parallel(commands: list[list[str]], max_workers: int | None = None):
    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        futures = {executor.submit(run, cmd): cmd for cmd in commands}

        try:
            for future in as_completed(futures):
                future.result()  # Will raise if the subprocess failed
        except Exception as e:
            print(f"Early exit: A subprocess failed. Cancelling remaining...")
            for f in futures:
                f.cancel()
            sys.exit(1)

def kill_process(pid: int):
    try:
        os.kill(pid, signal.SIGTERM)  # SIGTERM is a termination signal
        print(f"    Process {pid} has been terminated.")
    except ProcessLookupError:
        print(f"    Process {pid} not found.")
    except PermissionError:
        print(f"    Permission denied to kill process {pid}.")
    except Exception as e:
        print(f"Error occurred: {e}")
