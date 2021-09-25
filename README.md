# Association rule mining with apriori

For educational purpose again =)

## Installation

```sh
pip install git+https://github.com/remykarem/apriori.git

```

To compile the module yourself,

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
>>> rules, itemsets = apriori(transactions, min_support=0.3, min_confidence=0.2, max_len=3)
```

## Resources / references

* https://github.com/tommyod/Efficient-Apriori
