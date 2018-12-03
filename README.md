# httpwsrep
HTTP status codes for galera cluster `wsrep_*`

## HAProxy example

To check the galera clustger using `httpchk` by checking `SHOW GLOBAL STATUS LIKE 'wsrep_local_state_comment';`

    backend galera
        mode tcp
        option httpchk
        server node0 node0:3306 check port 9200
        server node1 node1:3306 check port 9200
        server node2 node2:3306 check port 9200
