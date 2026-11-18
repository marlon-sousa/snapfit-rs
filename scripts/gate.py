#!/usr/bin/env python3
"""QA Quality Gate & Architectural Invariant Checker for snapfit-rs.

Cross-platform script running formatting, clippy, boundary verification,
unit tests, documentation, dependency (cargo-deny) and packaging checks.
"""

import os
import subprocess
import sys
from collections.abc import Callable
from pathlib import Path

import architecture
import crate_readme

# Ensure UTF-8 output on all platforms (avoids Windows cp1252 charmap errors)
if hasattr(sys.stdout, "reconfigure"):
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass


def get_repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def run_command(cmd: list[str], env: dict[str, str] | None = None) -> bool:
    print(f"\n>>> Running: {' '.join(cmd)}")
    full_env = os.environ.copy()
    if env:
        full_env.update(env)
    result = subprocess.run(cmd, env=full_env, cwd=get_repo_root())
    if result.returncode != 0:
        print(f"❌ Command failed with code {result.returncode}: {' '.join(cmd)}")
        return False
    return True


Step = Callable[[], bool]


def command(cmd: list[str], env: dict[str, str] | None = None) -> Step:
    """A step running `cmd` with `env` added to the environment."""
    return lambda: run_command(cmd, env=env)


def checks() -> dict[str, list[Step]]:
    """The steps of each mode, in the order they run."""
    clippy = ["cargo", "clippy", "--workspace", "--all-targets", "--locked"]
    doc = ["cargo", "doc", "--workspace", "--no-deps", "--locked"]
    doc_env = {"RUSTDOCFLAGS": "-D warnings"}
    return {
        "fmt": [command(["cargo", "fmt", "--all", "--", "--check"])],
        "clippy": [
            command(clippy + ["--", "-D", "warnings"]),
            command(clippy + ["--all-features", "--", "-D", "warnings"]),
        ],
        "arch": [lambda: architecture.check(get_repo_root())],
        "test": [
            command(["cargo", "test", "--workspace", "--all-targets", "--locked"]),
            # `--all-targets` leaves doctests out, such as the README's example.
            command(["cargo", "test", "--workspace", "--doc", "--locked"]),
            command([sys.executable, "-m", "unittest", "discover", "-s", "scripts", "-q"]),
            command(["cargo", "test", "-p", "snapfit_bdd", "--features", "unstable", "--locked"]),
        ],
        # The default build is what docs.rs documents; the unstable build must document cleanly too.
        "doc": [
            command(doc, doc_env),
            command(doc + ["--all-features"], doc_env),
            lambda: crate_readme.check(get_repo_root()),
        ],
        "deny": [command(["cargo", "deny", "check"])],
        "package": [
            command(["cargo", "publish", "--workspace", "--dry-run", "--locked", "--allow-dirty"])
        ],
    }


def book_steps() -> list[Step]:
    """Every example, recorded output and compiler message of The Snapfit Book."""
    return [command(["cargo", "test", "-p", "snapfit_book", "--all-targets", "--locked"])]


ALIASES = {"fmt-check": "fmt", "arch-check": "arch", "book-check": "book"}

# `cargo xtask ci` runs every mode, in the order checks() lists them. `book` is
# not one of them: the `test` mode already runs the book's tests. Nor is
# `crate-readme`, which writes the file the `doc` mode checks.
CI_MODES = list(checks())


def other_steps() -> dict[str, list[Step]]:
    """The steps of each mode `cargo xtask ci` leaves out."""
    return {"book": book_steps(), "crate-readme": [lambda: crate_readme.write(get_repo_root())]}


VALID_MODES = {"all", "ci", *CI_MODES, *other_steps(), *ALIASES}


def main() -> int:
    args = sys.argv[1:]
    mode = args[0] if args else "all"

    if mode not in VALID_MODES:
        print(
            f"❌ Unknown mode '{mode}'. Available modes: {', '.join(sorted(VALID_MODES))}"
        )
        return 1

    modes = CI_MODES if mode in ("all", "ci") else [ALIASES.get(mode, mode)]
    steps = checks() | other_steps()
    success = True
    for name in modes:
        for step in steps[name]:
            success = step() and success

    if success:
        print("\n🎉 Quality Gate PASSED: All checks clean with zero errors!")
        return 0
    else:
        print("\n❌ Quality Gate FAILED: See details above.")
        return 1


if __name__ == "__main__":
    sys.exit(main())
