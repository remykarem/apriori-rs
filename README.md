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

## Benchmarks

Time taken (s) to generate frequent itemsets for the Online Retail II dataset (https://archive.ics.uci.edu/ml/machine-learning-databases/00502/) given minimum support and maximum length of itemset.

| Min support, length | apriori-rs | [efficient-apriori](https://github.com/tommyod/Efficient-Apriori) | [mlxtend](http://rasbt.github.io/mlxtend/user_guide/frequent_patterns/apriori/) |
|:-------------------:|:----------:|:----------------:|:-------:|
|            0.100, 1 |       0.2s |             0.1s |    0.1s |
|            0.100, 2 |       0.2s |             0.1s |    0.1s |
|            0.100, 3 |       0.2s |             0.1s |    0.1s |
|            0.100, 4 |       0.2s |             0.1s |    0.1s |
|            0.100, 5 |       0.2s |             0.1s |    0.1s |
|            0.050, 1 |       0.2s |             0.1s |    0.1s |
|            0.050, 2 |       0.2s |             0.2s |    0.1s |
|            0.050, 3 |       0.2s |             0.2s |    0.1s |
|            0.050, 4 |       0.2s |             0.2s |    0.1s |
|            0.050, 5 |       0.2s |             0.2s |    0.2s |
|            0.010, 1 |       0.2s |             0.1s |    0.1s |
|            0.010, 2 |    **16s** |             261s |     73s |
|            0.010, 3 |    **15s** |             272s |     79s |
|            0.010, 4 |    **17s** |             284s |     78s |
|            0.010, 5 |    **14s** |             279s |     92s |
|            0.005, 1 |       0.2s |             0.1s |    0.1s |
|            0.005, 2 |    **76s** |            1190s |    327s |
|            0.005, 3 |    **68s** |            1278s |    643s |
|            0.005, 4 |    **81s** |            1168s |    638s |
|            0.005, 5 |    **70s** |            1217s |    643s |

Benchmark was carried out on macOS Big Sur (11.6); 2.7 GHz Quad-Core Intel Core i7. Python version 3.8.11.

## Contributing

See any opportunities for better memory management or improvement in speed? Feel free to submit a PR :)
