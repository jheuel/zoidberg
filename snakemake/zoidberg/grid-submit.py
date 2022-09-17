#!/usr/bin/env python3

import sys
import requests

jobscript = sys.argv[1]

resp = requests.post(
    "http://localhost:8080/submit",
    json=[
        {"cmd": jobscript},
    ],
)
assert resp.ok, "http request failed"

print(resp.json()[0]["id"])
