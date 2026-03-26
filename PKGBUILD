# Maintainer: 123hi123 <https://github.com/123hi123>
pkgname=how2cli
pkgver=0.1.0
pkgrel=1
pkgdesc="Natural language to shell command. Type what you want, get the exact command + explanation."
arch=('x86_64' 'aarch64')
url="https://github.com/123hi123/how2cli"
license=('MIT')
makedepends=('cargo' 'git')
source=("git+https://github.com/123hi123/how2cli.git#tag=v${pkgver}")
sha256sums=('SKIP')

prepare() {
    cd "$pkgname"
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    cd "$pkgname"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --bin h --bin ht
}

package() {
    cd "$pkgname"
    install -Dm755 "target/release/h" "$pkgdir/usr/bin/h"
    install -Dm755 "target/release/ht" "$pkgdir/usr/bin/ht"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}
