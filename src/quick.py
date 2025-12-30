#!/usr/bin/env python3
import subprocess
import sys
from pathlib import Path

def main():
    # Get the script directory and workspace root
    script_dir = Path(__file__).parent.resolve()
    root_dir = script_dir.parent if script_dir.name == "scripts" else script_dir

    print("Expanding macros for eucalyptus-core...")

    # Run cargo expand
    result = subprocess.run(
        ["cargo", "expand", "--lib", "-p", "eucalyptus-core"],
        cwd=root_dir,
        capture_output=True,
        text=True
    )

    if result.returncode != 0:
        print(f"Error: cargo expand failed", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        sys.exit(1)

    # The expanded code is in stdout
    expanded_code = result.stdout

    # Write expanded code to temp file
    temp_dir = root_dir / "target" / "generated"
    temp_dir.mkdir(parents=True, exist_ok=True)
    temp_file = temp_dir / "expanded.rs"

    print(f"Writing expanded code to {temp_file}")
    temp_file.write_text(expanded_code, encoding="utf-8")

    # Run cbindgen
    print("Generating C bindings...")
    output_file = root_dir / "headers" / "dropbear.h"
    output_file.parent.mkdir(parents=True, exist_ok=True)

    cbindgen_config = root_dir / "cbindgen.toml"

    result = subprocess.run(
        [
            "cbindgen",
            "--config", str(cbindgen_config),
            "--output", str(output_file),
            str(temp_file)
        ],
        cwd=root_dir,
        capture_output=True,
        text=True
    )

    if result.returncode != 0:
        print(f"Error: cbindgen failed", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        sys.exit(1)

    print(f"✓ Generated {output_file}")

if __name__ == "__main__":
    main()