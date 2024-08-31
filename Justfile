docker-image:
    docker build -t beamup-shelltest .

docker-run-shelltests: docker-image
    docker run -v $(pwd):/app beamup-shelltest shelltests

build:
    cargo build

shelltests: build
    shelltest -c --diff --all shelltests/*.test
