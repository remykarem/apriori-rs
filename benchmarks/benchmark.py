import time
from efficient_apriori import itemsets_from_transactions
from apriori import generate_frequent_itemsets
from mlxtend.preprocessing import TransactionEncoder
from mlxtend.frequent_patterns import apriori as frequent_itemsets_mlxtend
import pandas as pd


def get_data():
    transactions = {}

    with open("benchmarks/data/online-retail.csv", "r", encoding="utf-8") as f:
        next(f)  # skip first line
        for line in f:
            invoice, stock_code, *_ = line.split(",")
            if invoice in transactions:
                transactions[invoice].add(stock_code)
            else:
                transactions[invoice] = set([stock_code])

    transactions = list(transactions.values())

    te = TransactionEncoder()
    te_ary = te.fit(transactions).transform(transactions)
    df = pd.DataFrame(te_ary, columns=te.columns_)

    return transactions, df


def get_params():
    # for min_support in [0.1, 0.05, 0.01, 0.005]:
    for min_support in [0.01]:
        for length in [5]:
        # for length in [1, 2, 3, 4, 5]:
            yield (min_support, length)


def run_benchmark():
    transactions, df_transactions = get_data()
    params = get_params()

    for param in params:
        min_support, length = param
        times = []

        tic = time.time()
        generate_frequent_itemsets(transactions, min_support, length)
        toc = time.time()
        times.append(toc - tic)

        tic = time.time()
        itemsets_from_transactions(
            transactions, min_support=min_support, max_length=length)
        toc = time.time()
        times.append(toc - tic)

        tic = time.time()
        frequent_itemsets_mlxtend(
            df_transactions, min_support=min_support, max_len=length)
        toc = time.time()
        times.append(toc - tic)

        print(min_support, length, times)


if __name__ == "__main__":
    run_benchmark()
