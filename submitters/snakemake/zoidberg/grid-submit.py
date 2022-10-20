#!/usr/bin/env python3

import sys
import requests
from os import environ

jobscript = sys.argv[1]

resp = requests.post(
    "http://localhost:8080/submit",
    json=[
        {"cmd": jobscript},
    ],
    headers={"cookie": environ["ZOIDBERG_SECRET"]},
)
assert resp.ok, "http request failed"

print(resp.json()[0]["id"])
