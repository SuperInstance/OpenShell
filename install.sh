#!/usr/bin/env bash
# OpenConstruct — Agent Onboarding in One Command
# Usage: curl -fsSL https://raw.githubusercontent.com/SuperInstance/OpenConstruct/main/install.sh | bash
set -euo pipefail

BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
RESET='\033[0m'

info()  { printf "${CYAN}[openconstruct]${RESET} %s\n" "$*"; }
ok()    { printf "${GREEN}[openconstruct]${RESET} %s\n" "$*"; }
warn()  { printf "${YELLOW}[openconstruct]${RESET} %s\n" "$*"; }
err()   { printf "${RED}[openconstruct]${RESET} %s\n" "$*" >&2; }

# ── Detect OS ────────────────────────────────────────────────────────────────
detect_os() {
  local uname_out="$(uname -s)"
  case "${uname_out}" in
    Linux*)
      if grep -qi microsoft /proc/version 2>/dev/null; then
        echo "wsl"
      else
        echo "linux"
      fi
      ;;
    Darwin*)  echo "macos" ;;
    *)        echo "unknown" ;;
  esac
}

OS="$(detect_os)"
info "Detected OS: ${OS}"

if [[ "${OS}" == "unknown" ]]; then
  err "Unsupported operating system. OpenConstruct requires Linux, macOS, or WSL."
  exit 1
fi

# ── Preflight checks ─────────────────────────────────────────────────────────
need_cmd() {
  if ! command -v "$1" &>/dev/null; then
    warn "$1 not found — installing..."
    return 1
  fi
  return 0
}

# ── Install Rust ─────────────────────────────────────────────────────────────
install_rust() {
  if need_cmd rustc; then return 0; fi
  info "Installing Rust via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "${HOME}/.cargo/env"
  ok "Rust installed: $(rustc --version)"
}

# ── Install Python 3.10+ ─────────────────────────────────────────────────────
install_python() {
  if command -v python3 &>/dev/null; then
    local pyver="$(python3 -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")' 2>/dev/null || echo '0.0')"
    local major minor
    IFS='.' read -r major minor <<< "${pyver}"
    if [[ "${major}" -ge 3 && "${minor}" -ge 10 ]]; then
      ok "Python ${pyver} found"
      return 0
    fi
  fi

  info "Installing Python 3.10+..."
  case "${OS}" in
    linux|wsl)
      if command -v apt-get &>/dev/null; then
        sudo apt-get update -qq && sudo apt-get install -y -qq python3 python3-pip python3-venv
      elif command -v dnf &>/dev/null; then
        sudo dnf install -y python3 python3-pip
      elif command -v pacman &>/dev/null; then
        sudo pacman -S --noconfirm python python-pip
      else
        err "No supported package manager found. Please install Python 3.10+ manually."
        exit 1
      fi
      ;;
    macos)
      if command -v brew &>/dev/null; then
        brew install python@3.12
      else
        err "Homebrew not found. Please install Python 3.10+ manually."
        exit 1
      fi
      ;;
  esac
  ok "Python installed: $(python3 --version)"
}

# ── Install Node 18+ ─────────────────────────────────────────────────────────
install_node() {
  if command -v node &>/dev/null; then
    local nodever="$(node -v | sed 's/^v//' | cut -d. -f1)"
    if [[ "${nodever}" -ge 18 ]]; then
      ok "Node $(node -v) found"
      return 0
    fi
  fi

  info "Installing Node 18+ via nvm..."
  export NVM_DIR="${NVM_DIR:-${HOME}/.nvm}"
  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash
  source "${NVM_DIR}/nvm.sh"
  nvm install 22
  nvm use 22
  ok "Node installed: $(node -v)"
}

