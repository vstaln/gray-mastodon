#!/bin/sh
here="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
sys="$(uname -s)"
case "$sys" in
MINGW* | MSYS* | CYGWIN*) rel="x86_64-pc-windows-msvc/mastodon.exe" ;;
Darwin*)
    case "$(uname -m)" in
    arm64) rel="aarch64-apple-darwin/mastodon" ;;
    *) rel="x86_64-apple-darwin/mastodon" ;;
    esac ;;
*)
    case "$(uname -m)" in
    aarch64 | arm64) rel="aarch64-unknown-linux-musl/mastodon" ;;
    *) rel="x86_64-unknown-linux-musl/mastodon" ;;
    esac ;;
esac
if [ -x "$here/$rel" ]; then exec "$here/$rel" "$@"; fi
for cand in "$here/target/release/mastodon.exe" "$here/target/release/mastodon" "$here/target/debug/mastodon.exe" "$here/target/debug/mastodon"; do
    if [ -x "$cand" ]; then exec "$cand" "$@"; fi
done
echo "mastodon binary not found in $here" >&2
exit 1
