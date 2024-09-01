build:
    cargo build

docker-image:
    docker build -t beamup-shelltest .

docker-run-shelltests: docker-image
    docker run -v $(pwd):/app beamup-shelltest in-docker-shelltests

# have to copy to a new dir outside of the mounted volume or
# we get an error when the link is created
in-docker-shelltests: build
    mkdir /tmp/app
    cp -R . /tmp/app
    cd /tmp/app && just shelltests

shelltests:
    shelltest -c --diff --all shelltests/*.test
