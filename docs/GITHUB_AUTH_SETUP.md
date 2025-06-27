# GitHub OAuth App Setup for git-worktree-cli

This guide explains how to set up GitHub OAuth authentication for `gwt` to display pull request information.

## Why OAuth Device Flow?

The OAuth device flow provides:
- **Better security**: No need to store long-lived personal access tokens
- **Easier setup**: Users don't need to manually create and manage tokens
- **Better UX**: Simple browser-based authentication flow

## Setting Up the OAuth App

### For Development/Personal Use

1. Go to [GitHub Settings > Developer settings > OAuth Apps](https://github.com/settings/developers)
2. Click "New OAuth App"
3. Fill in the details:
   - **Application name**: `git-worktree-cli` (or your preferred name)
   - **Homepage URL**: `https://github.com/yourusername/git-worktree-cli`
   - **Authorization callback URL**: `http://localhost` (required but not used)
   - **Enable Device Flow**: ✅ **Check this box**
4. Click "Register application"
5. Copy the **Client ID** (you'll need this in the next step)

### Configuring the Client ID

Edit `src/github.rs` and replace `YOUR_CLIENT_ID_HERE` with your actual Client ID:

```rust
const CLIENT_ID: &str = "your_actual_client_id_here";
```

### For Production/Distribution

For a production release, you have several options:

1. **Environment Variable**: Instead of hardcoding, read from environment:
   ```rust
   const CLIENT_ID: &str = env!("GWT_GITHUB_CLIENT_ID");
   ```

2. **Build-time Configuration**: Use a build script to inject the ID

3. **Runtime Configuration**: Store in a config file

## Using GitHub Authentication

Once configured and built:

```bash
# Authenticate with GitHub
gwt auth github

# List worktrees with PR information
gwt list

# Logout (remove stored token)
gwt auth github --logout
```

## Security Notes

- The OAuth device flow doesn't require a client secret
- Access tokens are stored securely in `~/.config/gwt/auth.json` with 600 permissions
- Tokens can be revoked at any time from GitHub settings
- The app only requests `repo` scope for PR access

## Troubleshooting

- If authentication fails, check that "Enable Device Flow" is checked in your OAuth app settings
- Ensure your Client ID is correctly set in the source code
- Check that you have internet connectivity during authentication