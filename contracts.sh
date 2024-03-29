#!/bin/bash

# ---------------------------------------------
# Script: contracts.sh
# Purpose:
# This script serves as a utility tool to compile contracts used in the test suite.
# It provides actions to compile individual contracts or both contracts used in the test suite.
# Actions:
# - compile-dao-assets-contract: Compiles the dao-assets-contract.
# - compile-vesting-wallet-contract: Compiles the vesting-wallet-contract.
# - compile-vote-escrow-contract: Compiles the vote-escrow-contract.
# - compile-test-contracts: Compiles both dao-assets-contract and vesting-wallet-contract.
# - test-contracts: Runs the test suite for contracts.
# ---------------------------------------------

ACTION=$1
TARGET_DIR=$2
BASE_DIR="$(pwd "$0")"

# Evaluate the action to perform
case $ACTION in
    "compile-dao-assets-contract")
        echo "Starting to compile dao-assets-contract..."
        cd "$BASE_DIR/contracts/extensions/dao-assets-contract" || { echo "Failed to navigate to dao-assets-contract directory"; exit 1; }
        cargo contract build
        cp "$BASE_DIR/target/ink/dao_assets_contract/dao_assets_contract.wasm" "$BASE_DIR/tests/contracts/wasm/test_dao_assets_contract.wasm"
        cp "$BASE_DIR/target/ink/dao_assets_contract/dao_assets_contract.json" "$BASE_DIR/tests/contracts/wasm/test_dao_assets_contract.json"
        cd "$BASE_DIR" || { echo "Failed to navigate back to base directory"; exit 1; }
        echo "Compilation of dao-assets-contract completed."
        ;;

    "compile-vesting-wallet-contract")
        echo "Starting to compile vesting-wallet-contract..."
        cd "$BASE_DIR/contracts/plugins/vesting-wallet-contract" || { echo "Failed to navigate to vesting-wallet-contract directory"; exit 1; }
        cargo contract build
        cp "$BASE_DIR/target/ink/vesting_wallet_contract/vesting_wallet_contract.wasm" "$BASE_DIR/tests/contracts/wasm/test_vesting_wallet_contract.wasm"
        cp "$BASE_DIR/target/ink/vesting_wallet_contract/vesting_wallet_contract.json" "$BASE_DIR/tests/contracts/wasm/test_vesting_wallet_contract.json"
        cd "$BASE_DIR" || { echo "Failed to navigate back to base directory"; exit 1; }
        echo "Compilation of vesting-wallet-contract completed."
        ;;

    "compile-vote-escrow-contract")
        echo "Starting to compile vote-escrow-contract..."
        cd "$BASE_DIR/contracts/plugins/vote-escrow-contract" || { echo "Failed to navigate to vote-escrow-contract directory"; exit 1; }
        cargo contract build
        cp "$BASE_DIR/target/ink/vote_escrow_contract/vote_escrow_contract.wasm" "$BASE_DIR/tests/contracts/wasm/test_vote_escrow_contract.wasm"
        cp "$BASE_DIR/target/ink/vote_escrow_contract/vote_escrow_contract.json" "$BASE_DIR/tests/contracts/wasm/test_vote_escrow_contract.json"
        cd "$BASE_DIR" || { echo "Failed to navigate back to base directory"; exit 1; }
        echo "Compilation of vote-escrow-contract completed."
        ;;

    "compile-genesis-dao-contract")
        echo "Starting to compile genesis-dao-contract..."
        cd "$BASE_DIR/contracts/hooks/genesis-dao-contract" || { echo "Failed to navigate to genesis-dao-contract directory"; exit 1; }
        cargo contract build
        cp "$BASE_DIR/target/ink/genesis_dao_contract/genesis_dao_contract.wasm" "$BASE_DIR/tests/contracts/wasm/test_genesis_dao_contract.wasm"
        cp "$BASE_DIR/target/ink/genesis_dao_contract/genesis_dao_contract.json" "$BASE_DIR/tests/contracts/wasm/test_genesis_dao_contract.json"
        cd "$BASE_DIR" || { echo "Failed to navigate back to base directory"; exit 1; }
        echo "Compilation of genesis-dao-contract completed."
        ;;

    "compile-test-contracts")
        echo "Starting to compile all test contracts..."
        # Call all individual compile actions
        ./contracts.sh compile-dao-assets-contract
        ./contracts.sh compile-vesting-wallet-contract
        ./contracts.sh compile-vote-escrow-contract
        ./contracts.sh compile-genesis-dao-contract
        echo "Compilation of all test contracts completed."
        ;;

    "test-contracts")
        echo "Running tests for contracts..."
        cargo test -p contracts-test-suite
        ;;

    "create-release")
        echo "Creating release"
        # compile to test first
        ./contracts.sh compile-test-contracts

        # copy to target dir
        mkdir -p "$TARGET_DIR/wasm"
        cp "$BASE_DIR/tests/contracts/wasm/test_dao_assets_contract.wasm" "$TARGET_DIR/wasm/dao_asset_contract.wasm"
        cp "$BASE_DIR/tests/contracts/wasm/test_vesting_wallet_contract.wasm" "$TARGET_DIR/wasm/vesting_wallet_contract.wasm"
        cp "$BASE_DIR/tests/contracts/wasm/test_vote_escrow_contract.wasm" "$TARGET_DIR/wasm/vote_escrow_contract.wasm"
        cp "$BASE_DIR/tests/contracts/wasm/test_genesis_dao_contract.wasm" "$TARGET_DIR/wasm/genesis_dao_contract.wasm"
    
        cp "$BASE_DIR/tests/contracts/wasm/test_dao_assets_contract.json" "$TARGET_DIR/wasm/dao_asset_contract.json"
        cp "$BASE_DIR/tests/contracts/wasm/test_vesting_wallet_contract.json" "$TARGET_DIR/wasm/vesting_wallet_contract.json"
        cp "$BASE_DIR/tests/contracts/wasm/test_vote_escrow_contract.json" "$TARGET_DIR/wasm/vote_escrow_contract.json"
        cp "$BASE_DIR/tests/contracts/wasm/test_genesis_dao_contract.json" "$TARGET_DIR/wasm/genesis_dao_contract.json"

        echo "Release created"
        ;;
    *)
        printf "\nInvalid action. Valid actions are\n\n - compile-dao-assets-contract\n - compile-vesting-wallet-contract\n - compile-vote-escrow-contract\n - compile-genesis-dao-contract\n - compile-test-contracts\n - test-contracts\n - create-release\n\n"
        ;;
esac
