#!/usr/bin/env bash
# Prints true when the release of <tag> publishes its book edition and front
# page onto the site branch, and false when it does not, as docs/WORKFLOW.md,
# "Releases", says: only a version without a pre-release part, such as
# v1.2.0, is published there. A release candidate, an alpha, a beta and any
# other pre-release are not, since the book teaches what a release holds.
# Usage: site.sh <tag>
set -euo pipefail

tag=$1
# Build metadata, after a +, is not a pre-release part.
identifiers='[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*'
if [[ $tag =~ ^v[0-9]+\.[0-9]+\.[0-9]+(\+$identifiers)?$ ]]; then
    echo true
else
    echo false
fi
