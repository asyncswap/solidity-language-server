#!/usr/bin/env sh
# Install solidity-language-server from GitHub releases
# Usage: curl -fsSL https://asyncswap.org/lsp/install.sh | sh

set -e

main() {

REPO="asyncswap/solidity-language-server"
BINARY="solidity-language-server"
DOCS_URL="https://solidity-language-server-docs.pages.dev"

# Prompt before overwriting an existing file. Returns 0 if ok to write.
confirm_write() {
    if [ -f "$1" ]; then
        printf "%s already exists. Overwrite? [y/N] " "$1"
        read -r REPLY < /dev/tty
        case "$REPLY" in
            y|Y) return 0 ;;
            *)   echo "Skipped $1"; return 1 ;;
        esac
    fi
    return 0
}

# Prompt before appending to an existing file. Returns 0 if ok to append.
confirm_append() {
    if [ -f "$1" ] && grep -q "$2" "$1" 2>/dev/null; then
        echo "$2 already configured in $1"
        return 1
    fi
    if [ -f "$1" ]; then
        printf "%s exists. Append solidity config? [y/N] " "$1"
        read -r REPLY < /dev/tty
        case "$REPLY" in
            y|Y) return 0 ;;
            *)   echo "Skipped $1"; return 1 ;;
        esac
    fi
    return 0
}

# ── Editor setup functions ────────────────────────────────

setup_neovim() {
    NVIM_LSP_DIR="${HOME}/.config/nvim/lsp"
    NVIM_PLUGIN_DIR="${HOME}/.config/nvim/plugin"
    NVIM_LSP_FILE="${NVIM_LSP_DIR}/solidity-language-server.lua"
    NVIM_PLUGIN_FILE="${NVIM_PLUGIN_DIR}/lsp.lua"

    if confirm_write "$NVIM_LSP_FILE"; then
        mkdir -p "$NVIM_LSP_DIR"
        cat > "$NVIM_LSP_FILE" << 'NEOVIM_LSP'
return {
  name = "Solidity Language Server",
  cmd = { "solidity-language-server", "--stdio" },
  filetypes = { "solidity" },
  root_markers = { "foundry.toml", ".git" },
  capabilities = {
    textDocument = {
      semanticTokens = {
        multilineTokenSupport = true,
      },
    },
    workspace = {
      fileOperations = {
        willCreate = true,
        didCreate = true,
        willRename = true,
        didRename = true,
        willDelete = true,
        didDelete = true,
      },
    },
  },
  settings = {
    ["solidity-language-server"] = {
      inlayHints = {
        parameters = true,
        gasEstimates = true,
      },
      lint = {
        enabled = true,
        severity = {},
        only = {},
        exclude = {},
      },
      fileOperations = {
        templateOnCreate = true,
        updateImportsOnRename = true,
        updateImportsOnDelete = true,
      },
      projectIndex = {
        fullProjectScan = true,
        cacheMode = "v2",
        incrementalEditReindex = false,
      },
    },
  },
  on_attach = function(client, bufnr)
    vim.lsp.inlay_hint.enable(true, { bufnr = bufnr })

    vim.api.nvim_create_autocmd("BufWritePost", {
      callback = function()
        vim.lsp.buf.format()
      end,
    })

    vim.lsp.completion.enable(true, client.id, bufnr, {
      autotrigger = true,
      convert = function(item)
        return { abbr = item.label:gsub('%b()', '') }
      end,
    })

    for _, char in ipairs({ "(", ",", "[" }) do
      vim.keymap.set("i", char, function()
        vim.api.nvim_feedkeys(char, "n", false)
        vim.defer_fn(vim.lsp.buf.signature_help, 50)
      end, { buffer = bufnr })
    end
  end,
}
NEOVIM_LSP
        echo "Wrote ${NVIM_LSP_FILE}"
    fi

    # Write vim.lsp.enable to its own file
    NVIM_SOLIDITY_FILE="${NVIM_PLUGIN_DIR}/solidity.lua"
    if confirm_write "$NVIM_SOLIDITY_FILE"; then
        mkdir -p "$NVIM_PLUGIN_DIR"
        echo 'vim.lsp.enable("solidity-language-server")' > "$NVIM_SOLIDITY_FILE"
        echo "Wrote ${NVIM_SOLIDITY_FILE}"
    fi

    echo "Full setup guide: ${DOCS_URL}/setup/neovim"
}

