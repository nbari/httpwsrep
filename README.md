# httpwsrep
HTTP status codes for galera cluster

[![crates.io](https://img.shields.io/crates/v/httpwsrep.svg)](https://crates.io/crates/httpwsrep)
[![Build Status](https://travis-ci.org/nbari/httpwsrep.svg?branch=master)](https://travis-ci.org/nbari/httpwsrep)

[![example](https://img.youtube.com/vi/yylV9WntnB4/0.jpg)](https://youtu.be/yylV9WntnB4)

This helps to check the galera cluster using `httpchk` which queries the galera
node and gets its current state `SHOW STATUS LIKE 'wsrep_local_state';`

if `wsrep_local_state` == `4` it will return `HTTP 200 OK`, otherwise `HTTP 503 Service Unavailable`

The posible values for `wsrep_local_state` are:

|Num|Comment|Description|
|---|-------|-----------|
| 1 | Joining | Node is joining the cluster
| 2 | Donor/Desynced | Node is the donor to the node joining the cluster
| 3 |  Joined | Node has joined the cluster
| 4 |  Synced | Node is synced with the cluster


## HAProxy example

    backend galera
        mode tcp
        option httpchk
        default-server check port 9200
        server node0 10.0.0.1:3306
        server node1 10.0.0.2:3306
        server node2 10.0.0.3:3306


## httpwsrep

You need to run `httpwsrep` in each galera node preferably using a supervisor,
for example if using [immortal](https://immortal.run) you could create
`/usr/local/etc/immortal/httpwsrep.yml` with something like this:

    cmd: /path/to/httpwsrep
    env:
        DSN: mysql://haproxy@tcp(10.0.0.1:3306)/
    log:
        file: /var/log/httpwsrep.log

> a valid mysql user needs to be created, in this case the user is `haproxy`

By default port `9200` is used but if required can change it using option `--port`
