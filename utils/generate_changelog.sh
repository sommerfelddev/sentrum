#|/usr/bin/env sh

set -e

script_dir=$(dirname -- "$(readlink -f "$0")")
root_dir=$script_dir/..

cwd=$(pwd)
cd "$root_dir"

version=$(head -n 1 docs/CHANGELOG.md | cut -f 1 -d ' ')

sed '/^$/q' docs/CHANGELOG.md

printf 'Verifying the release
---------------------

0. Import my gpg public key into your keyring (you only need to do this once, not for every release):

```bash
gpg --auto-key-locate clear,wkd --locate-keys sommerfeld@sommerfeld.dev
```

1. Download `sentrum-v%s-manifest.txt` and `sentrum-v%s-manifest.txt.asc` to the same directory where you are downloading the binary.
2. Verify the gpg signature is mine (should ouput `Good signature`)`:

```bash
gpg --verify sentrum-v%s-manifest.txt.asc
```

3. Verify the checksums of the binaries (should output `OK`):

```bash
sha256sum --check  --ignore-missing sentrum-v%s-manifest.txt
```
' "$version" "$version" "$version" "$version"

cd "$cwd"
