import time
from apriori import apriori
import pandas as pd

df_retail = pd.read_csv(
    "/Users/raimibinkarim/Downloads/CS5228/cs5228-assignment-1b/data/online-retail.csv")
transactions = df_retail.groupby(['Invoice']).agg(
    {'StockCode': tuple})['StockCode'].to_list()

# transactions = [
#     ("bread", "milk", "cheese"),
#     ("bread", "milk"),
#     ("milk", "cheese", "bread"),
#     ("milk", "cheese", "bread"),
#     ("milk", "cheese", "yoghurt"),
#     ("milk", "bread")]

tic = time.time()
apriori(transactions[:30], min_support=1/100, min_confidence=0.4, max_len=4)
toc = time.time()
print(toc-tic)