setup_helix() {
    HELIX_DIR="${HOME}/.config/helix"
    HELIX_FILE="${HELIX_DIR}/languages.toml"

    if confirm_append "$HELIX_FILE" "solidity-language-server"; then
        mkdir -p "$HELIX_DIR"
        cat >> "$HELIX_FILE" << 'HELIX'

[language-server.solidity-language-server]
command = "solidity-language-server"
args = ["--stdio"]

[language-server.solidity-language-server.config.solidity-language-server.inlayHints]
parameters = true
gasEstimates = true

[language-server.solidity-language-server.config.solidity-language-server.lint]
enabled = true
severity = []
only = []
exclude = []

[language-server.solidity-language-server.config.solidity-language-server.fileOperations]
templateOnCreate = true
updateImportsOnRename = true
updateImportsOnDelete = true

[language-server.solidity-language-server.config.solidity-language-server.projectIndex]
fullProjectScan = true
cacheMode = "v2"
incrementalEditReindex = false

[[language]]
name = "solidity"
language-servers = ["solidity-language-server"]
auto-format = false
HELIX
        echo "Wrote solidity config to ${HELIX_FILE}"
    fi

    echo "Full setup guide: ${DOCS_URL}/setup/helix"
}

setup_vscode() {
    VSCODE_DIR=".vscode"
    PROXY_FILE="${VSCODE_DIR}/lsp-proxy.json"

    echo "Note: install the Generic LSP Proxy extension (mjmorales.generic-lsp-proxy)"

    if confirm_write "$PROXY_FILE"; then
        mkdir -p "$VSCODE_DIR"
        cat > "$PROXY_FILE" << 'VSCODE_PROXY'
[
  {
    "languageId": "solidity",
    "command": "solidity-language-server",
    "fileExtensions": [
      ".sol"
    ]
  }
]
VSCODE_PROXY
        echo "Wrote ${PROXY_FILE}"
    fi

    echo "Full setup guide: ${DOCS_URL}/setup/vscode"
}

setup_zed() {
    ZED_FILE="${HOME}/.config/zed/settings.json"

    if [ -f "$ZED_FILE" ] && grep -q 'solidity-language-server' "$ZED_FILE" 2>/dev/null; then
        echo "solidity-language-server already configured in ${ZED_FILE}"
    else
        echo "Add the following to ${ZED_FILE}:"
        echo ""
        cat << 'ZED'
```
"lsp": {
  "solidity": {
    "binary": {
      "path": "solidity-language-server",
      "arguments": ["--stdio"]
    },
    "initialization_options": {
      "inlayHints": {
        "parameters": true,
        "gasEstimates": true
      },
      "lint": {
        "enabled": true
      }
    }
  }
}
```
ZED
    fi

    echo ""
    echo "Full setup guide: ${DOCS_URL}/setup/zed"
}

