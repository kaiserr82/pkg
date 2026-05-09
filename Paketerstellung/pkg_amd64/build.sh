#/bin/sh
rm *.deb

debuild -a amd64
debuild -a arm64
debuild clean
rm -f ../*.build ../*.changes ../*.dsc ../*.tar.*
dpkg-buildpackage -T clean
dh_clean
