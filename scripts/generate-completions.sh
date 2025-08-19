#!/bin/bash
#
# Generate shell completion scripts for waitup
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
COMPLETIONS_DIR="$PROJECT_DIR/completions"

# Create completions directory
mkdir -p "$COMPLETIONS_DIR"

# Build the project first
echo "Building waitup..."
cd "$PROJECT_DIR"
cargo build --release

echo "Generating completion scripts..."

# Generate completions for different shells
"$PROJECT_DIR/target/release/waitup" --generate-completion bash > "$COMPLETIONS_DIR/waitup.bash"
"$PROJECT_DIR/target/release/waitup" --generate-completion zsh > "$COMPLETIONS_DIR/_waitup"
"$PROJECT_DIR/target/release/waitup" --generate-completion fish > "$COMPLETIONS_DIR/waitup.fish"
"$PROJECT_DIR/target/release/waitup" --generate-completion powershell > "$COMPLETIONS_DIR/waitup.ps1"

echo "Completion scripts generated in $COMPLETIONS_DIR:"
ls -la "$COMPLETIONS_DIR"

echo ""
echo "Installation instructions:"
echo ""
echo "Bash:"
echo "  sudo cp $COMPLETIONS_DIR/waitup.bash /etc/bash_completion.d/"
echo "  # OR for user-only:"
echo "  mkdir -p ~/.local/share/bash-completion/completions"
echo "  cp $COMPLETIONS_DIR/waitup.bash ~/.local/share/bash-completion/completions/waitup"
echo ""
echo "Zsh:"
echo "  # Add to your .zshrc:"
echo "  fpath=(~/.local/share/zsh/completions \$fpath)"
echo "  # Then:"
echo "  mkdir -p ~/.local/share/zsh/completions"
echo "  cp $COMPLETIONS_DIR/_waitup ~/.local/share/zsh/completions/"
echo ""
echo "Fish:"
echo "  mkdir -p ~/.config/fish/completions"
echo "  cp $COMPLETIONS_DIR/waitup.fish ~/.config/fish/completions/"
echo ""
echo "PowerShell:"
echo "  # Add to your PowerShell profile:"
echo "  . $COMPLETIONS_DIR/waitup.ps1"
