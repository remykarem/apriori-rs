# Association rule mining with apriori

For educational purpose again =)

## Installation

First install Rust

```sh
curl https://sh.rustup.rs -sSf | sh -s -- -y
```

then install the package

```sh
pip install git+https://github.com/remykarem/apriori-rs.git
```

To compile the module yourself (macOS),

```sh
cargo rustc --release -- -C link-arg=-undefined -C link-arg=dynamic_lookup && mv target/release/libapriori.dylib ./apriori.so
```

## Usage

```python
>>> from apriori import apriori

>>> transactions = [
...    set(["bread", "milk", "cheese"]),
...    set(["bread", "milk"]),
...    set(["milk", "cheese", "bread"]),
...    set(["milk", "cheese", "bread"]),
...    set(["milk", "cheese", "yoghurt"]),
...    set(["milk", "bread"])]

>>> rules, _ = apriori(transactions, min_support=0.3, min_confidence=0.2, max_len=3)
```

```python
>>> rules
[{"cheese", "bread"} -> {"milk"},
 {"cheese"} -> {"milk"},
 {"bread"} -> {"milk"},
 {"milk"} -> {"bread"},
 {"milk", "cheese"} -> {"bread"},
 {"cheese"} -> {"bread", "milk"},
 {"cheese"} -> {"bread"},
 {"milk"} -> {"cheese"},
 {"bread", "milk"} -> {"cheese"},
 {"bread"} -> {"milk", "cheese"},
 {"bread"} -> {"cheese"},
 {"milk"} -> {"cheese", "bread"}]
```

## Resources / references

- https://github.com/tommyod/Efficient-Apriori
