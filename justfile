mod docs
mod vsc "sonance-vscode"

_default:
    just --list

clean:
    just docs clean
