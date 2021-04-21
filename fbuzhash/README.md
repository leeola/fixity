# fbuzhash

**NOTE**: This crate is a port of https://github.com/silvasur/buzhash and will focus on a subset of parity
and correctness. The only addition i expect to make are some golden-master tests to ensure parity of
ported code. Anything else is out of scope.

Package buzhash implements a buzhash algorithm using this defintion <http://en.wikipedia.org/wiki/Rolling_hash#Cyclic_polynomial>.

## Rolling hash

Buzhash is a rolling hash function, that means that the current hash sum is the sum of the last n consumed bytes.

Example:

* Message 1: <code>This is a stupid example text to demo<strong>nstrate buzhash.</strong></code>
* Message 2: <code>Another text to demo<strong>nstrate buzhash.</strong></code>

When hashing both messages with a buzhasher with n=16, both messages will have the same hash sum, since the last 16 characters (`nstrate buzhash.`) are equal.

This can be useful, when searching for a data fragment in large files, without knowing the fragment (only its hash). This is used in binary diff tools, such as `rdiff` (although they use a different rolling hash function).
