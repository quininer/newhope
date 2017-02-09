Test
----

```
	wget https://cryptojedi.org/crypto/data/newhope-20160815.tar.bz2
	tar xf newhope-20160815.tar.bz2
	mv newhope-20160815 newhope
	cargo test
	cargo test --feature tor
	cargo bench
```
