#|/usr/bin/env sh

set -e

script_dir=$(dirname -- "$(readlink -f "$0")")
root_dir=$script_dir/..

cwd=$(pwd)
cd "$root_dir"

version=$(head -n 1 docs/CHANGELOG.md | cut -f 1 -d ' ')

sed -i -E 's/^version = .+$/version = "'"$version"'"/' Cargo.toml
cargo build
git add Cargo.toml Cargo.lock docs/CHANGELOG.md
git commit -m "Bump to v$version"
git tag -a v"$version" -m "$(utils/generate_changelog.sh)"

git push
git push mandibles
cargo publish

cd "$cwd"