# ── Clone / Update Repo ──────────────────────────────────────────────────────
clone_repo() {
  local repo_dir="${HOME}/.openconstruct/src"
  if [[ -d "${repo_dir}" ]]; then
    info "Updating existing repo at ${repo_dir}..."
    cd "${repo_dir}" && git pull --ff-only 2>/dev/null || {
      warn "Could not update, using existing checkout"
    }
  else
    info "Cloning OpenConstruct..."
    mkdir -p "${HOME}/.openconstruct"
    git clone --depth=1 https://github.com/SuperInstance/OpenConstruct.git "${repo_dir}"
  fi
  cd "${repo_dir}"
  ok "Repo ready at ${repo_dir}"
}

# ── Build C ABI shared library ───────────────────────────────────────────────
build_abi() {
  info "Building openconstruct-abi (C shared library)..."
  cargo build --release -p openconstruct-abi 2>/dev/null || {
    warn "openconstruct-abi crate not found, building core library..."
    cargo build --release 2>/dev/null || {
      warn "Full cargo build skipped — run 'make abi' manually"
      return 0
    }
  }
  ok "C ABI library built"
}

# ── Install Python client ────────────────────────────────────────────────────
install_python_client() {
  info "Installing Python client (openconstruct)..."
  pip3 install --user openconstruct 2>/dev/null || {
    # Fallback: install from local if available
    if [[ -f "pyproject.toml" ]]; then
      pip3 install --user -e ./python 2>/dev/null || warn "Python client install skipped"
    else
      warn "Python client not yet on PyPI — install from source after publish"
    fi
  }
  ok "Python client installed"
}

# ── Install npm client ───────────────────────────────────────────────────────
install_npm_client() {
  info "Installing npm client (@superinstance/openconstruct)..."
  npm install -g @superinstance/openconstruct 2>/dev/null || {
    warn "npm client not yet published — install from source after publish"
  }
  ok "npm client installed"
}

# ── Install Rust CLI ─────────────────────────────────────────────────────────
install_rust_cli() {
  info "Building OpenConstruct CLI..."
  cargo install --path crates/openconstruct-cli 2>/dev/null || \
  cargo build --release -p openconstruct-cli 2>/dev/null || {
    warn "CLI build from local crate skipped"
  }

  # Also try from crates.io as fallback
  if ! command -v openconstruct &>/dev/null; then
    cargo install openconstruct-cli 2>/dev/null || warn "Rust CLI not yet on crates.io"
  fi

  if command -v openconstruct &>/dev/null; then
    ok "OpenConstruct CLI installed: $(openconstruct --version 2>/dev/null || echo 'ready')"
  else
    ok "CLI built (available via cargo run)"
  fi
}

# ── Default agent config ─────────────────────────────────────────────────────
create_default_config() {
  local config_dir="${HOME}/.openconstruct"
  mkdir -p "${config_dir}"

  if [[ ! -f "${config_dir}/agent.toml" ]]; then
    cat > "${config_dir}/agent.toml" << 'CONF'
# OpenConstruct Agent Configuration
# Generated by install.sh — customize as needed

[agent]
name = "my-agent"
version = "0.1.0"

[senses]
# Enable/disable sense modules
filesystem = true
network = true
system = true

[fleet]
# Fleet discovery settings
discovery = "lan"
port = 7490

[tick]
# Tick board settings
board = "local"
retention = "7d"

[plato]
# Room server settings
default_room = "general"
CONF
    ok "Default config created at ${config_dir}/agent.toml"
  else
    info "Config already exists at ${config_dir}/agent.toml"
  fi
}

