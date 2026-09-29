#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
repo=xiaotianxt/keychain-secret
tap=$(brew --repo xiaotianxt/tap)
[[ -z $(git status --porcelain) ]] || { echo 'Source checkout is dirty' >&2; exit 1; }
[[ -z $(git -C "$tap" status --porcelain) ]] || { echo 'Tap checkout is dirty' >&2; exit 1; }
[[ $(git branch --show-current) == main ]] || { echo 'Release from main' >&2; exit 1; }
git fetch origin main --tags
git merge-base --is-ancestor origin/main HEAD
version=$(cargo metadata --locked --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])')
[[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo 'Expected stable semver' >&2; exit 1; }
tag=v$version
asset=keychain-secret-$tag-darwin-arm64.tar.gz
if git rev-parse "$tag" >/dev/null 2>&1; then
  [[ $(git rev-list -n 1 "$tag") == $(git rev-parse HEAD) ]] || { echo 'Tag points elsewhere' >&2; exit 1; }
else
  git tag -a "$tag" -m "$tag"
fi
git push origin main "refs/tags/$tag"
if ! gh release view "$tag" --repo "$repo" >/dev/null 2>&1; then
  run=''
  for _ in {1..12}; do
    run=$(gh run list --repo "$repo" --workflow release.yml --commit "$(git rev-parse HEAD)" --limit 1 --json databaseId --jq '.[0].databaseId // empty')
    [[ -z $run ]] || break
    sleep 5
  done
  [[ -n $run ]] || { echo 'Release run not found; inspect Actions before rerunning' >&2; exit 1; }
  gh run watch "$run" --repo "$repo" --exit-status
fi
work=$(mktemp -d "${TMPDIR:-/tmp}/keychain-release.XXXXXX")
trap 'rm -rf "$work"' EXIT
gh release download "$tag" --repo "$repo" --pattern "$asset" --dir "$work"
sha=$(shasum -a 256 "$work/$asset" | cut -d ' ' -f 1)
git -C "$tap" pull --ff-only
formula="$tap/Formula/keychain-secret.rb"
cat > "$formula" <<EOF
class KeychainSecret < Formula
  desc "Small stdin-friendly CLI for macOS Keychain secrets"
  homepage "https://github.com/$repo"
  url "https://github.com/$repo/releases/download/$tag/$asset"
  version "$version"
  sha256 "$sha"
  license "MIT"

  depends_on :macos
  depends_on arch: :arm64

  def install
    bin.install "keychain-secret"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/keychain-secret --version")
  end
end
EOF
git -C "$tap" diff --check
if [[ -n $(git -C "$tap" status --porcelain -- Formula/keychain-secret.rb) ]]; then
  git -C "$tap" add Formula/keychain-secret.rb
  git -C "$tap" commit -m "keychain-secret $version"
fi
git -C "$tap" push origin HEAD:main
printf 'Published %s. Install/upgrade with Homebrew, then run brew test keychain-secret.\n' "$tag"
