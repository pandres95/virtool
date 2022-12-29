# Virtool: Virto tools CLI

Toolkit to prove the usability of tools behind [Virto Network](https://virto.network)'s framework. It uses [`libwallet`](https://github.com/virto-network/libwallet) and [`sube`](https://github.com/virto-network/sube) to work.

## Getting Started

### Requisites

#### Rust

Since under the hood, `sube` uses some unstable features (like `async_trait`s), you'll need to install the latest version from the **nightly** channel.

[Here](https://www.rust-lang.org/tools/install) you can find the instructions on how you can install Rust toolchain. After doing so, update to the latest version on `nightly`, running:

```sh
rustup update nightly
```

After that, you'll need to add `~/.cargo/bin` to the session `PATH`.

```sh
# If you use bash
echo "export PATH=$PATH:$HOME/.cargo/bin" >> ~/.bashrc
# If you use zsh
echo "export PATH=$PATH:$HOME/.cargo/bin" >> ~/.zshrc
```

#### Dependencies

For this specific proof of concept, the tool uses the following dependencies:

**prs** We use it as a storage backend for your accounts (see [src/vault/pass.rs](https://github.com/virto-network/libwallet/blob/master/src/vault/pass.rs) on `libwallet` to get more details).

To install a pretty handful CLI to manage your accounts run:

```sh
cargo install prs-cli
```

**The [Westend](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fwestend-rpc.polkadot.io#/chainstate) chain**

Even though you can change it to use any other chain (see [cmd/balance.rs:15](./src/cmd//balance.rs) and [cmd/transfer.rs:19](./src/cmd/transfer.rs) to change them).

### Installing from source

Download the source repository

```sh
git clone https://github.com/pandres95/virtool.git
cd virtool
```

Update the dependencies on your local machine:

```sh
cargo update
```

To build and install:

```sh
cargo +nightly install --path .
```

### Setting up your account

First, you'll need to specify the name you'll use for storing your account locally. Let's say it's `acct`.

If you previously have a mnemonic seed to setup your account, you can add it using `prs` CLI:

```sh
prs add libwallet_accounts/acct
```

Otherwise, go to the next section.

## Usage

When running `virtool`, the program shows the following help:

```sh
virtool 1.0.0

USAGE:
    virtool --uname <uname> <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -u, --uname <uname>

SUBCOMMANDS:
    balance
    help        Prints this message or the help of the given subcommand(s)
    transfer
```

### Get your balance

To find the balance for you `acct` account, run:

```sh
virtool -u acct balance
```

### Make a transfer

To make a transfer from you `acct` account to another account, run:

```sh
virtool -u acct transfer --dest <dest> --value <value>
```

Where:

* `dest` is the address for the destination account. This can be either in SS58 format or in Public Key Hex (`0x`) format.
* `value` is the amount to transfer in the smallest unit (e.g. Westend's smallest unit is 10<sup>-12</sup> WND).