# AGENTS.md: working in snapfit

The manual for anyone, human or agent, changing snapfit. [docs/WORKFLOW.md](docs/WORKFLOW.md) describes the stacked pull requests and the definition of done; this file says what every change follows and how it is reviewed.

## What to read

- [docs/SPEC.md](docs/SPEC.md): what the macros accept, generate and refuse, and what ships in 0.1.0. Code does what it says, no more and no less.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): how the code is organised and written, before writing any.
- [.claude/skills/](.claude/skills/): the procedures, such as having a change reviewed. Claude Code loads them on its own; another agent reads the one it needs.
- The issues the work belongs to, and its pull request. Once a decision is agreed there, it binds the work.

## Rules

1. **No code before its behaviour.** A change to what the macros accept, generate or refuse starts in docs/SPEC.md. If the code would need to do something SPEC.md does not say, stop and ask the maintainer.
2. **Every behaviour has a test a user would recognise.** A behaviour added to or changed in SPEC.md comes with a Gherkin scenario under `tests/features/`, in the same pull request as the code. A refusal comes with a compile-fail fixture under `tests/fixtures/compile_fail/` whose scenario checks its message and the line it points at.
3. **A fixed bug gets a regression test that fails without the fix**, and a Gherkin scenario, tagged `@regression`, when a user can meet the bug.
4. **Unstable is opt-in, planned is refused.** An API whose shape is not settled ships only behind the `unstable` Cargo feature, with a stable fixture proving the stable build refuses it. A planned feature is a compile error that names it. Anything removed for a release is listed in SPEC.md, §0, so that the work on its issue restores it.
5. **Diagnostics are for people.** Every error the macros report is a sentence that reads well spoken by a screen reader, points at the field it is about, and is reported together with every other problem found, as docs/ARCHITECTURE.md, "Errors", says.
6. **Generated names are promises.** Never change how a generated item is named or typed, such as `new()`, `init()` or `get_<field>()`, without the maintainer's agreement.
7. **Small pull requests.** One thing each, stacked as docs/WORKFLOW.md describes, with Conventional Commits messages carrying the changelog trailers docs/WORKFLOW.md asks for.
8. **Write for a screen reader.** Pull requests, comments and documents use headings, lists, prose and simple tables, never ASCII art. A diagram is Mermaid with `accTitle` and `accDescr`.
9. **Mark agent comments.** A comment an agent writes on GitHub says it is an agent's.
10. **Check a library before ruling it in or out.** Before saying that a crate can or cannot do something, read its source or its documentation, including its extension points, at two versions: the one we use, and the newest release, not yanked, whose `rust-version` is at most ours in `Cargo.toml`. A newer release may do what the one we use cannot. The pull request says what was checked at each version: the file and line, or the documentation page.
11. **One question at a time.** A question for the maintainer is asked in plain prose, one decision at a time, with the option recommended and why.
12. **Issues come first.** Every change starts from an agreed issue, except one whose commits, merges aside, are all `test` commits and that changes no `Cargo.toml` or `Cargo.lock`, and its pull request names it, as docs/WORKFLOW.md, section 10, says. Its label says what kind of decision the issue holds. A decision taken outside the issue, such as in a chat, is recorded on it at once.

## Reviewing a change

Every pull request is reviewed by an independent reviewer before it is opened, one given the change, not its author's reasoning. A later push to an open pull request is reviewed the same way while its CI runs. An agent's `.claude/skills/review-change` says how to start and resume the reviewer. The reviewer is given this section in full, the documents under "What to read", the issues the change belongs to, the commits to review and the draft pull request description. For each item, it lists every place in the diff where the item applies, not only the first, and checks each against the rule it names.

