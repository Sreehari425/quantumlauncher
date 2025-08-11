import argparse
import os
import shutil
import sys
import time

from tests.launch import IS_XWAYLAND, IS_X11
from . import launch, procs, create

IS_COMPATIBLE: bool = (IS_X11 or IS_XWAYLAND
                       or sys.platform.startswith("win"))

if not IS_COMPATIBLE:
    print("""Unsupported platform!
This test suite only currently supports:
- Windows
- x11 (Linux, etc)

For more info see tests/README.md
""")
    sys.exit(1)


def rmdir(directory_path: str) -> None:
    if os.path.exists(directory_path) and os.path.isdir(directory_path):
        shutil.rmtree(directory_path)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--timeout", type=int, default=60, help="Timeout in seconds (default: 60)")
    parser.add_argument("--existing", action="store_true",
                        help="Use existing instances from previous test, instead of redownloading")
    # parser.add_argument("--instance", required=True, help="Instance ID to test")
    args = parser.parse_args()
    print("(building launcher...)")
    procs.run(["cargo", "build"])

    procs.prepare_ql_bin()

    # Customize as needed
    versions = create.VERSIONS_L2 + create.VERSIONS_L3

    if not args.existing:
        rmdir("tests/QuantumLauncher/instances")
        rmdir("tests/QuantumLauncher/assets")
        create.create(versions)

    for version in versions:
        if not launch.test(version, args.timeout):
            sys.exit(1)
        time.sleep(2)


if __name__ == "__main__":
    main()
