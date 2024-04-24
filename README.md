# sentrum

![Crates.io Total Downloads](https://img.shields.io/crates/d/sentrum?style=for-the-badge&label=crates.io)
![AUR Version](https://img.shields.io/aur/version/sentrum?style=for-the-badge)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/sommerfelddev/sentrum/ci.yml?style=for-the-badge&label=CI)
![GitHub Release](https://img.shields.io/github/v/release/sommerfelddev/sentrum?style=for-the-badge)

Daemon that monitors the Bitcoin blockchain for transactions involving your
wallets and sends you notifications in many different channels (ntfy push
notifications, email, telegram, nostr, arbitrary command, etc).

Example screenshot of many noifications for a single transaction:
<p align="center">
  <img src="https://i.nostr.build/nWkqo.jpg" width="540" height="692">
</p>

## Installation

Either:

* Compile from source using `cargo install sentrum` (binary will be installed in
`~/.cargo/bin`)
* Download the binary from the
[releases page](https://github.com/sommerfelddev/sentrum/releases)
* If using archlinux, install it from the AUR:
[sentrum](https://aur.archlinux.org/packages/sentrum),
[sentrum-bin](https://aur.archlinux.org/packages/sentrum-bin) or
[sentrum-git](https://aur.archlinux.org/packages/sentrum-git)

## Configuration

### Config file path

It will look for a `sentrum.toml` configuration file located in any of these
directories (with this priority):

1. Current working directory
2. `$XDG_CONFIG_HOME/sentrum`
3. `~/.config/sentrum`
4. `/etc/sentrum` (more appropriate if running as a systemd service)

Alternatively, you can pass the configuration file path as an argument in the
invocation and that will override any of the above.

**Start by copying the sample configuration to where you want it.** E.g.

```bash
cp sentrum.sample.toml sentrum.toml
sudo cp sentrum.sample.toml /etc/sentrum/sentrum.toml
```

or

```bash
sudo cp sentrum.sample.toml /etc/sentrum/sentrum.toml
```

### What to configure

You can use the [sentrum.sample.toml](sentrum.sample.toml) file as an
example.
Most options have very good defaults so you don't need to change them unless you
want to. **In the examples below, commented options showcase their defaults,
unless explicitly said otherwise.**

#### Required

* `wallets`: what wallets you want to monitor
* `actions`: what actions you want to take once a relevant transaction is found

#### Optional

* `electrum`: by default, public electrum servers are used. You can configure it
  to connect to your own
* `message`: this allows you to configure the subject and body templates of the
  notification message and choose the relevant data from the transaction that
you want to include

## Wallets

For each wallet you want to track, add the following configuration:

```toml
[[wallets]]
# Identifier for naming purposes (required)
name = "alice"
# Wallet xpub (required)
xpub = "xpub6CkXHzuU1NyHUFNiQZLq2bgt6QPqjZbwpJ1MDgDeo4bWZ8ZP7HZr7v9WTLCQFhxVhqiJNcw5wSKE77rkAK1SzcuHjt36ZUibBHezGzGL9h9"
# Script kind ("legacy","nested_segwit","segwit","taproot") (optional)
#kind = "segwit"
```

It assumes a BIP84 (native segwit, `bc1` style addresses) wallet. If your wallet
has a different script kind add the field `kind = "legacy"` (or `nested_segwit`,
or `taproot`).

More complex wallet types are supported by providing `primary = "<desc>"` and
`change = "<desc>"` wallet descriptors instead of `xpub =` and `kind = `.

## Actions

For each new relevant transaction, you can take multiple actions. For each
action you desire to take, you need to add the configuration:

```toml
[[actions]]
# Action type (required)
type =  "<INSERT ACTION KIND>"
<.... INSERT ACTION SPECIFIC CONFIGURATION HERE...>
```

Below we explain the configuration for each action kind. You can have multiple
actions of the same kind (e.g. you want to send multiple emails from different
accounts for some reason).

### ntfy

This is the best straightforward way to get push notifications on a smartphone.

1. Install the android/iOS app following the relevant links in https://ntfy.sh
2. If you don't run your own ntfy self-hosted server, create an account at
   ntfy.sh
3. Open the app, give it the needed permissions and configure your account
   credentials
4. Click on the `+` button and create a "topic", preferably named `sentrum`
   since that's what will be used by default.

Then you just need to add the relevant configuration:

```toml
[[actions]]
type =  "ntfy"
# Credentials (required if you use a public server like the default one)
credentials.username = "<YOUR USERNAME HERE>"
credentials.password = "<YOUR PASSWORD HERE>"
# ntfy server (optional)
#url = "https://ntfy.sh"
# notification channel name (optional)
#topic = "sentrum"
# Proxy used to connect (optional, defaults to None)
#proxy = "socks5://127.0.0.1:9050"
# Priority ("max", "high", "default", "low", "min") (optional)
#priority = "default"
```

### nostr

Get notified by a nostr [NIP04 encrypted
DM](https://github.com/nostr-protocol/nips/blob/master/04.md) (leaks metadata
but widely supported) or a
[NIP59 GiftWrap sealed sender DM](https://github.com/nostr-protocol/nips/blob/master/59.md)
(more private but not supported by many clients). Add:

```toml
[[actions]]
type = "nostr"
# Which npub to send the DM (required)
recipient = "<YOUR npub, hex pubkey, nprofile or nip05>"
# If NIP59 giftwrap DMs should be used instead of NIP04 (optional)
#sealed_dm = false
# Which relays to use to send DMs
#relays = ["wss://nostr.bitcoiner.social", "wss://nostr.oxtr.dev", "wss://nostr.orangepill.dev", "wss://relay.damus.io"]
```

### email

You need to add the configuration below and essentially configure an
authenticated connection to your email provider's SMTP server. I cannot help you
out with every provider's weird rules (maybe you need to allow 3rd party apps
for gmail, who knows).

```toml
[[actions]]
type =  "email"
# SMTP server (required)
server = "<insert smtp server url (e.g. smtp.gmail.com)"
# SMTP connection type ("tls", "starttls" or "plain") (optional)
#connection = "tls"
# SMTP port (optional, defaults to 587 for TLS, 465 for STARTTLS and 25 for plain connections
#port = 1025
# SMTP credentials (required in most cases)
credentials.authentication_identity = "<insert login email>"
credentials.secret = "<insert password>"
# Accept self signed certificates (needed if you are using protonmail-bridge) (optional)
#self_signed_cert = false
# Configure sender (required)
from = "sentrum <youremailhere@host.tld>"
# Configure recipient (optional, defaults to the same as the "from" sender)
#to = "sentrum <youremailhere@host.tld>"
```

### telegram

1. Create a new bot using [@Botfather](https://t.me/botfather) to get a token in the format `123456789:blablabla`.
2. Optionally configure the bot (name, profile pic, etc) with @Botfather
3. Open a chat with your bot
4. Add the relevant config:

```toml
[[actions]]
type =  "telegram"
# Auth token of the bot created with @Botfather (required)
bot_token = "<insert bot token>"
# 10-digit user id of the DM recipient, go to your profile to get it (required)
user_id = 1234567890
```

### command

Runs an external command where you can use transaction details as arguments.
You can check what parameters (such as `{wallet}` or `{tx_net}` you can use in
the [message](#message) configuration, since they are the same.

```toml
[[actions]]
type = "command"
cmd = "notify-send"
args = ["[{wallet}] new tx: {tx_net} sats"]
```


### terminal_print

Justs prints the notification text in the terminal. You can potentially pipe it
to something else.

```toml
[[actions]]
type =  "terminal_print"
```

### desktop_notification

Displays the transaction message as a native desktop notification on the same
computer sentrum is running.

```toml
[[actions]]
type =  "desktop_notification"
```

## Message

You can configure the message template and it applies to almost every action
type. This configuration is entirely optional since the default templates will
be used if omitted.

Here is the default template:

```toml
[message]
subject = "[{wallet}] new transaction"
body = "net: {tx_net} sats, balance: {total_balance} sats, txid: {txid_short}"
# Can be "plain", "markdown" or "html"
format = "plain"
# Configure blockexplorer urls. This is used to create the {tx_url} parameter
block_explorers.mainnet = "https://mempool.space/tx/{txid}"
block_explorers.testnet = "https://mempool.space/testnet/tx/{txid}"
block_explorers.signet = "https://mempool.space/signet/tx/{txid}"
```

In the subject and body templates, you can use the following parameters:

* `{tx_net}`: difference between the owned outputs and owned inputs
* `{wallet}`: name of the configured wallet
* `{total_balance}`: total balance of the wallet
* `{txid}`: txid of the transaction
* `{txid_short}`: truncated txid, easier on the eyes
* `{received}`: sum of owned outputs
* `{sent}`: sum of owned inputs
* `{fee}`: transaction fee
* `{current_height}`: current blockheight
* `{tx_height}`: blockheight transaction confirmation
* `{confs}`: number of transaction confirmations (0 for unconfirmed)
* `{conf_timestamp}`: timestamp of the first confirmation in the `%Y-%m-%d %H:%M:%S` format
* `{tx_url}`: a block explorer URL to the transaction

## Electrum server

By default, public electrum servers will be used. I **strongly suggest
configuring your own electrum server if you want privacy (as you should)**.

The defaults are:

```toml
[electrum]
# Defaults:
# - mainnet: ssl://fulcrum.sethforprivacy.com:50002
# - testnet: ssl://electrum.blockstream.info:60002
# - signet: ssl://mempool.space:60602
# Use "tcp://" if you are connecting without SSL (e.g. "tcp://localhost:50001"
# or "tcp://fwafiuesngirdghrdhgiurdhgirdgirdhgrd.onion:50001"
url = "ssl://fulcrum.sethforprivacy.com:50002"
# blockchain network ("bitcoin", "testnet", "signet", "regtest")
network = "bitcoin"
# Optional socks5 proxy (defaults to None)
#socks5 = 127.0.0.1:9050
# If using ssl with a trusted certificate, set this to true
certificate_validation = false
```

# Usage

Just run `sentrum` without arguments (uses default config search paths) or
`sentrum <path/to/config/file>`.

You can pass the `--test` flag to send a single test notification to all
configured actions.

By default, only new transactions can trigger actions. If you pass
`--notify-past-txs`, it will send notifications of past transactions
in the initial wallet sync. If you have a long transaction history, this will
spam your notification channels for every transaction.

## systemd service

The ideal use-case is as a long running daemon, so it makes sense to configure
it as a systemd service.

If you installed sentrum from the AUR, you just need to edit
`/etc/sentrum/sentrum.conf` and do `sudo systemclt enable --now sentrum.service`

If you are installing `sentrum` manually (e.g. from the releases page or `cargo
install`), you should (either from the cloned repository or from inside the
extracted release archive):

1. Copy systemd files to appropriate places:

```bash
sudo cp contrib/systemd/sentrum.service
sudo cp contrib/systemd/sentrum.sysusers /etc/sysusers.d/sentrum.conf
sudo cp contrib/systemd/sentrum.tmpfiles /etc/tmpfiles.d/sentrum.conf
```

2. Reload systemd daemon, sysusers and tmpfiles:

```bash
sudo systemclt daemon-reload
sudo systemd-sysusers
sudo systemd-tmpfiles --create
```

3. Place the `sentrum.toml` (or `sentrum.sample.toml`) configuration file in
`/etc/sentrum` and make sure the `sentrum` user owns it:

```bash
sudo cp sentrum.toml /etc/sentrum
sudo chown sentrum:sentrum /etc/sentrum/sentrum.toml
```

4. Enable and start the service:

```bash
sudo systemclt enable --now sentrum.service
```

5. Check if everything is fine with `systemctl status sentrum`

6. Check the logs with `journalctl -fu sentrum`

## Docker

To run sentrum using Docker, you can either build the image yourself or use the prebuilt image.

### Building the image

To build the image from source, run the following:

```bash
git clone https://github.com/sommerfelddev/sentrum.git
cd sentrum
docker build -t sentrum:local .
```

To use the prebuilt image, simply pull from GHCR:

```bash
docker pull docker pull ghcr.io/sommerfelddev/sentrum:latest
```

Note that there are two types of tags:

`latest`: a tag from the latest commit to master
`x.x.x`: (i.e. `0.1.1`) a tag of the corresponding sentrum version

### Running the image

To run the image, simply run the following, passing in the `sentrum.toml` file you created and configured earlier:

```bash
docker run --rm -it --volume ./sentrum.toml:/sentrum.toml ghcr.io/sommerfelddev/sentrum:latest
```

If using Docker compose, you can configure the service as follows:

```yaml
services:
  sentrum:
    container_name: sentrum
    image: ghcr.io/sommerfelddev/sentrum:latest
    restart: unless-stopped
    volumes:
      - ./sentrum.toml:/sentrum.toml
```

## Future Work

* More action types:
    - Matrix DM
    - SimpleX chat DM
    - IRC
    - XMPP
    - Whatsapp/Signal using linked devices (harder)
    - HTTP request
* More wallet types:
    - Single Address (blocked by
    https://github.com/bitcoindevkit/bdk/issues/759)
    - Collections of wallets as a single entity
* Notifications for the first tx confirmation and after N confirmations
* Filtering notifications by the transaction amounts (e.g. no action for
transactions smaller than 1M sats)
* Debian package (using `cargo-deb`)
* Allow per wallet actions
* Support other blockchain backends (bitcoind-rpc, explora, block filters, dojo)
* Maybe create a little web UI that helps with writing the configuration
* Incentivize node distributions to package sentrum
