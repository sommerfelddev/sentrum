#|/usr/bin/env sh

script_dir=$(dirname -- "$(readlink -f "$0")")

sed '/^$/q' "$script_dir"/../docs/CHANGELOG.md

echo '## Verifying the release

0. Import my gpg public key into your keyring (you only need to do this once, not for every release):

```bash
gpg --auto-key-locate clear,wkd --locate-keys sommerfeld@sommerfeld.dev
```

1. Download `sentrum-%s-manifest.txt` and `sentrum-%s-manifest.txt.asc` to the same directory where you are downloading the binary.
2. Verify the gpg signature is mine (should ouput `Good signature`)`:

```bash
gpg --verify sentrum-%s-manifest.txt.asc
```

3. Verify the checksums of the binaries (should output `OK`):

```bash
sha256sum --check  --ignore-missing sentrum-%s-manifest.txt
```' | sed "s/%s/$1/g"