setup_vim() {
    VIMRC="${HOME}/.vimrc"

    echo ""
    echo "Requires the following Vim plugins:"
    echo "  - prabirshrestha/vim-lsp"
    echo "  - prabirshrestha/asyncomplete.vim"
    echo "  - prabirshrestha/asyncomplete-lsp.vim"
    echo "  - prabirshrestha/async.vim"
    echo ""
    echo "Install with Vundle (add to .vimrc before 'call vundle#end()'):"
    echo ""
    echo "  Plugin 'prabirshrestha/async.vim'"
    echo "  Plugin 'prabirshrestha/vim-lsp'"
    echo "  Plugin 'prabirshrestha/asyncomplete.vim'"
    echo "  Plugin 'prabirshrestha/asyncomplete-lsp.vim'"
    echo ""

    if confirm_append "$VIMRC" "solidity-language-server"; then
        cat >> "$VIMRC" << 'VIMRC_CONTENT'

" Solidity language server (https://github.com/asyncswap/solidity-language-server)
if executable('solidity-language-server')
  au User lsp_setup call lsp#register_server({
      \ 'name': 'solidity-language-server',
      \ 'cmd': {server_info->['solidity-language-server', '--stdio']},
      \ 'root_uri': {server_info->lsp#utils#path_to_uri(
      \   empty(lsp#utils#find_nearest_parent_file_directory(
      \     lsp#utils#get_buffer_path(), ['foundry.toml', '.git']))
      \   ? getcwd()
      \   : lsp#utils#find_nearest_parent_file_directory(
      \     lsp#utils#get_buffer_path(), ['foundry.toml', '.git']))},
      \ 'whitelist': ['solidity'],
      \ })
endif

" LSP keybindings (applied per-buffer when LSP attaches)
function! s:on_lsp_buffer_enabled() abort
  setlocal omnifunc=lsp#complete
  setlocal signcolumn=yes
  nmap <buffer> gd <plug>(lsp-definition)
  nmap <buffer> gr <plug>(lsp-references)
  nmap <buffer> gi <plug>(lsp-implementation)
  nmap <buffer> gt <plug>(lsp-type-definition)
  nmap <buffer> K <plug>(lsp-hover)
  nmap <buffer> <leader>rn <plug>(lsp-rename)
  nmap <buffer> [d <plug>(lsp-previous-diagnostic)
  nmap <buffer> ]d <plug>(lsp-next-diagnostic)
  nmap <buffer> <leader>a <plug>(lsp-code-action)
endfunction

augroup lsp_install
  au!
  autocmd User lsp_buffer_enabled call s:on_lsp_buffer_enabled()
augroup END

let g:lsp_diagnostics_echo_cursor = 1
VIMRC_CONTENT
        echo "Added solidity-language-server config to ${VIMRC}"
    fi

    echo "Full setup guide: ${DOCS_URL}/setup/vim"
}

setup_emacs() {
    # Find init file
    if [ -f "${HOME}/.emacs.d/init.el" ]; then
        INIT_EL="${HOME}/.emacs.d/init.el"
    elif [ -f "${HOME}/.config/emacs/init.el" ]; then
        INIT_EL="${HOME}/.config/emacs/init.el"
    else
        INIT_EL="${HOME}/.emacs.d/init.el"
        mkdir -p "$(dirname "$INIT_EL")"
    fi

    if confirm_append "$INIT_EL" "solidity-language-server"; then
        cat >> "$INIT_EL" << 'EMACS'

(lsp-register-client
 (make-lsp-client
  :new-connection (lsp-stdio-connection '("solidity-language-server" "--stdio"))
  :major-modes '(solidity-mode)
  :server-id 'solidity-language-server
  :initialization-options
  '(:solidity-language-server
    (:inlayHints
     (:parameters t
      :gasEstimates t)
     :lint
     (:enabled t
      :severity []
      :only []
      :exclude [])
     :fileOperations
     (:templateOnCreate t
      :updateImportsOnRename t
      :updateImportsOnDelete t)
     :projectIndex
     (:fullProjectScan t
      :cacheMode "v2"
      :incrementalEditReindex :json-false)))))
EMACS
        echo "Added solidity-language-server config to ${INIT_EL}"
    fi

    echo "Full setup guide: ${DOCS_URL}/setup/emacs"
}

setup_sublime() {
    # macOS vs Linux settings path
    if [ "$(uname -s)" = "Darwin" ]; then
        SUBLIME_DIR="${HOME}/Library/Application Support/Sublime Text/Packages/User"
    else
        SUBLIME_DIR="${HOME}/.config/sublime-text/Packages/User"
    fi
    SUBLIME_FILE="${SUBLIME_DIR}/LSP.sublime-settings"

    if [ -f "$SUBLIME_FILE" ] && grep -q 'solidity-language-server' "$SUBLIME_FILE" 2>/dev/null; then
        echo "solidity-language-server already configured in ${SUBLIME_FILE}"
    else
        echo "Add the following to ${SUBLIME_FILE}:"
        echo ""
        cat << 'SUBLIME'
```
{
  "clients": {
    "solidity-language-server": {
      "command": ["solidity-language-server", "--stdio"],
      "selector": "source.solidity",
      "settings": {
        "solidity-language-server": {
          "inlayHints": {
            "parameters": true,
            "gasEstimates": true
          },
          "lint": {
            "enabled": true
          }
        }
      }
    }
  }
}
```
SUBLIME
    fi

    echo ""
    echo "Full setup guide: ${DOCS_URL}/setup/sublime"
}

