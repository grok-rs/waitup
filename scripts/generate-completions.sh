#!/bin/bash
#
# Generate shell completion scripts for wait-for
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
COMPLETIONS_DIR="$PROJECT_DIR/completions"

# Create completions directory
mkdir -p "$COMPLETIONS_DIR"

# Build the project first
echo "Building wait-for..."
cd "$PROJECT_DIR"
cargo build --release

echo "Generating completion scripts..."

# Generate completions for different shells
"$PROJECT_DIR/target/release/wait-for" --generate-completion bash > "$COMPLETIONS_DIR/wait-for.bash"
"$PROJECT_DIR/target/release/wait-for" --generate-completion zsh > "$COMPLETIONS_DIR/_wait-for"
"$PROJECT_DIR/target/release/wait-for" --generate-completion fish > "$COMPLETIONS_DIR/wait-for.fish"
"$PROJECT_DIR/target/release/wait-for" --generate-completion powershell > "$COMPLETIONS_DIR/wait-for.ps1"

echo "Completion scripts generated in $COMPLETIONS_DIR:"
ls -la "$COMPLETIONS_DIR"

echo ""
echo "Installation instructions:"
echo ""
echo "Bash:"
echo "  sudo cp $COMPLETIONS_DIR/wait-for.bash /etc/bash_completion.d/"
echo "  # OR for user-only:"
echo "  mkdir -p ~/.local/share/bash-completion/completions"
echo "  cp $COMPLETIONS_DIR/wait-for.bash ~/.local/share/bash-completion/completions/wait-for"
echo ""
echo "Zsh:"
echo "  # Add to your .zshrc:"
echo "  fpath=(~/.local/share/zsh/completions \$fpath)"
echo "  # Then:"
echo "  mkdir -p ~/.local/share/zsh/completions"
echo "  cp $COMPLETIONS_DIR/_wait-for ~/.local/share/zsh/completions/"
echo ""
echo "Fish:"
echo "  mkdir -p ~/.config/fish/completions"
echo "  cp $COMPLETIONS_DIR/wait-for.fish ~/.config/fish/completions/"
echo ""
echo "PowerShell:"
echo "  # Add to your PowerShell profile:"
echo "  . $COMPLETIONS_DIR/wait-for.ps1"
