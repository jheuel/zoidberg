#!/usr/bin/env python

import sys
import requests
from os import environ

def print_and_exit(s):
    print(s)
    exit(0)

resp = requests.post(
    "http://localhost:8080/status",
    json=[{"id": int(sys.argv[1])}],
    headers={"cookie": environ["ZOIDBERG_SECRET"]},
)

translation = {
    "Submitted": "running",
    "Completed": "success",
    "Failed": "failed",
}

j = resp.json()

if len(j) == 0:
    print_and_exit("failed")

if "Running" in j[0]["status"]:
    print_and_exit("running")

print_and_exit(translation[resp.json()[0]["status"]])
