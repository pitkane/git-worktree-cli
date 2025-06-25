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
