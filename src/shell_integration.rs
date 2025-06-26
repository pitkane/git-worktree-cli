use clap_complete::Shell;

/// Generate shell wrapper functions for gwt add auto-navigation
pub fn generate_shell_integration(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash => BASH_INTEGRATION,
        Shell::Zsh => ZSH_INTEGRATION,
        Shell::Fish => FISH_INTEGRATION,
        Shell::PowerShell => POWERSHELL_INTEGRATION,
        Shell::Elvish => ELVISH_INTEGRATION,
        _ => panic!("Unsupported shell for integration: {:?}", shell),
    }
}

const BASH_INTEGRATION: &str = r#"
# gwt shell integration for Bash
# Add this to your ~/.bashrc

gwt_add() {
    if [[ "$*" == *"--print-path"* ]]; then
        # Pass through if --print-path is explicitly used
        command gwt add "$@"
    else
        # Use --print-path internally and cd to result
        local result=$(command gwt add --print-path "$@" 2>/dev/null)
        local exit_code=$?
        
        if [ $exit_code -eq 0 ] && [ -n "$result" ] && [ -d "$result" ]; then
            cd -- "$result" || return 1
            echo "✓ Switched to worktree: $(basename "$result")"
            
            # Execute hooks manually since we skipped them in --print-path mode
            if command -v gwt >/dev/null 2>&1; then
                # Run hooks by calling gwt add normally but redirecting output
                command gwt add "$(basename "$result")" >/dev/null 2>&1 || true
            fi
        else
            # If failed, run normal gwt add to show error messages
            command gwt add "$@"
            return $?
        fi
    fi
}

# Optional: alias gwt add to gwt_add for seamless integration
# Uncomment the next line if you want gwt add to auto-navigate by default
# alias gwt='gwt_wrapper'
# gwt_wrapper() { if [ "$1" = "add" ]; then shift; gwt_add "$@"; else command gwt "$@"; fi; }
"#;

const ZSH_INTEGRATION: &str = r#"
# gwt shell integration for Zsh
# Add this to your ~/.zshrc

gwt_add() {
    if [[ "$*" == *"--print-path"* ]]; then
        # Pass through if --print-path is explicitly used
        command gwt add "$@"
    else
        # Use --print-path internally and cd to result
        local result=$(command gwt add --print-path "$@" 2>/dev/null)
        local exit_code=$?
        
        if [[ $exit_code -eq 0 && -n "$result" && -d "$result" ]]; then
            cd -- "$result" || return 1
            echo "✓ Switched to worktree: $(basename "$result")"
            
            # Execute hooks manually since we skipped them in --print-path mode
            if command -v gwt >/dev/null 2>&1; then
                # Run hooks by calling gwt add normally but redirecting output
                command gwt add "$(basename "$result")" >/dev/null 2>&1 || true
            fi
        else
            # If failed, run normal gwt add to show error messages
            command gwt add "$@"
            return $?
        fi
    fi
}

# Optional: alias gwt add to gwt_add for seamless integration
# Uncomment the next line if you want gwt add to auto-navigate by default
# alias gwt='gwt_wrapper'
# gwt_wrapper() { if [[ "$1" == "add" ]]; then shift; gwt_add "$@"; else command gwt "$@"; fi; }
"#;

const FISH_INTEGRATION: &str = r#"
# gwt shell integration for Fish
# Add this to your ~/.config/fish/config.fish

function gwt_add
    # Check if --print-path is in arguments
    if string match -q '*--print-path*' -- $argv
        # Pass through if --print-path is explicitly used
        command gwt add $argv
    else
        # Use --print-path internally and cd to result
        set result (command gwt add --print-path $argv 2>/dev/null)
        set exit_code $status
        
        if test $exit_code -eq 0 -a -n "$result" -a -d "$result"
            cd -- "$result"; or return 1
            echo "✓ Switched to worktree: "(basename "$result")
            
            # Execute hooks manually since we skipped them in --print-path mode
            if command -v gwt >/dev/null 2>&1
                # Run hooks by calling gwt add normally but redirecting output
                command gwt add (basename "$result") >/dev/null 2>&1; or true
            end
        else
            # If failed, run normal gwt add to show error messages
            command gwt add $argv
            return $status
        end
    end
end

# Optional: alias gwt add to gwt_add for seamless integration
# Uncomment the next lines if you want gwt add to auto-navigate by default
# function gwt
#     if test "$argv[1]" = "add"
#         set -e argv[1]
#         gwt_add $argv
#     else
#         command gwt $argv
#     end
# end
"#;

const POWERSHELL_INTEGRATION: &str = r#"
# gwt shell integration for PowerShell
# Add this to your PowerShell profile

function gwt_add {
    param([Parameter(ValueFromRemainingArguments)]$Args)
    
    # Check if --print-path is in arguments
    if ($Args -join ' ' -like '*--print-path*') {
        # Pass through if --print-path is explicitly used
        & gwt add @Args
    } else {
        # Use --print-path internally and cd to result
        $result = & gwt add --print-path @Args 2>$null
        
        if ($LASTEXITCODE -eq 0 -and $result -and (Test-Path $result)) {
            Set-Location $result
            Write-Host "✓ Switched to worktree: $(Split-Path $result -Leaf)" -ForegroundColor Green
            
            # Execute hooks manually since we skipped them in --print-path mode
            if (Get-Command gwt -ErrorAction SilentlyContinue) {
                # Run hooks by calling gwt add normally but redirecting output
                & gwt add $(Split-Path $result -Leaf) >$null 2>&1
            }
        } else {
            # If failed, run normal gwt add to show error messages
            & gwt add @Args
        }
    }
}

# Optional: alias gwt add to gwt_add for seamless integration
# Uncomment the next lines if you want gwt add to auto-navigate by default
# function gwt {
#     param([Parameter(ValueFromRemainingArguments)]$Args)
#     if ($Args[0] -eq "add") {
#         gwt_add @Args[1..($Args.Length-1)]
#     } else {
#         & gwt @Args
#     }
# }
"#;

const ELVISH_INTEGRATION: &str = r#"
# gwt shell integration for Elvish
# Add this to your ~/.elvish/rc.elv

fn gwt_add {|@args|
    # Check if --print-path is in arguments
    if (echo $@args | grep -q -- --print-path) {
        # Pass through if --print-path is explicitly used
        gwt add $@args
    } else {
        # Use --print-path internally and cd to result
        var result exit-code = (gwt add --print-path $@args 2>/dev/null | slurp)
        
        if (and (== $exit-code 0) (not-eq $result "") (path:is-dir $result)) {
            cd $result
            echo "✓ Switched to worktree: "(path:base $result)
            
            # Execute hooks manually since we skipped them in --print-path mode
            try {
                gwt add (path:base $result) >/dev/null 2>&1
            } catch e {
                # Ignore hook execution errors
            }
        } else {
            # If failed, run normal gwt add to show error messages
            gwt add $@args
        }
    }
}

# Optional: alias gwt add to gwt_add for seamless integration
# Uncomment the next lines if you want gwt add to auto-navigate by default
# fn gwt {|@args|
#     if (eq $args[0] add) {
#         gwt_add $@args[1:]
#     } else {
#         command gwt $@args
#     }
# }
"#;