# ── Detect OS and install binary ──────────────────────────

cat << 'LOGO'

███████╗ ██████╗ ██╗     ██╗██████╗ ██╗████████╗██╗   ██╗
██╔════╝██╔═══██╗██║     ██║██╔══██╗██║╚══██╔══╝╚██╗ ██╔╝
███████╗██║   ██║██║     ██║██║  ██║██║   ██║    ╚████╔╝
╚════██║██║   ██║██║     ██║██║  ██║██║   ██║     ╚██╔╝
███████║╚██████╔╝███████╗██║██████╔╝██║   ██║      ██║
╚══════╝ ╚═════╝ ╚══════╝╚═╝╚═════╝ ╚═╝   ╚═╝      ╚═╝

██╗      █████╗ ███╗   ██╗ ██████╗ ██╗   ██╗ █████╗  ██████╗ ███████╗
██║     ██╔══██╗████╗  ██║██╔════╝ ██║   ██║██╔══██╗██╔════╝ ██╔════╝
██║     ███████║██╔██╗ ██║██║  ███╗██║   ██║███████║██║  ███╗█████╗
██║     ██╔══██║██║╚██╗██║██║   ██║██║   ██║██╔══██║██║   ██║██╔══╝
███████╗██║  ██║██║ ╚████║╚██████╔╝╚██████╔╝██║  ██║╚██████╔╝███████╗
╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝  ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝

███████╗███████╗██████╗ ██╗   ██╗███████╗██████╗
██╔════╝██╔════╝██╔══██╗██║   ██║██╔════╝██╔══██╗
███████╗█████╗  ██████╔╝██║   ██║█████╗  ██████╔╝
╚════██║██╔══╝  ██╔══██╗╚██╗ ██╔╝██╔══╝  ██╔══██╗
███████║███████╗██║  ██║ ╚████╔╝ ███████╗██║  ██║
╚══════╝╚══════╝╚═╝  ╚═╝  ╚═══╝  ╚══════╝╚═╝  ╚═╝

LOGO

case "$(uname -s)" in
    Linux*)  OS="x86_64-unknown-linux-gnu" ;;
    Darwin*)
        case "$(uname -m)" in
            arm64) OS="aarch64-apple-darwin" ;;
            *)     OS="x86_64-apple-darwin" ;;
        esac
        ;;
    MINGW*|MSYS*|CYGWIN*)
        OS="x86_64-pc-windows-msvc"
        ;;
    *)
        echo "Error: unsupported operating system '$(uname -s)'" >&2
        exit 1
        ;;
esac

if [ "$OS" = "x86_64-pc-windows-msvc" ]; then
    BASE_DIR="${USERPROFILE}/.solidity-language-server"
else
    BASE_DIR="${HOME}/.solidity-language-server"
fi

# Get latest release tag
TAG=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"//;s/".*//')
if [ -z "$TAG" ]; then
    echo "Error: could not determine latest release" >&2
    exit 1
fi

# Strip leading 'v' for version directory (v0.1.32 -> 0.1.32)
VERSION="${TAG#v}"
INSTALL_DIR="${BASE_DIR}/${VERSION}/bin"

echo "Installing ${BINARY} ${TAG}..."

# Download
if [ "$OS" = "x86_64-pc-windows-msvc" ]; then
    ARCHIVE="${BINARY}-${OS}.zip"
else
    ARCHIVE="${BINARY}-${OS}.tar.gz"
fi

URL="https://github.com/${REPO}/releases/download/${TAG}/${ARCHIVE}"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "Downloading ${URL}..."
curl -fsSL "$URL" -o "${TMPDIR}/${ARCHIVE}"

