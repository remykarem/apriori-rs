# apriori-rs

Fast implementation of apriori algorithm for association rule mining. Written in Rust 🦀 with Python bindings.

This implementation uses multithreading using the [Rayon](https://github.com/rayon-rs/rayon) library.

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

### Generating frequent itemsets

Prepare the data as a list of sets of strings.

```python
>>> from apriori import generate_frequent_itemsets

>>> transactions = [
...    set(["bread", "milk", "cheese"]),
...    set(["bread", "milk"]),
...    set(["milk", "cheese", "bread"]),
...    set(["milk", "cheese", "bread"]),
...    set(["milk", "cheese", "yoghurt"]),
...    set(["milk", "bread"])]
```

Then

```python
>>> itemsets, id2item = generate_frequent_itemsets(transactions, min_support=0.5, max_length=3)

>>> itemsets[1]
{frozenset({2}): 4, frozenset({0}): 5, frozenset({1}): 6}

>>> itemsets[2]
{frozenset({0, 1}): 5, frozenset({1, 2}): 4, frozenset({0, 2}): 3}

>>> itemsets[3]
{frozenset({0, 1, 2}): 3}
 
>>> id2item
{2: 'cheese', 0: 'bread', 3: 'yoghurt', 1: 'milk'}
```

Use `generate_frequent_itemsets_id` if your items are indices.

### Association rules

```python
>>> rules, counts = apriori(
...     transactions, 
...     min_support=0.3, 
...     min_confidence=0.2, 
...     max_length=3)
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

Obtain confidence and lift for a rule.

```python
>>> rules[0]
{"bread", "cheese"} -> {"milk"}

>>> rules[0].confidence
1.0

>>> rules[0].lift
1.0
```

## Resources / references
## Contributing

See any opportunities for better memory management or improvement in speed? Feel free to submit a PR :)
