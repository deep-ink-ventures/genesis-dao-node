#!/bin/bash

# ---------------------------------------------
# Script: contracts.sh
# Purpose:
# This script serves as a utility tool to compile contracts used in the test suite.
# It provides actions to compile individual contracts or both contracts used in the test suite.
# Actions:
# - compile-dao-assets-contract: Compiles the dao-assets-contract.
# - compile-vesting-wallet-contract: Compiles the vesting-wallet-contract.
# - compile-test-contracts: Compiles both dao-assets-contract and vesting-wallet-contract.
# - test-contracts: Runs the test suite for contracts.
# ---------------------------------------------

ACTION=$1
BASE_DIR="$(pwd "$0")"

# Evaluate the action to perform
case $ACTION in
    "compile-dao-assets-contract")
        echo "Starting to compile dao-assets-contract..."
        cd "$BASE_DIR/contracts/extensions/dao-assets-contract" || { echo "Failed to navigate to dao-assets-contract directory"; exit 1; }
        cargo contract build
        cp "$BASE_DIR/target/ink/dao_assets_contract/dao_assets_contract.wasm" "$BASE_DIR/tests/contracts/wasm/test_dao_assets_contract.wasm"
        cd "$BASE_DIR" || { echo "Failed to navigate back to base directory"; exit 1; }
        echo "Compilation of dao-assets-contract completed."
        ;;

    "compile-vesting-wallet-contract")
        echo "Starting to compile vesting-wallet-contract..."
        cd "$BASE_DIR/contracts/plugins/vesting-wallet-contract" || { echo "Failed to navigate to vesting-wallet-contract directory"; exit 1; }
        cargo contract build
        cp "$BASE_DIR/target/ink/vesting_wallet_contract/vesting_wallet_contract.wasm" "$BASE_DIR/tests/contracts/wasm/test_vesting_wallet_contract.wasm"
        cd "$BASE_DIR" || { echo "Failed to navigate back to base directory"; exit 1; }
        echo "Compilation of vesting-wallet-contract completed."
        ;;

    "compile-test-contracts")
        echo "Starting to compile all test contracts..."
        # Call both individual compile actions
        ./contracts.sh compile-dao-assets-contract
        ./contracts.sh compile-vesting-wallet-contract
        echo "Compilation of all test contracts completed."
        ;;

    "test-contracts")
        echo "Running tests for contracts..."
        cargo test -p contracts-test-suite
        ;;

    *)
        printf "\nInvalid action. Valid actions are\n\n - compile-dao-assets-contract\n - compile-vesting-wallet-contract\n - compile-test-contracts\n - test-contracts\n\n"
        ;;
esac
