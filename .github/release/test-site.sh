#!/usr/bin/env bash
# Checks site.sh against docs/WORKFLOW.md, "Releases", with the tags below.
# Run from the repository's root.
set -euo pipefail

failures=0
# expect <tag> <expected output>
expect() {
    local tag=$1 expected=$2 actual
    actual=$(.github/release/site.sh "$tag")
    if [ "$actual" != "$expected" ]; then
        echo "$tag: printed \"$actual\", expected \"$expected\"" >&2
        failures=$((failures + 1))
    fi
}

# A version without a pre-release part is published.
expect v0.1.0 true
expect v1.2.0 true
expect v10.20.30 true
# Build metadata is not a pre-release part.
expect v1.0.0+build.5 true
# No pre-release is, not even a release candidate.
expect v0.2.0-rc.1 false
expect v0.1.0-alpha.2 false
expect v0.1.0-beta.1 false
expect v1.0.0-dev.1 false
expect v1.0.0-rc.1+build.5 false
# Nor is a tag that is not a version.
expect v1.0 false
expect 1.0.0 false

if [ "$failures" -gt 0 ]; then
    echo "$failures cases of site.sh failed." >&2
    exit 1
fi
echo "site.sh tells every release whether it publishes the site."