1. Every behaviour the change adds or changes, against docs/SPEC.md and its issues, and against rules 1 to 4 above: its scenario, its fixture, its regression test, its gate.
2. Every diagnostic the change adds or changes, against rule 5 above and docs/ARCHITECTURE.md, "Errors": its words, the field it points at, and that it is reported with the other problems found, not instead of them.
3. Every generated item the change adds or changes, against rule 6 above and docs/ARCHITECTURE.md, "Generated code": its name, its hygiene, and the expansion tests that pin it.
4. `snapfit-macros/src/lib.rs` and `snapfit/src/lib.rs`, against docs/ARCHITECTURE.md, "Folders and files", rule 1: no new bare `pub` other than the macros and the runtime crate's agreed API.
5. Every file the change adds or makes longer, against docs/ARCHITECTURE.md, "Folders and files", rule 6: what it holds, and whether it holds one thing well; if not, the private submodules it splits into.
6. Every new module: its role line, its folder and its name, against docs/ARCHITECTURE.md, "Module roles" and "Folders and files".
7. Every module of `domain/`, `use_cases/` and `adapters/`, against docs/ARCHITECTURE.md, "Module roles" and "The typed model": a decision an adapter takes, a call that goes up a layer, a compiler type named in `domain/` or `use_cases/`, a lookup by name that could fail, and a port or trait around a pure function are findings.
8. Against docs/ARCHITECTURE.md, "Writing the code": every `unwrap`, `expect`, `panic!`, indexing, `todo!` and `unimplemented!` outside tests; every hand-written trait impl and `new`, naming the derive that would produce it or why none does; every helper, naming what already provides it, if anything; every string or `bool` standing for a closed set, and every constraint stated in a comment that a type could hold; every newtype deriving `Deref`; every closure in a chain of adapters and every `for` loop; every line that changes nothing.
9. Every test the change adds or moves, against docs/ARCHITECTURE.md, "Folders and files", rule 5, and "Tests": where it lives, what its name states, and that a regression test fails without its fix.
10. Every pattern the change copies from elsewhere in the code or from a library, reviewed where it came from: what the original does that the copy leaves out, and every other place that copies it.
11. Every input the change accepts, asking whether a test covers the awkward forms a user can write: empty, repeated, several mistakes at once, and written through a macro. For each form no test covers, the reviewer writes a probe and reports what it does, or reports the gap.
12. Every comment and doc comment, against docs/ARCHITECTURE.md, "Comments".
13. Every crate the change adds, removes or uses in a new way, and every claim that a library can or cannot do something, against rule 10 above: it names the source it was checked against at each version, and what the newest release rule 10 names does that the one we use cannot.
14. Every statement of docs/SPEC.md, docs/ARCHITECTURE.md, docs/WORKFLOW.md and README.md the change makes untrue, which the change updates itself.
15. The commit messages and the description, against rules 7 and 12 above and docs/WORKFLOW.md: that the description names its issues unless every commit, merges aside, is a `test` and no `Cargo.toml` or `Cargo.lock` changes, and that the change does what they agreed, no more; for a layer of a stack, that its branch sits on the current tip of the layer below.
16. Every change a user meets in the book, such as a diagnostic's words or the generated code, named in the description as a chapter of the site to update.
17. The checks under "Building and testing", run at the reviewed revision, without changing the checkout the author works in.

The report lists findings, each marked blocking or not, with its `file:line` and a fix, then a verdict: approve, or changes needed. A finding the author rejects goes back to the reviewer with the reason, and a blocking one closes only when the reviewer agrees or the maintainer decides. A pull request is reported ready to the maintainer only once both the review and the CI of its latest push have passed.

## Building and testing

The toolchain is pinned in `rust-toolchain.toml`. Before pushing, run from the repository's root:

- `cargo xtask ci`, the quality gate CI runs: formatting, clippy with every warning an error on the default and the `unstable` builds, the architecture check (no compiler type in `domain/` or `use_cases/`, no `mod.rs`, files under about 200 lines), every unit test and Gherkin scenario on both builds, every doctest, the documentation of both builds and the crate's README against the root README, the dependencies with cargo-deny, and a publish dry run of the crates. docs/WORKFLOW.md, section 7, lists each check and how to install cargo-deny;
- `cargo xtask book-check`, on a branch that has the site, when the change alters a diagnostic or generated code the book shows.
