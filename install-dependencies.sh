#!/usr/bin/env bash
# Install dependencies needed for KabutOS development

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

    if ! command -v apt > /dev/null; then
        echo "This command expects Debian or Ubuntu" >&2
        return 1
    fi

    local packages=" gcc-riscv64-unknown-elf "

    # CI doesn't need quite as much
    if [[ -z "${ci}" ]]; then
        packages+=" vim tmux git "
        packages+=" binutils-riscv64-unknown-elf "
        packages+=" qemu-system-riscv64 "
        packages+=" gdb-multiarch "
    fi

    # Install RustUp if needed
    if ! command -v rustup > /dev/null; then
        curl --proto '=https' --tlsv1.3 -sSf https://sh.rustup.rs | sh
    fi

    # Install Rust Targets
    rustup target add riscv64imac-unknown-none-elf

    # shellcheck disable=SC2086
    sudo apt install ${packages}
}

main "${@}"
