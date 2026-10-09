# Integration Guides

## Git Pre-commit Hooks

Automatically validate code before every commit.

### Installation

Create the hook file:
```bash
mkdir -p .git/hooks
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/sh
# SlugRails pre-commit hook
# Validates staged changes against architectural rules

# Skip if no SlugRails binary found
if ! command -v slugrails >/dev/null 2>&1; then
    echo "SlugRails not found. Skipping architectural validation."
    exit 0
fi

# Run diff-based check (fast)
if ! slugrails check --diff; then
    echo ""
    echo "Architectural violations detected!"
    echo "Fix issues or add approved exceptions before committing."
    echo ""
    exit 1
fi
EOF

chmod +x .git/hooks/pre-commit
```

### Usage

- **Normal workflow**: `git add . && git commit` — automatically validates
- **Skip validation**: `git commit --no-verify` (use sparingly)
- **Manual run**: `.git/hooks/pre-commit`

### Best Practices

- Use `--diff` for speed (<5ms typical)
- Keep hooks idempotent and fast
- Provide clear error messages for developers

## AI Agent Integration

Connect SlugRails to your AI coding assistant.

### Hermes Agent

**Register the server**:
```bash
hermes mcp add slugrails -- ~/.slugthug/bin/slugrails serve
```

**Verify registration**:
```bash
hermes mcp list
```

**Remove if needed**:
```bash
hermes mcp remove slugrails
```

### Claude Code

**Register the server**:
```bash
claude mcp add slugrails -- ~/.slugthug/bin/slugrails serve
```

### Grok / Codex

Use the same pattern:
```bash
grok mcp add slugrails -- ~/.slugthug/bin/slugrails serve
codex mcp add slugrails -- ~/.slugthug/bin/slugrails serve
```

### Other MCP Clients (OpenCode, Cursor, Zed)

Add this configuration to your client's `.mcp.json`:

```json
{
  "mcpServers": {
    "slugrails": {
      "command": "~/.slugthug/bin/slugrails",
      "args": ["serve"]
    }
  }
}
```

## CI/CD Pipeline Integration

### GitHub Actions

```yaml
name: Architectural Governance
on: [push, pull_request]

jobs:
  slugrails:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install SlugRails
        run: |
          git clone https://github.com/SlugThugLabs/slugrails.git
          cd slugrails
          cargo build --release
          mkdir -p ~/.slugthug/bin
          cp target/release/slugrails ~/.slugthug/bin/
          echo "$HOME/.slugthug/bin" >> $GITHUB_PATH
      
      - name: Run Full Validation
        run: slugrails check --all
```

### GitLab CI

```yaml
slugrails:
  image: rust:latest
  script:
    - git clone https://github.com/SlugThugLabs/slugrails.git
    - cd slugrails && cargo build --release
    - cp target/release/slugrails ~/.slugthug/bin/
    - export PATH="$HOME/.slugthug/bin:$PATH"
    - slugrails check --all
```

### Jenkins

```groovy
pipeline {
    agent any
    stages {
        stage('SlugRails') {
            steps {
                sh '''
                    if [ ! -f ~/.slugthug/bin/slugrails ]; then
                        git clone https://github.com/SlugThugLabs/slugrails.git
                        cd slugrails
                        cargo build --release
                        mkdir -p ~/.slugthug/bin
                        cp target/release/slugrails ~/.slugthug/bin/
                    fi
                    ~/.slugthug/bin/slugrails check --all
                '''
            }
        }
    }
}
```

## IDE Integration

### VS Code

1. Install the **MCP Client** extension
2. Add to `settings.json`:
```json
{
  "mcp.servers": {
    "slugrails": {
      "command": "~/.slugthug/bin/slugrails",
      "args": ["serve"]
    }
  }
}
```

### Vim/Neovim

Use with **mcp.nvim**:
```lua
require('mcp').setup({
  servers = {
    slugrails = {
      command = '~/.slugthug/bin/slugrails',
      args = {'serve'}
    }
  }
})
```

## Advanced Workflows

### Monorepo Support

For projects with multiple packages:

```bash
# Validate specific package
slugrails validate --root ./packages/frontend

# Check all packages
for pkg in packages/*; do
  echo "Validating $pkg..."
  slugrails check --all --root "$pkg" || exit 1
done
```

### Docker Integration

**Dockerfile**:
```dockerfile
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/slugrails /usr/local/bin/
ENTRYPOINT ["slugrails"]
```

**Usage**:
```bash
docker build -t slugrails .
docker run -v $(pwd):/project slugrails check --all --root /project
```

### Remote Development

For SSH/remote development environments:

1. Install SlugRails on the remote machine
2. Configure your local AI agent to use remote MCP:
   ```bash
   # Local machine
   hermes mcp add slugrails -- ssh user@remote-host "~/.slugthug/bin/slugrails serve"
   ```

## Migration from Other Tools

### From traditional linters

- **ESLint/TSLint**: Replace language-specific rules with SlugRails' multi-language guards
- **RuboCop**: Use custom guards for Ruby-specific patterns  
- **Checkstyle**: Migrate Java rules to custom guard patterns

### From architecture enforcement tools

- **ArchUnit**: Map layer dependency rules to `structural/layer-dependencies`
- **Depcruiser**: Convert dependency graphs to `allowed_dependencies` in ARCHITECTURE.md
- **SonarQube**: Replace quality gates with SlugRails' built-in hygiene/quality guards

## Performance Optimization

### Large Repository Tuning

For repositories with 1000+ files:

1. **Use incremental checks**: Always prefer `--diff` over `--all` in pre-commit
2. **Exclude generated code**: Add patterns to `.gitignore`
3. **Parallel validation**: Split monorepos into separate validation jobs
4. **Cache results**: Store exception tokens and catalog between runs

### Memory-Constrained Environments

For containers with limited memory:

```bash
# Limit parallelism
RAYON_NUM_THREADS=2 slugrails check --all

# Monitor memory usage
/usr/bin/time -v slugrails check --all
```

These integrations ensure SlugRails fits seamlessly into any development workflow while maintaining architectural integrity.