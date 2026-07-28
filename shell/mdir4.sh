# Source this file from bash or zsh to make Mdir4 change the current shell directory on exit.
mdir4() {
    local cwd_file selected_dir exit_status
    cwd_file="$(mktemp "${TMPDIR:-/tmp}/mdir4-cwd.XXXXXX")" || return 1

    command mdir4 --cwd-file "$cwd_file" "$@"
    exit_status=$?
    selected_dir="$(command cat -- "$cwd_file")"
    command rm -f -- "$cwd_file"

    if [ "$exit_status" -eq 0 ] && [ -n "$selected_dir" ] && [ -d "$selected_dir" ]; then
        builtin cd -- "$selected_dir" || return $?
    fi
    return "$exit_status"
}
