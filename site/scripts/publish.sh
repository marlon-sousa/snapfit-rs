#!/usr/bin/env bash
# Publishes one version of the snapfit site and book to the `site` branch,
# preserving every previously published `/book/<older-version>/` directory and
# preserving existing hashed files in `_astro/` so older book pages stay styled.
#
# Usage:
#   site/scripts/publish.sh open <pages-worktree>
#   site/scripts/publish.sh publish <pages-worktree> <dist-dir> <version> [--no-push]

set -euo pipefail

usage() {
  echo "Usage: site/scripts/publish.sh open <pages-worktree> | site/scripts/publish.sh publish <pages-worktree> <dist-dir> <version> [--no-push]" >&2
  exit 2
}

open_worktree() {
  local folder=$1
  local found=0
  git ls-remote --exit-code origin refs/heads/site >/dev/null || found=$?
  if [ "$found" -eq 0 ]; then
    git fetch --depth=1 origin refs/heads/site
    git worktree add --detach "$folder" FETCH_HEAD
  elif [ "$found" -ne 2 ]; then
    echo "origin cannot be asked whether it has a site branch." >&2
    exit 1
  else
    git worktree add --orphan -b site "$folder"
    touch "$folder/.nojekyll"
  fi
}

publish_version() {
  local folder=$1 dist=$2 version=$3 push_mode=${4:-}
  case "$version" in
    "" | . | .. | .* | */* | *\\*)
      echo "$version: a version must be a single folder name not starting with a dot." >&2
      exit 1
      ;;
  esac
  if [ ! -d "$dist/book/$version" ]; then
    echo "$dist/book/$version: no book edition folder was built for $version." >&2
    exit 1
  fi
  if [ ! -f "$dist/index.html" ]; then
    echo "$dist/index.html: no front page was built." >&2
    exit 1
  fi

  mkdir -p "$folder/book" "$folder/releases" "$folder/_astro"
  rm -rf "$folder/book/$version" "$folder/releases"
  cp -R "$dist/book/$version" "$folder/book/$version"
  cp "$dist/book/index.html" "$folder/book/index.html"
  cp -R "$dist/releases" "$folder/releases"
  cp "$dist/index.html" "$folder/index.html"
  cp "$dist/index.md" "$folder/index.md"
  cp "$dist/404.html" "$folder/404.html"
  cp "$dist/favicon.svg" "$folder/favicon.svg"
  if [ -f "$dist/snapfit-site.json" ]; then
    cp "$dist/snapfit-site.json" "$folder/snapfit-site.json"
  fi
  if [ -d "$dist/_astro" ]; then
    cp -R "$dist/_astro/." "$folder/_astro/"
  fi
  touch "$folder/.nojekyll"

  local checkout=$PWD
  cd "$folder"
  git add --all -- "book/$version" "book/index.html" "releases" "index.html" "index.md" "404.html" "favicon.svg" ".nojekyll" "_astro"
  if [ -e "snapfit-site.json" ]; then
    git add -- "snapfit-site.json"
  fi

  local stray
  stray=$(git status --porcelain --untracked-files=all | grep -v '^[MADRC] ' || true)
  if [ -n "$stray" ]; then
    echo "Unexpected changes outside $version/ and root pages; refusing to publish:" >&2
    echo "$stray" >&2
    exit 1
  fi

  if git diff --cached --quiet; then
    echo "$version is already published on the site branch."
    return
  fi

  git -c user.name="github-actions[bot]" \
    -c user.email="41898282+github-actions[bot]@users.noreply.github.com" \
    commit --quiet -m "site: $version"

  if [ "$push_mode" != "--no-push" ]; then
    local commit
    commit=$(git rev-parse HEAD)
    git -C "$checkout" push origin "$commit:refs/heads/site"
  fi
}

case "${1-}" in
  open)
    [ "$#" -eq 2 ] || usage
    open_worktree "$2"
    ;;
  publish)
    [ "$#" -eq 4 ] || [ "$#" -eq 5 ] || usage
    publish_version "$2" "$3" "$4" "${5-}"
    ;;
  *) usage ;;
esac
