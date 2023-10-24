import urllib.request
import time
import os
from tqdm import tqdm

URLS_PATH = "./resources/images_urls.csv"
OUTPUT_PATH = "../DeepBee/original_images"
DELAY_BETWEEN_REQUESTS = 5

if not os.path.exists(OUTPUT_PATH):
    os.makedirs(OUTPUT_PATH)

with open(URLS_PATH, "r") as file:
    urls = [i.strip().split(",") for i in file.readlines()]

for filename, url in tqdm(urls):
    urllib.request.urlretrieve(url, os.path.join(OUTPUT_PATH, filename))
    #time.sleep(DELAY_BETWEEN_REQUESTS)
