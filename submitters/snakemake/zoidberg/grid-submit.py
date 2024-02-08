#!/usr/bin/env python3

import sys
import requests
from os import environ

from snakemake.utils import read_job_properties

jobscript = sys.argv[1]
job_properties = read_job_properties(jobscript)

payload = {
    "cmd": jobscript
}

payload["threads"] = job_properties.get("threads", 1)

resp = requests.post(
    "http://localhost:8080/submit",
    json=[
        payload,
    ],
    headers={"cookie": environ["ZOIDBERG_SECRET"]},
)
assert resp.ok, "http request failed"

print(resp.json()[0]["id"])
