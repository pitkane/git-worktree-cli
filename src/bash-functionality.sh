#!/bin/zsh

function gwtinit() {
	if [ -z "$1" ]; then
		echo "Error: Repository URL is required"
		echo "Usage: gwtinit <repository-url>"
		return 1
	fi
	
	"$HOME/.git-worktree-scripts/node_modules/.bin/tsx" "$HOME/.git-worktree-scripts/src/git-worktree-init.ts" "$1"
}

function gwtlist() {
	"$HOME/.git-worktree-scripts/node_modules/.bin/tsx" "$HOME/.git-worktree-scripts/src/git-worktree-list.ts" "$@"
}

function gwtadd() {
	if [ -z "$1" ]; then
		echo "Error: Folder name is required"
		echo "Usage: gwtadd <folder-name>"
		return 1
	fi
	
	"$HOME/.git-worktree-scripts/node_modules/.bin/tsx" "$HOME/.git-worktree-scripts/src/git-worktree-add.ts" "$1"
}

function gwtswitch() {
	"$HOME/.git-worktree-scripts/node_modules/.bin/tsx" "$HOME/.git-worktree-scripts/src/git-worktree-switch.ts" "$1"
}

function gwtremove() {
	"$HOME/.git-worktree-scripts/node_modules/.bin/tsx" "$HOME/.git-worktree-scripts/src/git-worktree-remove.ts" "$1"
}

# Tab completion for gwtswitch
function _gwtswitch_completion() {
	local current_word="${COMP_WORDS[COMP_CWORD]}"
	local completions
	
	# Get completions from our TypeScript script
	completions=$("$HOME/.git-worktree-scripts/node_modules/.bin/tsx" "$HOME/.git-worktree-scripts/src/git-worktree-completion.ts" "gwtswitch" "$current_word" 2>/dev/null)
	
	if [[ -n "$completions" ]]; then
		# Check if it's a help message (starts with #)
		if [[ "$completions" =~ ^# ]]; then
			# Show help message but don't complete
			echo >&2
			echo "$completions" | sed 's/^# //' >&2
			COMPREPLY=()
		else
			# Normal completion
			COMPREPLY=($(compgen -W "$completions" -- "$current_word"))
		fi
	else
		COMPREPLY=()
	fi
}

# Register completion for gwtswitch
complete -F _gwtswitch_completion gwtswitch

# Tab completion for gwtadd (placeholder for future implementation)
function _gwtadd_completion() {
	local current_word="${COMP_WORDS[COMP_CWORD]}"
	local completions
	
	# Get completions from our TypeScript script
	completions=$("$HOME/.git-worktree-scripts/node_modules/.bin/tsx" "$HOME/.git-worktree-scripts/src/git-worktree-completion.ts" "gwtadd" "$current_word" 2>/dev/null)
	
	if [[ -n "$completions" ]]; then
		if [[ "$completions" =~ ^# ]]; then
			echo >&2
			echo "$completions" | sed 's/^# //' >&2
			COMPREPLY=()
		else
			COMPREPLY=($(compgen -W "$completions" -- "$current_word"))
		fi
	else
		COMPREPLY=()
	fi
}

# Register completion for gwtadd
complete -F _gwtadd_completion gwtadd

# Tab completion for gwtremove
function _gwtremove_completion() {
	local current_word="${COMP_WORDS[COMP_CWORD]}"
	local completions
	
	# Get completions from our TypeScript script
	completions=$("$HOME/.git-worktree-scripts/node_modules/.bin/tsx" "$HOME/.git-worktree-scripts/src/git-worktree-completion.ts" "gwtremove" "$current_word" 2>/dev/null)
	
	if [[ -n "$completions" ]]; then
		if [[ "$completions" =~ ^# ]]; then
			echo >&2
			echo "$completions" | sed 's/^# //' >&2
			COMPREPLY=()
		else
			COMPREPLY=($(compgen -W "$completions" -- "$current_word"))
		fi
	else
		COMPREPLY=()
	fi
}

# Register completion for gwtremove
complete -F _gwtremove_completion gwtremove
