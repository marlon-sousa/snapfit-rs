#!/usr/bin/env python3
"""QA Quality Gate & Architectural Invariant Checker for snapfit-rs.

Cross-platform script running formatting, clippy, boundary verification,
unit tests, and documentation checks.
"""

import os
import subprocess
import sys
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

VALID_MODES = {
    "all",
    "ci",
    "fmt",
    "fmt-check",
    "clippy",
    "arch",
    "arch-check",
    "test",
    "doc",
}


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
    for crate_name in ["snapfit", "snapfit-macros", "xtask", "tests"]:
        crate_src = root / crate_name / "src"
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


def main() -> int:
    args = sys.argv[1:]
    mode = args[0] if args else "all"

    if mode not in VALID_MODES:
        print(
            f"❌ Unknown mode '{mode}'. Available modes: {', '.join(sorted(VALID_MODES))}"
        )
        return 1

    run_all = mode in ("all", "ci")
    success = True

    if run_all or mode in ("fmt", "fmt-check"):
        success = run_command(["cargo", "fmt", "--all", "--", "--check"]) and success

    if run_all or mode == "clippy":
        success = (
            run_command(
                [
                    "cargo",
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--",
                    "-D",
                    "warnings",
                ]
            )
            and success
        )
        success = (
            run_command(
                [
                    "cargo",
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--all-features",
                    "--",
                    "-D",
                    "warnings",
                ],
                env=UNSTABLE_ENV,
            )
            and success
        )

    if run_all or mode in ("arch", "arch-check"):
        success = check_architectural_invariants() and success

    if run_all or mode == "test":
        success = (
            run_command(["cargo", "test", "--workspace", "--all-targets"]) and success
        )
        success = (
            run_command(
                ["cargo", "test", "-p", "snapfit_bdd", "--features", "unstable"],
                env=UNSTABLE_ENV,
            )
            and success
        )

    if run_all or mode == "doc":
        success = (
            run_command(
                ["cargo", "doc", "--workspace", "--no-deps"],
                env={"RUSTDOCFLAGS": "-D warnings"},
            )
            and success
        )

    if success:
        print("\n🎉 Quality Gate PASSED: All checks clean with zero errors!")
        return 0
    else:
        print("\n❌ Quality Gate FAILED: See details above.")
        return 1


if __name__ == "__main__":
    sys.exit(main())
