# To install `just`, see
# https://github.com/casey/just#packages

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

import 'config.just'

_bot_target := ROBOLAB_BOT_TARGET

# enforce using *-pc-windows-gnu by default on windows
_host_target := if os_family() == "windows" {
    arch() + '-pc-windows-gnu'
} else {
    `rustc --print host-tuple`
}

# we need to filter out self-contained musl libc files
# to avoid linking with duplicate symbols (like _start)
export CARGO_TARGET_ARMV5TE_UNKNOWN_LINUX_MUSLEABI_LINKER := if os_family() == "windows" {
    absolute_path('.cargo/bin/zcc-static-arm926ej_s.bat')
} else {
    absolute_path('.cargo/bin/zcc-static-arm926ej_s.sh')
}

# a private default (first in file) recipe to list all recipes
_default:
    @just -l

# install helpers for cross compiling the project
setup:
    rustup update
    @just _setup
    rustup target add {{ _bot_target }}

[unix]
_setup:
    sh -c 'f=$(type cargo-zigbuild)' || cargo install --locked cargo-zigbuild
    sh -c 'f=$(type cargo-nextest)' || cargo install --locked cargo-nextest

[windows]
_setup:
    if (-not (Get-Command cargo-zigbuild -ErrorAction SilentlyContinue)) { \
        cargo install --locked cargo-zigbuild \
    }
    if (-not (Get-Command cargo-nextest -ErrorAction SilentlyContinue)) { \
        cargo install --locked cargo-nextest \
    }

# build the artifacts for your bot in release mode
[group('building')]
build-bot:
    cargo zigbuild --target {{ _bot_target }} --release

# run the clippy linter
[group('house-keeping')]
clippy:
    cargo-zigbuild clippy --target {{ _bot_target }}

# build & open documentation for this project and dependencies
[group('house-keeping')]
docs:
    cargo-zigbuild doc --target {{ _bot_target }} --open --no-deps -p robolab-bot-base -p communication -p movement -p planet -p bot -p rumqttc -p ev3dev-lang-rust -p serde_json -p serde -p clap

# format your code
[group('house-keeping')]
fmt:
    cargo fmt

# run the tests
[group('house-keeping')]
test: test-communication test-planet test-movement

# test communication crate
[group('house-keeping')]
test-communication:
    cargo-zigbuild nextest run --target {{ _host_target }} --package communication -j 1

# test planet crate
[group('house-keeping')]
test-planet:
    cargo-zigbuild nextest run --target {{ _host_target }} --package planet

# test movement crate
[group('house-keeping')]
test-movement:
    cargo-zigbuild nextest run --target {{ _host_target }} --package movement

# build & deploy your code to the robot
[group('robot')]
deploy: build-bot
    scp target/{{ _bot_target }}/release/bot robot@{{ ROBOT_IP }}:
    @just run

# run your already deployed code on the bot
[group('robot')]
run: kill
    $(if which kitten 2>/dev/null; then echo "kitten ssh"; else echo "ssh"; fi) -tt robot@{{ ROBOT_IP }} "chmod +x ./bot && ((if which conrun; then conrun -ioe ./bot; else ./bot; fi) || true)"

# kill the running bot
[group('robot')]
kill:
    -ssh robot@{{ ROBOT_IP }} robolab-kill

# attach & follow the log of the current run
[group('robot')]
logcat:
    ssh robot@{{ ROBOT_IP }} "tail -f logs/latest.log"

# copy all logs from the robot
[group('robot')]
sync-logs:
    scp -r robot@{{ ROBOT_IP }}:logs .

# delete all logs on the robot
[group('robot')]
clean-logs:
    ssh robot@{{ ROBOT_IP }} 'bash -c "rm logs/* -f"'

# reload the modbrick kernel module
[group('robot')]
modbrick-reload:
    ssh robot@{{ ROBOT_IP }} "sudo modbrick-reload"