# Download checksums and signature
CHECKSUMS_URL="https://github.com/${REPO}/releases/download/${TAG}/checksums-sha256.txt"
SIGNATURE_URL="https://github.com/${REPO}/releases/download/${TAG}/checksums-sha256.txt.asc"
PUBLIC_KEY_URL="https://raw.githubusercontent.com/${REPO}/main/public-key.asc"

curl -fsSL "$CHECKSUMS_URL" -o "${TMPDIR}/checksums-sha256.txt"
curl -fsSL "$SIGNATURE_URL" -o "${TMPDIR}/checksums-sha256.txt.asc"
curl -fsSL "$PUBLIC_KEY_URL" -o "${TMPDIR}/public-key.asc"

# Verify release integrity
cd "$TMPDIR"
echo "Verifying release integrity..."

# GPG signature
if command -v gpg >/dev/null 2>&1; then
    gpg --quiet --import public-key.asc 2>/dev/null
    if gpg --quiet --verify checksums-sha256.txt.asc checksums-sha256.txt 2>/dev/null; then
        echo "  GPG signature: verified"
    else
        echo "  GPG signature: FAILED" >&2
        exit 1
    fi
else
    echo "  GPG signature: skipped (gpg not installed)"
fi

# SHA-256 checksum
EXPECTED=$(grep "$ARCHIVE" checksums-sha256.txt | awk '{print $1}')
if command -v shasum >/dev/null 2>&1; then
    ACTUAL=$(shasum -a 256 "$ARCHIVE" | awk '{print $1}')
elif command -v sha256sum >/dev/null 2>&1; then
    ACTUAL=$(sha256sum "$ARCHIVE" | awk '{print $1}')
else
    ACTUAL=""
fi

if [ -n "$ACTUAL" ]; then
    if [ "$EXPECTED" = "$ACTUAL" ]; then
        echo "  SHA-256 checksum: verified"
    else
        echo "  SHA-256 checksum: FAILED" >&2
        echo "    expected: $EXPECTED" >&2
        echo "    got:      $ACTUAL" >&2
        exit 1
    fi
else
    echo "  SHA-256 checksum: skipped (shasum/sha256sum not installed)"
fi

# Extract
if [ "$OS" = "x86_64-pc-windows-msvc" ]; then
    unzip -q "$ARCHIVE"
else
    tar xzf "$ARCHIVE"
fi

# Determine binary name (Windows has .exe)
if [ "$OS" = "x86_64-pc-windows-msvc" ]; then
    BIN_NAME="${BINARY}.exe"
else
    BIN_NAME="${BINARY}"
fi

# Install into versioned directory
mkdir -p "$INSTALL_DIR"
cp "$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
chmod +x "$INSTALL_DIR/$BIN_NAME" 2>/dev/null || true

echo "Installed ${BINARY} ${TAG} to ${INSTALL_DIR}/${BIN_NAME}"

# Create stable symlink at ~/.solidity-language-server/bin/
SYMLINK_DIR="${BASE_DIR}/bin"
mkdir -p "$SYMLINK_DIR"
ln -sf "$INSTALL_DIR/$BIN_NAME" "$SYMLINK_DIR/$BIN_NAME"
echo "Symlinked ${SYMLINK_DIR}/${BIN_NAME} -> ${TAG}"

