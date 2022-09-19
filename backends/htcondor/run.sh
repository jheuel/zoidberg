#!/usr/bin/bash

# apparently this vodoo kills all processes opened in this script
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

ssh -N -L 8080:localhost:8080 lxplus7103 &

sleep 10
/afs/cern.ch/work/j/jheuel/zoidberg/target/debug/zoidberg_client http://localhost:8080
