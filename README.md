# xagrs

xagrs is an xargs clone in Rust.

This is mainlytme practicing Rust. _Do not follow me, I'm also lost._

## Usage

You _should_ be able to replace xargs with xagrs:

```
$ echo 'world' | xagrs echo 'hello,'
hello, world
$ echo "world\nmy friend" | xagrs -L1 -iXX echo 'hello, XX!'
hello, world!
hello, my friend!
```
