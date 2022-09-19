#!/usr/bin/env python

import sys
import requests

resp = requests.post("http://localhost:8080/status", json=[{"id": int(sys.argv[1])}])

translation = {
    "Submitted": "running",
    "Completed": "success",
    "Failed": "failed",
}

print(translation[resp.json()[0]["status"]])
