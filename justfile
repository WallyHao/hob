# --- justfile ---
# Task entry point. CI and a developer enter through the same recipes, so a
# command is never copied into YAML.

set shell := ["bash", "-euo", "pipefail", "-c"]

# The recipes below, in execution order.
default:
    @just --list

# --- gates ---

# Run every gate, in the order listed.
check: lint typecheck size test budget

# rustfmt and clippy over every target.
lint:
    cargo fmt --all --check
    cargo clippy --all-targets --all-features -- -D warnings

# Rewrite files: format, then apply the clippy suggestions that are safe.
format:
    cargo fmt --all
    cargo clippy --fix --allow-dirty --all-targets --all-features

# Type-check without the lint pass.
typecheck:
    # Rust folds type checking into compilation, so this overlaps `lint` by
    # design: it is the cheaper gate when only types moved.
    cargo check --all-targets --all-features

# Enforce the 120-line source limit from the project conventions.
size:
    ./scripts/check_size.sh

# The test suite.
test:
    # Bench targets are excluded on purpose: they are iai-callgrind harnesses
    # and only run under `just bench`, where the runner drives valgrind.
    cargo test --all-features

# The lightweight gates: release binary size, peak heap and dependency policy.
budget:
    ./scripts/check_binary_size.sh
    ./scripts/check_heap.sh
    ./scripts/check_deps.sh

# Instruction-count benchmarks; deterministic under valgrind.
bench:
    # `just env` installs the matching iai-callgrind-runner. Record a baseline
    # with `cargo bench --bench startup -- --save-baseline main`, then gate
    # against it with `--baseline main --regression-fail-fast`.
    #
    # The runner is installed under CARGO_HOME, which is not on every PATH;
    # point at it explicitly so `just env` plus `just bench` is enough.
    IAI_CALLGRIND_RUNNER="${IAI_CALLGRIND_RUNNER:-${CARGO_HOME:-$HOME/.cargo}/bin/iai-callgrind-runner}" \
        cargo bench --bench startup

# --- environment ---

# Check the toolchain and install whatever is missing.
env:
    #!/usr/bin/env bash
    set -euo pipefail

    have() { command -v "$1" >/dev/null 2>&1; }

    # `cargo install` drops binaries in $CARGO_HOME/bin, which is not on PATH on
    # every host; put it there for this run so the checks below see the tools
    # that were just installed.
    ensure_cargo_bin_on_path() {
        local bin="${CARGO_HOME:-$HOME/.cargo}/bin"
        [[ ":$PATH:" == *":$bin:"* ]] || export PATH="$bin:$PATH"
    }

    install_cargo_tool() {
        local command="$1" crate="$2"
        if have "$command"; then
            printf '%-16s %s\n' "$command" "$(command -v "$command")"
            return
        fi
        echo "installing $crate ..."
        cargo install --locked "$crate"
        ensure_cargo_bin_on_path
    }

    # --- Rust itself ---
    if ! have cargo || ! have rustc; then
        echo "installing rustup ..."
        # --no-modify-path: the installer must not edit shell profiles behind the
        # user's back; PATH is adjusted inside this process only.
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
            sh -s -- -y --no-modify-path --profile minimal
        if [[ -f "$HOME/.cargo/env" ]]; then
            # shellcheck disable=SC1091
            . "$HOME/.cargo/env"
        fi
        ensure_cargo_bin_on_path
        echo "note: open a new shell or source \$HOME/.cargo/env to pick up rustup"
    fi

    if have rustup; then
        # Honors rust-toolchain.toml once it exists; a no-op otherwise.
        rustup show active-toolchain >/dev/null 2>&1 || rustup toolchain install
        rustup component add rustfmt clippy llvm-tools-preview
    else
        # A system-provided toolchain (NixOS, distro packages) cannot receive
        # components; report what is there and what is not.
        printf '%-16s %s\n' rustc "$(rustc --version)"
        printf '%-16s %s\n' cargo "$(cargo --version)"
        if ! have rustfmt || ! have cargo-clippy; then
            echo "warning: rustfmt and/or clippy are missing from the system toolchain" >&2
        fi
    fi

    # --- Companion tools the gate recipes shell out to ---
    install_cargo_tool cargo-deny cargo-deny
    install_cargo_tool cargo-llvm-cov cargo-llvm-cov
    install_cargo_tool cargo-nextest cargo-nextest
    install_cargo_tool cog cocogitto
    install_cargo_tool typos typos-cli

    # The iai-callgrind runner must match the library version exactly, so
    # install the version pinned in Cargo.lock instead of the latest one.
    runner_version=$(awk -F'"' '/^name = "iai-callgrind-runner"$/{seen=1; next} seen && /^version/{print $2; exit}' Cargo.lock)
    if [ -n "$runner_version" ]; then
        installed=$(iai-callgrind-runner --version 2>/dev/null | awk '{print $NF}')
        if [ "$installed" = "$runner_version" ]; then
            printf '%-16s %s\n' iai-callgrind-runner "$(command -v iai-callgrind-runner)"
        else
            echo "installing iai-callgrind-runner $runner_version ..."
            cargo install --locked iai-callgrind-runner --version "=$runner_version"
            ensure_cargo_bin_on_path
        fi
    fi

    # pre-commit is not a cargo tool; uv is the fallback this project expects.
    if have pre-commit; then
        printf '%-16s %s\n' pre-commit "$(command -v pre-commit)"
    elif have uv; then
        echo "installing pre-commit with uv ..."
        uv tool install pre-commit
    else
        echo "warning: pre-commit is missing; install it with uv, pipx or brew" >&2
    fi

    # valgrind backs `just bench` and the heap budget.
    if have valgrind; then
        printf '%-16s %s\n' valgrind "$(valgrind --version)"
    else
        echo "warning: valgrind is missing; just bench and the heap budget need it" >&2
    fi

    echo "environment ready"