# Add to PATH if needed
PATH_EXPORT='export PATH="$HOME/.solidity-language-server/bin:$PATH"'
case ":$PATH:" in
    *":${SYMLINK_DIR}:"*)
        echo ""
        echo "PATH already includes ${SYMLINK_DIR}"
        ;;
    *)
        # Detect shell rc file
        SHELL_RC=""
        case "$(basename "${SHELL:-/bin/sh}")" in
            zsh)  SHELL_RC="${HOME}/.zshrc" ;;
            bash)
                if [ -f "${HOME}/.bashrc" ]; then
                    SHELL_RC="${HOME}/.bashrc"
                elif [ -f "${HOME}/.bash_profile" ]; then
                    SHELL_RC="${HOME}/.bash_profile"
                else
                    SHELL_RC="${HOME}/.bashrc"
                fi
                ;;
            fish)
                # fish uses a different syntax
                SHELL_RC=""
                echo ""
                echo "Add this to ~/.config/fish/config.fish:"
                echo ""
                echo "  set -gx PATH \$HOME/.solidity-language-server/bin \$PATH"
                ;;
            *)    SHELL_RC="${HOME}/.profile" ;;
        esac

        if [ -n "$SHELL_RC" ]; then
            if [ -f "$SHELL_RC" ] && grep -qF '.solidity-language-server/bin' "$SHELL_RC" 2>/dev/null; then
                echo ""
                echo "PATH export already in ${SHELL_RC}"
            else
                printf "\nAdd solidity-language-server to PATH in %s? [Y/n] " "$SHELL_RC"
                read -r PATH_REPLY < /dev/tty
                case "$PATH_REPLY" in
                    n|N)
                        echo ""
                        echo "Skipped. Add this manually to your shell profile:"
                        echo ""
                        echo "  ${PATH_EXPORT}"
                        ;;
                    *)
                        echo "" >> "$SHELL_RC"
                        echo "# solidity-language-server" >> "$SHELL_RC"
                        echo "$PATH_EXPORT" >> "$SHELL_RC"
                        echo "Added PATH export to ${SHELL_RC}"
                        echo ""
                        echo "Run 'source ${SHELL_RC}' or open a new terminal to use it."
                        ;;
                esac
            fi
        fi
        ;;
esac

setup_claude() {
    echo ""
    echo "Run these commands in Claude Code:"
    echo ""
    echo "  /plugin marketplace add asyncswap/solidity-language-server-plugin"
    echo "  /plugin install solidity-language-server@asyncswap"
    echo "  /reload-plugins"
    echo ""
    echo "Full setup guide: ${DOCS_URL}/agents/claude"
}

setup_opencode() {
    OPENCODE_DIR="${HOME}/.config/opencode"
    OPENCODE_FILE="${OPENCODE_DIR}/opencode.json"

    if confirm_append "$OPENCODE_FILE" "solidity-language-server"; then
        mkdir -p "$OPENCODE_DIR"
        cat > "$OPENCODE_FILE" << 'OPENCODE'
{
  "$schema": "https://opencode.ai/config.json",
  "lsp": {
    "solidity": {
      "command": ["solidity-language-server", "--stdio"],
      "extensions": [".sol"],
      "initialization": {
        "solidity-language-server": {
          "inlayHints": {
            "parameters": true
          },
          "lint": {
            "enabled": true
          },
          "fileOperations": {
            "templateOnCreate": true,
            "updateImportsOnRename": true,
            "updateImportsOnDelete": true
          },
          "projectIndex": {
            "fullProjectScan": true
          }
        }
      }
    }
  }
}
OPENCODE
        echo "Wrote ${OPENCODE_FILE}"
    fi

    echo ""
    echo "Enable the experimental LSP tool with:"
    echo ""
    echo "  OPENCODE_EXPERIMENTAL_LSP_TOOL=true opencode"
    echo ""
    echo "Full setup guide: ${DOCS_URL}/agents/opencode"
}

# ── Editor setup prompt ──────────────────────────────────

echo ""
echo "Select your editor or agent:"
echo ""
echo "  1) Neovim"
echo "  2) Helix"
echo "  3) VS Code / Cursor"
echo "  4) Zed"
echo "  5) Vim"
echo "  6) Emacs"
echo "  7) Sublime Text"
echo "  8) Claude Code"
echo "  9) OpenCode"
echo "  10) Skip"
echo ""
printf "> "
read -r EDITOR_CHOICE < /dev/tty

case "$EDITOR_CHOICE" in
    1) setup_neovim ;;
    2) setup_helix ;;
    3) setup_vscode ;;
    4) setup_zed ;;
    5) setup_vim ;;
    6) setup_emacs ;;
    7) setup_sublime ;;
    8) setup_claude ;;
    9) setup_opencode ;;
    *) echo "Skipping editor setup." ;;
esac

}

main
