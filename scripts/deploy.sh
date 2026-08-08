#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
#  OrbitPay – Deploy script
#
#  Usage:
#    ./scripts/deploy.sh <network> <wasm-path>
#
#  Networks:  testnet | mainnet | futurenet | standalone
#
#  Prerequisites:
#    • stellar CLI installed  (https://developers.stellar.org/docs/tools/stellar-cli)
#    • A funded identity configured: `stellar keys generate --global deployer`
#    • DEPLOYER_IDENTITY env var set (defaults to "deployer")
# ─────────────────────────────────────────────────────────────────────────────

set -euo pipefail

NETWORK="${1:-testnet}"
WASM_PATH="${2:-}"
IDENTITY="${DEPLOYER_IDENTITY:-deployer}"
LOG_PREFIX="[OrbitPay Deploy]"

# ── Validation ────────────────────────────────────────────────────────────────

if [[ -z "$WASM_PATH" ]]; then
    echo "$LOG_PREFIX ERROR: WASM path not provided."
    echo "  Usage: ./scripts/deploy.sh <network> <wasm-path>"
    exit 1
fi

if [[ ! -f "$WASM_PATH" ]]; then
    echo "$LOG_PREFIX ERROR: WASM file not found at '$WASM_PATH'."
    echo "  Run 'make build' first."
    exit 1
fi

# ── Network RPC configuration ─────────────────────────────────────────────────

case "$NETWORK" in
    testnet)
        NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
        RPC_URL="https://soroban-testnet.stellar.org"
        ;;
    futurenet)
        NETWORK_PASSPHRASE="Test SDF Future Network ; October 2022"
        RPC_URL="https://rpc-futurenet.stellar.org"
        ;;
    mainnet)
        NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
        RPC_URL="https://mainnet.sorobanrpc.com"
        echo ""
        echo "  ⚠️  MAINNET DEPLOYMENT"
        echo "  Network : $NETWORK"
        echo "  Identity: $IDENTITY"
        echo "  WASM    : $WASM_PATH"
        echo ""
        read -r -p "  Type 'deploy mainnet' to confirm: " CONFIRM
        if [[ "$CONFIRM" != "deploy mainnet" ]]; then
            echo "  Aborted."
            exit 0
        fi
        ;;
    standalone)
        NETWORK_PASSPHRASE="Standalone Network ; February 2017"
        RPC_URL="http://localhost:8000/soroban/rpc"
        ;;
    *)
        echo "$LOG_PREFIX ERROR: Unknown network '$NETWORK'."
        echo "  Supported: testnet | mainnet | futurenet | standalone"
        exit 1
        ;;
esac

# ── Preflight checks ──────────────────────────────────────────────────────────

if ! command -v stellar &>/dev/null; then
    echo "$LOG_PREFIX ERROR: 'stellar' CLI not found."
    echo "  Install: https://developers.stellar.org/docs/tools/stellar-cli"
    exit 1
fi

echo ""
echo "$LOG_PREFIX Starting deployment"
echo "  Network : $NETWORK"
echo "  RPC URL : $RPC_URL"
echo "  Identity: $IDENTITY"
echo "  WASM    : $WASM_PATH  ($(wc -c < "$WASM_PATH") bytes)"
echo ""

# ── Upload WASM ───────────────────────────────────────────────────────────────

echo "$LOG_PREFIX Uploading WASM to the network..."

WASM_HASH=$(stellar contract upload \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    --rpc-url "$RPC_URL" \
    --source "$IDENTITY" \
    --wasm "$WASM_PATH")

echo "$LOG_PREFIX WASM hash: $WASM_HASH"

# ── Deploy contract instance ──────────────────────────────────────────────────

echo "$LOG_PREFIX Deploying contract instance..."

CONTRACT_ID=$(stellar contract deploy \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    --rpc-url "$RPC_URL" \
    --source "$IDENTITY" \
    --wasm-hash "$WASM_HASH")

echo ""
echo "$LOG_PREFIX ✅  Deployment successful!"
echo "  Contract ID : $CONTRACT_ID"
echo "  WASM Hash   : $WASM_HASH"
echo "  Network     : $NETWORK"
echo ""

# ── Persist deployment info ───────────────────────────────────────────────────

DEPLOY_LOG="deployments/${NETWORK}.json"
mkdir -p deployments

cat > "$DEPLOY_LOG" <<JSON
{
  "network": "$NETWORK",
  "contract_id": "$CONTRACT_ID",
  "wasm_hash": "$WASM_HASH",
  "deployed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "wasm_path": "$WASM_PATH"
}
JSON

echo "$LOG_PREFIX Deployment record saved to $DEPLOY_LOG"
