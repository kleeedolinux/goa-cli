#!/bin/bash

# GOA CLI macOS Setup and Installation Script
# This script clones the GOA CLI repository, builds it if Rust is available,
# and installs it using the install.sh script

echo "GOA CLI Setup for macOS"
echo "======================="
echo

# Define installation directory
INSTALL_DIR="/usr/local/bin"
BINARY_PATH="$INSTALL_DIR/goa"

# Check if GOA CLI is already installed
IS_UPDATE=0
if [ -f "$BINARY_PATH" ]; then
    IS_UPDATE=1
    echo "GOA CLI is already installed."
    read -p "Do you want to update it? (y/n): " confirm_update
    
    if [[ $confirm_update != "y" && $confirm_update != "Y" ]]; then
        echo "Update cancelled. Exiting..."
        exit 0
    fi
    
    echo "Updating GOA CLI..."
fi

# Check if git is installed
if ! command -v git &> /dev/null; then
    echo "Error: git is required but not installed. Please install git and try again."
    echo "You can install git using Homebrew: brew install git"
    exit 1
fi

# Check if Rust/Cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Rust is not installed on your system."
    read -p "Would you like to install Rust now? (y/n): " install_rust
    
    if [[ $install_rust == "y" || $install_rust == "Y" ]]; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        
        # Source the cargo environment
        source "$HOME/.cargo/env"
        
        if ! command -v cargo &> /dev/null; then
            echo "Error: Failed to install Rust. Please install it manually."
            echo "Visit https://www.rust-lang.org/tools/install for instructions."
            exit 1
        fi
        
        echo "Rust installed successfully!"
    else
        echo "Rust installation declined. Proceeding to download pre-built binary..."
        # Download and run the install script directly
        curl -sSL https://raw.githubusercontent.com/kleeedolinux/goa-cli/master/scripts/install.sh | bash
        exit $?
    fi
fi

# Create a temporary directory for the clone
TEMP_DIR=$(mktemp -d)
echo "Working in temporary directory: $TEMP_DIR"

# Clone the repository
echo "Cloning GOA CLI repository..."
git clone https://github.com/kleeedolinux/goa-cli.git "$TEMP_DIR"

if [ $? -ne 0 ]; then
    echo "Error: Failed to clone the repository."
    exit 1
fi

# Change to the repository directory
cd "$TEMP_DIR"

# Build the project
echo "Building GOA CLI from source..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Error: Failed to build the project."
    echo "Falling back to pre-built binary..."
    curl -sSL https://raw.githubusercontent.com/kleeedolinux/goa-cli/master/scripts/install.sh | bash
    exit $?
fi

# Define installation directory
if [ ! -d "$INSTALL_DIR" ]; then
    echo "Creating installation directory..."
    sudo mkdir -p "$INSTALL_DIR"
fi

# Copy the built binary
echo "Installing GOA CLI to $BINARY_PATH..."
sudo cp "./target/release/goa" "$BINARY_PATH"
sudo chmod +x "$BINARY_PATH"

if [ $? -ne 0 ]; then
    echo "Error: Failed to install the GOA CLI binary. Do you have permission to write to $INSTALL_DIR?"
    echo "Falling back to install script..."
    curl -sSL https://raw.githubusercontent.com/kleeedolinux/goa-cli/master/scripts/install.sh | bash
    exit $?
fi

# Clean up
echo "Cleaning up temporary files..."
cd -
rm -rf "$TEMP_DIR"

echo
if [ $IS_UPDATE -eq 1 ]; then
    echo "GOA CLI has been successfully updated!"
else
    echo "GOA CLI has been successfully installed!"
fi
echo "You can now use the 'goa' command from your terminal."
echo "For help, run: goa --help"
echo
echo "Thank you for using Go on Airplanes!" 