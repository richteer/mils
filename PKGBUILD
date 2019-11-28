# Maintainer: Eric Richter <richteer@lastprime.net>
pkgname=mils
pkgver=0.1.0
pkgrel=1
makedepends=('rust' 'cargo')
depends=('mediainfo')
arch=('i686' 'x86_64' 'armv6h' 'armv7h')
license=('MIT')

build() {
    cargo build --release
}

package() {
	mkdir -p $pkgdir/usr/bin/
	cp $srcdir/../target/release/mils $pkgdir/usr/bin/
}
