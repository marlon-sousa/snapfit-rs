"""The architecture check, which the gate's `arch` mode runs."""

from pathlib import Path

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


def check(root: Path) -> bool:
    """Whether the code keeps the architecture's invariants: no compiler type in `domain/` or `use_cases/`, no `mod.rs`, files under about 200 lines."""
    print("\n>>> Checking Architectural Invariants & File Lengths...")
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
