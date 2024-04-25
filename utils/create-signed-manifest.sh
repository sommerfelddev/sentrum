#|/usr/bin/env sh

set -e

script_dir=$(dirname -- "$(readlink -f "$0")")

tag=$(git -C "$script_dir" describe --tags --abbrev=0)

cwd=$(pwd)
cd "$(mktemp -d)"

wget "https://github.com/sommerfelddev/sentrum/releases/download/$tag/sentrum-$tag-darwin-x86_64.tar.gz"
wget "https://github.com/sommerfelddev/sentrum/releases/download/$tag/sentrum-$tag-linux-x86_64.tar.gz"
wget "https://github.com/sommerfelddev/sentrum/releases/download/$tag/sentrum-$tag-linux-aarch64.tar.gz"
wget "https://github.com/sommerfelddev/sentrum/releases/download/$tag/sentrum-$tag-windows-x86_64.zip"

sha256sum -b -- * > sentrum-"$tag"-manifest.txt

sha256sum --check sentrum-"$tag"-manifest.txt

gpg -b --armor sentrum-"$tag"-manifest.txt

gpg --verify sentrum-"$tag"-manifest.txt.asc

pwd

cd "$cwd"
