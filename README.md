# Association rule mining with apriori

For educational purpose again =)

Python:

```python
>>> from apriori import apriori

>>> transactions = [
...    set(["bread", "milk", "cheese"]),
...    set(["bread", "milk"]),
...    set(["milk", "cheese", "bread"]),
...    set(["milk", "cheese", "bread"]),
...    set(["milk", "cheese", "yoghurt"]),
...    set(["milk", "bread"])]
>>> rules, inventory = apriori(transactions, min_support=0.3, min_confidence=0.2, max_len=3)
```

Building

```sh
cargo rustc --release -- -C link-arg=-undefined -C link-arg=dynamic_lookup && mv target/release/libapriori.dylib ./apriori.so
```
