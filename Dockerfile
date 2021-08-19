FROM ubuntu:20.04

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libsqlite3-0 \
    cron

ADD target/release/photo-archiver /app/

ADD entry.sh /app/

# self-documentation
ADD Dockerfile /app/

CMD ["/bin/bash", "entry.sh"]
