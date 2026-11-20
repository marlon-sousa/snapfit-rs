# The `snapfit-rs` Documentation Site & Book Builder

An [Astro](https://astro.build) static site generator that builds the `snapfit-rs` front page (from the stripped root [README.md](../README.md)), the Releases page (`/releases/`), and **The Snapfit Book** (`/book/<release>/`) from [../book/](../book/README.md).

## Reading it locally

```sh
cd site
npm ci
npm run dev
```

Then open <http://localhost:4321/snapfit-rs/>.

## Environment variables (`book.config.mjs`)

- `BOOK_VERSION`: The version segment for the book edition (default: `v0.1.0`), served at `/snapfit-rs/book/<version>/`.
- `BOOK_SOURCE_REF`: The git ref (`main` or tag `v0.1.0`) that links to repository files point at on GitHub.
- `BOOK_ROOT`: The repository root directory (defaults to `..`).
- `SNAPFIT_RELEASES_FILE`: Optional path to an existing `snapfit-site.json` manifest (such as the one on the `site` branch) to merge previously published releases into the front page and `/releases/`.
- `SNAPFIT_PUBLISHED_PAGES`: Optional path to the `site` branch's pages, read by `scripts/pages.mjs`. A link to an edition other than `BOOK_VERSION` is looked for there too, since a build makes only its own edition. Set it with `SNAPFIT_RELEASES_FILE` when that file lists older editions.

## Verification gate (`npm run verify`)

`npm run verify` runs seven checks in sequence and stops at the first failure:

1. `npm test` — Tests README stripping rules and where the page check looks for a link to an older edition, and verifies `scripts/publish.sh` preserves prior release editions and `_astro/` assets.
2. `npm run check` — Astro and TypeScript type check.
3. `npm run contrast` — WCAG 2.1 AA contrast verification of all color pairs in `src/styles/tokens.css` across light and dark schemes, plus `theme-color` synchronization.
4. `npm run build:strict` — Forced production build (`astro build --force`) that fails on any error output.
5. `npm run pages` — Verifies that the front page, `/releases/`, `/book/`, `/book/<version>/`, every chapter in `../book/chapters/`, `.md` copies, `llms.txt`, `snapfit-site.json`, and `404.html` were built, that each HTML page has one `<h1>` and a non-empty `<main>`, and that all internal links and fragments resolve, in the build or, for an older edition, in `SNAPFIT_PUBLISHED_PAGES`.
6. `npm run a11y` — Runs `@axe-core/playwright` across every built page in both light and dark color schemes once Mermaid diagrams are rendered.
7. `npm run layout` — Verifies every built page at `360`, `390`, `768`, and `1280` pixels wide to ensure zero horizontal page overflow.

## Publishing to the `site` branch

`scripts/publish.sh` manages the `site` branch worktree so each release adds its frozen `/book/<version>/` directory and updates the root front page, `/releases/`, and `/book/` index while leaving all previously published release folders untouched.
