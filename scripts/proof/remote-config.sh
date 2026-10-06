#!/bin/sh
# Sourced by the isolated proof helpers, before any worker command is sent.
lay_proof_validate_host() {
    case "$1" in
        ''|[!A-Za-z0-9_]*|*[!A-Za-z0-9_.@-]*)
            echo "LAY_PROOF_REMOTE must be a plain SSH host or alias" >&2
            exit 64
            ;;
    esac
}

lay_proof_validate_path() {
    case "$2" in
        ''|/|*[!A-Za-z0-9_./-]*|*/../*|*/..|*/./*|*/.)
            echo "$1 must be an absolute path without shell syntax or traversal" >&2
            exit 64
            ;;
    esac
    case "$2" in
        /*) ;;
        *) echo "$1 must be an absolute path" >&2; exit 64 ;;
    esac
}
