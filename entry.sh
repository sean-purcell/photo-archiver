#!/bin/bash

cat > crontab <<< "
0 5 * * * /app/photo-archiver -t /data/token.json archive -R /data > /proc/1/fd/1 2> /proc/1/fd/2
"

/usr/bin/crontab crontab

cron -f
