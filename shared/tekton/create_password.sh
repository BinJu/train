#!/bin/bash

export TEST_USER=testuser
export TEST_PASS=testpassword
if [ ! -f auth ]; then
    mkdir auth
fi
docker run \
 --entrypoint htpasswd \
 httpd:2 -Bbn $TEST_USER $TEST_PASS > auth/htpasswd
