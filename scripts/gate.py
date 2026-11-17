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

# Ensure UTF-8 output on all platforms (avoids Windows cp1252 charmap errors)
if hasattr(sys.stdout, "reconfigure"):
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

FORBIDDEN_COMPILER_TOKENS = [
    "syn::",
    "use syn",
    "quote::",
    "quote!",
    "use quote",
    "darling::",
    "use darling",
    "proc_macro::",
    "use proc_macro",
    "proc_macro2::",
    "use proc_macro2",
]

SOFT_LINE_LIMIT = 200

# The `unstable` feature is built in its own target dir. Cargo unifies features
# across a build, so sharing one target dir would let the unstable build leak
# into the stable compile-fail fixtures that prove the gate.
UNSTABLE_ENV = {"CARGO_TARGET_DIR": str(Path("target") / "unstable")}

# The publish dry run builds the packaged crates in a target dir of its own, so
# that their rlibs never sit beside the ones the compile-fail fixtures link to.
PACKAGE_ENV = {"CARGO_TARGET_DIR": str(Path("target") / "package-check")}


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


def check_architectural_invariants() -> bool:
    print("\n>>> Checking Architectural Invariants & File Lengths...")
    root = get_repo_root()
    violations: list[str] = []

    # 1. Compiler leak check in domain and use_cases
    isolated_dirs = [
        root / "snapfit-macros" / "src" / "domain",
        root / "snapfit-macros" / "src" / "use_cases",
    ]

    for d in isolated_dirs:
        if not d.exists():
            continue
        for rs_file in d.rglob("*.rs"):
            try:
                content = rs_file.read_text(encoding="utf-8")
            except Exception as e:
                violations.append(f"  [ERROR] Cannot read {rs_file}: {e}")
                continue

            for idx, line in enumerate(content.splitlines(), start=1):
                trimmed = line.strip()
                if trimmed.startswith("//"):
                    continue
                for token in FORBIDDEN_COMPILER_TOKENS:
                    if token in line:
                        rel = rs_file.relative_to(root)
                        violations.append(
                            f"  [LEAK] {rel}:{idx}: forbidden compiler token '{token}'"
                        )

    # 2. File line length guideline check and modern module layout (no mod.rs)
    for crate_name in ["snapfit", "snapfit-macros", "xtask", "tests", "book"]:
        for subdir in ["src", "examples"]:
            crate_src = root / crate_name / subdir
            if not crate_src.exists():
                continue
            for rs_file in crate_src.rglob("*.rs"):
                rel = rs_file.relative_to(root)
                if rs_file.name == "mod.rs":
                    violations.append(
                        f"  [MOD.RS] {rel}: forbidden 'mod.rs' detected. Use modern Rust module layout (<name>.rs and submodules in <name>/)."
                    )
                try:
                    line_count = len(rs_file.read_text(encoding="utf-8").splitlines())
                except Exception:
                    continue
                if line_count > SOFT_LINE_LIMIT:
                    violations.append(
                        f"  [SIZE] {rel}: {line_count} lines exceeds ~{SOFT_LINE_LIMIT} line soft limit"
                    )

    if violations:
        print("❌ Architectural Boundary Check Failed:")
        for v in violations:
            print(v)
        return False

    print("✅ Architectural Invariants: 0 compiler leaks, no mod.rs, all file lengths compliant.")
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
            command(clippy + ["--all-features", "--", "-D", "warnings"], UNSTABLE_ENV),
        ],
        "arch": [check_architectural_invariants],
        "test": [
            command(["cargo", "test", "--workspace", "--all-targets", "--locked"]),
            command(
                ["cargo", "test", "-p", "snapfit_bdd", "--features", "unstable", "--locked"],
                UNSTABLE_ENV,
            ),
        ],
        # docs.rs documents snapfit with the `unstable` feature.
        "doc": [
            command(doc, doc_env),
            command(doc + ["--all-features"], {**doc_env, **UNSTABLE_ENV}),
        ],
        "deny": [command(["cargo", "deny", "check"])],
        "package": [
            command(
                ["cargo", "publish", "--workspace", "--dry-run", "--locked", "--allow-dirty"],
                PACKAGE_ENV,
            )
        ],
    }


def book_steps() -> list[Step]:
    """Every example, recorded output and compiler message of The Snapfit Book."""
    return [command(["cargo", "test", "-p", "snapfit_book", "--all-targets", "--locked"])]


ALIASES = {"fmt-check": "fmt", "arch-check": "arch", "book-check": "book"}

# `cargo xtask ci` runs every mode, in the order checks() lists them. `book` is
# not one of them: the `test` mode already runs the book's tests.
CI_MODES = list(checks())

VALID_MODES = {"all", "ci", "book", *CI_MODES, *ALIASES}


def main() -> int:
    args = sys.argv[1:]
    mode = args[0] if args else "all"

    if mode not in VALID_MODES:
        print(
            f"❌ Unknown mode '{mode}'. Available modes: {', '.join(sorted(VALID_MODES))}"
        )
        return 1

    modes = CI_MODES if mode in ("all", "ci") else [ALIASES.get(mode, mode)]
    steps = checks() | {"book": book_steps()}
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