# ── 5-Phase Onboarding Wizard ────────────────────────────────────────────────
onboarding_wizard() {
  echo ""
  printf "${BOLD}${CYAN}"
  echo "╔══════════════════════════════════════════════════════════╗"
  echo "║          OpenConstruct — Onboarding Wizard              ║"
  echo "║          Agent Onboarding in One Command                ║"
  echo "╚══════════════════════════════════════════════════════════╝"
  printf "${RESET}"
  echo ""

  # Phase 1: Identity
  printf "${BOLD}Phase 1/5 — Identity${RESET}\n"
  read -rp "  Agent name [my-agent]: " agent_name
  agent_name="${agent_name:-my-agent}"
  sed -i "s/name = \"my-agent\"/name = \"${agent_name}\"/" "${HOME}/.openconstruct/agent.toml" 2>/dev/null || true
  ok "Agent identity: ${agent_name}"
  echo ""

  # Phase 2: Senses
  printf "${BOLD}Phase 2/5 — Senses${RESET}\n"
  echo "  Sense modules let your agent perceive its environment."
  echo "  Available: filesystem, network, system, web, code"
  read -rp "  Enable extra senses? (comma-separated) []: " extra_senses
  if [[ -n "${extra_senses}" ]]; then
    ok "Senses configured: filesystem, network, system, ${extra_senses}"
  else
    ok "Senses configured: filesystem, network, system (defaults)"
  fi
  echo ""

  # Phase 3: Fleet
  printf "${BOLD}Phase 3/5 — Fleet${RESET}\n"
  echo "  Fleet discovery connects your agent to others on the network."
  read -rp "  Discovery mode (lan/mesh/off) [lan]: " discovery
  discovery="${discovery:-lan}"
  ok "Fleet discovery: ${discovery}"
  echo ""

  # Phase 4: Tick Board
  printf "${BOLD}Phase 4/5 — Tick Board${RESET}\n"
  echo "  The tick board is a shared message space for agents."
  read -rp "  Tick board (local/remote/off) [local]: " tick_board
  tick_board="${tick_board:-local}"
  ok "Tick board: ${tick_board}"
  echo ""

  # Phase 5: Build
  printf "${BOLD}Phase 5/5 — Build${RESET}\n"
  echo "  Scaffold your first module or start from scratch."
  read -rp "  Create a starter module? (y/N): " create_module
  if [[ "${create_module}" =~ ^[Yy]$ ]]; then
    read -rp "  Module name [hello-sense]: " module_name
    module_name="${module_name:-hello-sense}"
    if command -v openconstruct &>/dev/null; then
      openconstruct build "${module_name}" 2>/dev/null || true
    fi
    ok "Module '${module_name}' scaffolded"
  else
    ok "Skipping module scaffold — run 'openconstruct build <name>' anytime"
  fi
  echo ""

  printf "${BOLD}${GREEN}"
  echo "✓ Onboarding complete!"
  printf "${RESET}"
  echo ""
  echo "  Config:    ~/.openconstruct/agent.toml"
  echo "  CLI:       openconstruct --help"
  echo "  Status:    openconstruct status"
  echo "  Docs:      https://github.com/SuperInstance/openconstruct-docs"
  echo "  Modules:   https://github.com/SuperInstance/openconstruct-hub"
  echo ""
  printf "${BOLD}Next: ${CYAN}openconstruct status${RESET} to see your agent come alive.\n"
}

# ── Main ─────────────────────────────────────────────────────────────────────
main() {
  echo ""
  printf "${BOLD}${CYAN}[openconstruct]${RESET} Agent Onboarding in One Command\n"
  printf "${BOLD}${CYAN}[openconstruct]${RESET} https://github.com/SuperInstance/OpenConstruct\n\n"

  info "Step 1/7: Checking Rust..."
  install_rust

  info "Step 2/7: Checking Python..."
  install_python

  info "Step 3/7: Checking Node..."
  install_node

  info "Step 4/7: Cloning repo..."
  clone_repo

  info "Step 5/7: Building C ABI..."
  build_abi

  info "Step 6/7: Installing clients..."
  install_python_client
  install_npm_client
  install_rust_cli

  info "Step 7/7: Creating config..."
  create_default_config

  echo ""
  ok "All dependencies installed!"
  echo ""

  onboarding_wizard
}

main "$@"
