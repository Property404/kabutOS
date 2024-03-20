#!/usr/bin/env bash
# Install dependencies needed for KabutOS development
# shellcheck disable=SC2086
set -e

main() {
    local -r USAGE="Usage: $(basename "${0}")"
    local -r HELP="Install KabutOS dependencies

$USAGE

Help:
    --ci        Install for CI instead of developer"

    local ci
    while true; do
        case "$1" in
            --ci) ci=1; shift ;;
            -h | --help ) echo "$HELP"; return 0 ;;
            -- ) shift; break ;;
            -* ) echo -e "Unrecognized option: $1\n$USAGE" >&2; return 1 ;;
            * ) break ;;
        esac
    done


    local packages=" git curl "

    # Ubuntu
    if command -v apt-get > /dev/null; then
        packages+=" qemu-system-riscv64 "
        packages+=" binutils-riscv64-unknown-elf "
        packages+=" gdb-multiarch "
        sudo apt-get install -y ${packages}

    # macOS
    elif command -v brew > /dev/null; then
        packages+=" coreutils qemu "
        brew install ${packages}

    # And fuck everyone else
    else
        echo "Unsupported OS" >&2
        return 1
    fi


    # Install RustUp if needed
    if ! command -v rustup > /dev/null; then
        curl --proto '=https' --tlsv1.3 -sSf https://sh.rustup.rs | sh
        export PATH="${PATH}:${HOME}/.cargo/bin"
    fi

    # Update
    rustup update stable

    # Install Rust Targets
    rustup target add riscv64gc-unknown-none-elf
}

main "${@}"
