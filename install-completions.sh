#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2025 RAprogramm <andrey.rozanov.vl@gmail.com>
# SPDX-License-Identifier: MIT

set -e

SHELL_NAME="${1:-}"

if [ -z "$SHELL_NAME" ]; then
    echo "Usage: $0 <shell>"
    echo "Supported shells: bash, fish, zsh, powershell, elvish"
    exit 1
fi

case "$SHELL_NAME" in
    bash)
        COMP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/bash-completion/completions"
        mkdir -p "$COMP_DIR"
        cargo quality completions bash > "$COMP_DIR/cargo-quality"
        echo "✓ Bash completions installed to: $COMP_DIR/cargo-quality"
        echo "  Restart your shell or run: source $COMP_DIR/cargo-quality"
        ;;
    fish)
        COMP_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/fish/completions"
        mkdir -p "$COMP_DIR"

        # Generate cargo quality subcommand completions
        cat > "$COMP_DIR/cargo.fish" << 'EOF'
# Completion for cargo quality subcommand
complete -c cargo -n "__fish_seen_subcommand_from quality" -s h -l help -d 'Print help'
complete -c cargo -n "__fish_seen_subcommand_from quality" -f -a "check" -d 'Check code quality'
complete -c cargo -n "__fish_seen_subcommand_from quality" -f -a "fix" -d 'Fix quality issues'
complete -c cargo -n "__fish_seen_subcommand_from quality" -f -a "format" -d 'Format code'
complete -c cargo -n "__fish_seen_subcommand_from quality" -f -a "fmt" -d 'Run cargo +nightly fmt'
complete -c cargo -n "__fish_seen_subcommand_from quality" -f -a "diff" -d 'Show proposed changes'
complete -c cargo -n "__fish_seen_subcommand_from quality" -f -a "help" -d 'Display help'
complete -c cargo -n "__fish_seen_subcommand_from quality" -f -a "completions" -d 'Generate completions'

# Diff options
complete -c cargo -n "__fish_seen_subcommand_from quality; and __fish_seen_subcommand_from diff" -s s -l summary -d 'Brief summary'
complete -c cargo -n "__fish_seen_subcommand_from quality; and __fish_seen_subcommand_from diff" -s i -l interactive -d 'Interactive mode'

# Check options
complete -c cargo -n "__fish_seen_subcommand_from quality; and __fish_seen_subcommand_from check" -s v -l verbose -d 'Detailed output'

# Fix options
complete -c cargo -n "__fish_seen_subcommand_from quality; and __fish_seen_subcommand_from fix" -s d -l dry-run -d 'Dry run'

# Completions options
complete -c cargo -n "__fish_seen_subcommand_from quality; and __fish_seen_subcommand_from completions" -f -a "bash fish zsh powershell elvish"
EOF

        echo "✓ Fish completions installed to: $COMP_DIR/cargo.fish"
        echo "  Completions will be available in new fish sessions"
        ;;
    zsh)
        COMP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/zsh/site-functions"
        mkdir -p "$COMP_DIR"
        cargo quality completions zsh > "$COMP_DIR/_cargo-quality"
        echo "✓ Zsh completions installed to: $COMP_DIR/_cargo-quality"
        echo "  Restart your shell or run: autoload -U compinit && compinit"
        ;;
    powershell)
        if command -v pwsh &> /dev/null; then
            PROFILE_DIR=$(pwsh -NoProfile -Command 'Split-Path $PROFILE')
            mkdir -p "$PROFILE_DIR"
            cargo quality completions powershell > "$PROFILE_DIR/cargo-quality.ps1"
            echo "✓ PowerShell completions installed to: $PROFILE_DIR/cargo-quality.ps1"
            echo "  Add this to your \$PROFILE:"
            echo "    . $PROFILE_DIR/cargo-quality.ps1"
        else
            echo "❌ PowerShell (pwsh) not found"
            exit 1
        fi
        ;;
    elvish)
        COMP_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/elvish/lib"
        mkdir -p "$COMP_DIR"
        cargo quality completions elvish > "$COMP_DIR/cargo-quality.elv"
        echo "✓ Elvish completions installed to: $COMP_DIR/cargo-quality.elv"
        echo "  Add this to your rc.elv:"
        echo "    use cargo-quality"
        ;;
    *)
        echo "❌ Unsupported shell: $SHELL_NAME"
        echo "Supported: bash, fish, zsh, powershell, elvish"
        exit 1
        ;;
esac
