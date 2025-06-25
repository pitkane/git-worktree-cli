# Setting up GWT Tab Completions

## For Zsh

Add the following to your `~/.zshrc`:

```bash
# GWT completions
if [[ -f ~/.git-worktree-scripts/completions/gwt-completions.zsh ]]; then
    fpath=(~/.git-worktree-scripts/completions $fpath)
    autoload -Uz compinit && compinit
    source ~/.git-worktree-scripts/completions/gwt-completions.zsh
fi
```

Or if you prefer to copy the completion file to a standard location:

```bash
# Copy to zsh completions directory
cp ~/.git-worktree-scripts/completions/gwt-completions.zsh /usr/local/share/zsh/site-functions/_gwt

# Then reload your shell
exec zsh
```

## For Bash

Generate bash completions:
```bash
gwt completions bash > ~/.git-worktree-scripts/completions/gwt-completions.bash
```

Add to your `~/.bashrc`:
```bash
# GWT completions
if [[ -f ~/.git-worktree-scripts/completions/gwt-completions.bash ]]; then
    source ~/.git-worktree-scripts/completions/gwt-completions.bash
fi
```

## Features

The completions provide:

- **Command completion**: Tab after `gwt` shows all available commands
- **Branch name completion**: 
  - `gwt add <TAB>` shows available remote branches
  - `gwt remove <TAB>` shows existing worktrees that can be removed
- **URL completion**: `gwt init <TAB>` provides URL completion
- **Shell completion**: `gwt completions <TAB>` shows available shells (bash, zsh, fish)

## Testing

After setting up, test with:
```bash
gwt <TAB>              # Should show: init, add, list, remove, completions
gwt add <TAB>          # Should show available branches
gwt remove <TAB>       # Should show existing worktrees
```