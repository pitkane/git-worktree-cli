#compdef gwt

# Get list of git branches for completion
_gwt_branches() {
    local branches
    if command -v gwt >/dev/null 2>&1; then
        # Get branches from worktrees
        branches=(${(f)"$(gwt list 2>/dev/null | tail -n +4 | awk '{print $1}')"})
        
        # Also get remote branches that don't have worktrees yet
        if [[ -d .git ]] || git rev-parse --git-dir > /dev/null 2>&1; then
            local remote_branches=(${(f)"$(git branch -r 2>/dev/null | grep -v HEAD | sed 's/.*origin\///' | grep -v -E '^\s*$')"})
            branches+=($remote_branches)
        fi
        
        # Remove duplicates and sort
        branches=(${(u)branches})
        
        _describe 'branch' branches
    fi
}

# Get list of removable worktrees
_gwt_removable_worktrees() {
    local worktrees
    if command -v gwt >/dev/null 2>&1; then
        worktrees=(${(f)"$(gwt list 2>/dev/null | tail -n +4 | awk '{print $1}')"})
        _describe 'worktree' worktrees
    fi
}

_gwt() {
    local -a _arguments_options
    local ret=1

    _arguments_options=(-s -S -C)

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
        '(-h --help)'{-h,--help}'[Print help information]' \
        '(-V --version)'{-V,--version}'[Print version information]' \
        '1: :_gwt_commands' \
        '*::arg:->args' \
        && ret=0

    case $state in
        args)
            case $words[1] in
                init)
                    _arguments "${_arguments_options[@]}" : \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1:repository url:_urls' \
                        && ret=0
                    ;;
                add)
                    _arguments "${_arguments_options[@]}" : \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1:branch name:_gwt_branches' \
                        && ret=0
                    ;;
                list)
                    _arguments "${_arguments_options[@]}" : \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        && ret=0
                    ;;
                remove)
                    _arguments "${_arguments_options[@]}" : \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '::branch name:_gwt_removable_worktrees' \
                        && ret=0
                    ;;
                completions)
                    _arguments "${_arguments_options[@]}" : \
                        '(-h --help)'{-h,--help}'[Print help]' \
                        '1:shell:(bash zsh fish)' \
                        && ret=0
                    ;;
                *)
                    _message "unknown gwt command: $words[1]"
                    ;;
            esac
            ;;
    esac

    return ret
}

_gwt_commands() {
    local commands=(
        'init:Initialize a new worktree project from a repository URL'
        'add:Add a new worktree for a branch'
        'list:List all worktrees in the current project'
        'remove:Remove a worktree'
        'completions:Generate shell completions'
    )
    _describe 'gwt command' commands
}

# Don't run the completion function when being source'd or eval'd
if [ "$funcstack[1]" = "_gwt" ]; then
    _gwt "$@"
fi