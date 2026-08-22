FROM ubuntu:22.04

# SIPp was removed from Ubuntu/Debian archives — build the release tarball.
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        build-essential libssl-dev libpcap-dev libncurses-dev wget ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && wget -q https://github.com/SIPp/sipp/releases/download/v3.7.3/sipp-3.7.3.tar.gz \
    && tar xzf sipp-3.7.3.tar.gz \
    && cd sipp-3.7.3 \
    && ./configure >/dev/null \
    && make -j"$(nproc)" >/dev/null \
    && make install >/dev/null \
    && cd / \
    && rm -rf /sipp-3.7.3 /sipp-3.7.3.tar.gz

WORKDIR /sipp
COPY uas_register_answer.xml .
COPY users/ users/

ENTRYPOINT ["sipp"]
