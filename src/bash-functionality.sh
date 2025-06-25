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
	
	local result
	result=$("$HOME/.git-worktree-scripts/node_modules/.bin/tsx" "$HOME/.git-worktree-scripts/src/git-worktree-add.ts" "$1")
	local exit_code=$?
	
	# Print the output from the script
	echo "$result"
	
	if [ $exit_code -eq 0 ]; then
		# Extract the target path from the output
		local target_path
		target_path=$(echo "$result" | grep "✓ Worktree created at:" | sed 's/✓ Worktree created at: //')
		
		if [ -n "$target_path" ] && [ -d "$target_path" ]; then
			echo ""
			echo "Changing to worktree directory..."
			cd "$target_path"
			echo "✓ Now in: $(pwd)"
		fi
	fi
	
	return $exit_code
}

function gwtswitch() {
	local result
	result=$("$HOME/.git-worktree-scripts/node_modules/.bin/tsx" "$HOME/.git-worktree-scripts/src/git-worktree-switch.ts" "$@")
	local exit_code=$?
	
	# Print the output from the script
	echo "$result"
	
	if [ $exit_code -eq 0 ]; then
		# Extract the target path from the output
		local target_path
		target_path=$(echo "$result" | grep "Switching to worktree:" | sed 's/Switching to worktree: //')
		
		if [ -n "$target_path" ] && [ -d "$target_path" ]; then
			cd "$target_path"
		fi
	fi
	
	return $exit_code
}

function gwtremove() {
	# Store original directory for fallback
	local original_dir="$(pwd)"
	
	# Run the TypeScript script directly for interactive prompts
	"$HOME/.git-worktree-scripts/node_modules/.bin/tsx" "$HOME/.git-worktree-scripts/src/git-worktree-remove.ts" "$1"
	local exit_code=$?
	
	if [ $exit_code -eq 0 ]; then
		# Check if we're removing current worktree (no parameter given)
		if [ -z "$1" ]; then
			# Look for project root directory with config file
			local project_root=""
			local current_dir="$(pwd)"
			
			# First check if we're already in project root
			if [ -f "git-worktree-config.yaml" ]; then
				project_root="$current_dir"
			else
				# Search up the directory tree for the config file
				local check_dir="$current_dir"
				while [ "$check_dir" != "/" ]; do
					check_dir="$(dirname "$check_dir")"
					if [ -f "$check_dir/git-worktree-config.yaml" ]; then
						project_root="$check_dir"
						break
					fi
				done
			fi
			
			# Navigate to project root if found
			if [ -n "$project_root" ] && [ -d "$project_root" ]; then
				echo ""
				echo "Changing to project root..."
				cd "$project_root"
				echo "✓ Now in: $(pwd)"
			fi
		fi
	fi
	
	return $exit_code
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
