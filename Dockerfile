FROM opensuse/leap:15.6

WORKDIR /app

RUN zypper --non-interactive install shelltestrunner rustup && rustup toolchain install stable

RUN zypper --non-interactive install just

ENTRYPOINT ["just"]
