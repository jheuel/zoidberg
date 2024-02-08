#!/usr/bin/bash

# apparently this vodoo kills all processes opened in this script
# trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
# ssh -N -L 8080:localhost:8080 lxplus7103 &
# sleep 10

/home/home4/institut_1b/jheuel/repositories/zoidberg/target/release/zoidberg_client
