#!/bin/bash

# GOA CLI macOS Setup and Installation Script
# This script clones the GOA CLI repository, builds it if Rust is available,
# and installs it using the install.sh script

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Print styled text
print_header() {
    echo -e "${BOLD}${BLUE}$1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_info() {
    echo -e "${CYAN}ℹ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_step() {
    echo -e "${PURPLE}▶ $1${NC}"
}

# Progress bar function
progress_bar() {
    local duration=$1
    local barsize=40
    local char="▓"
    
    for i in $(seq 1 $barsize); do
        echo -ne "${CYAN}${char}${NC}"
        sleep $(bc <<< "scale=3; $duration/$barsize")
    done
    echo -e " ${GREEN}100%${NC}"
}

# Display banner
display_banner() {
    echo -e "${BOLD}${BLUE}"
    echo "  _____  ____           _____ _      _____  "
    echo " / ____|/ __ \   /\    / ____| |    |_   _| "
    echo "| |  __| |  | | /  \  | |    | |      | |   "
    echo "| | |_ | |  | |/ /\ \ | |    | |      | |   "
    echo "| |__| | |__| / ____ \| |____| |____ _| |_  "
    echo " \_____|\____/_/    \_\\_____|______|_____| "
    echo -e "${NC}"
    echo -e "${BOLD}${CYAN}macOS Setup and Installation${NC}\n"
}

# Main script starts here
clear
display_banner

# Define installation directory
INSTALL_DIR="/usr/local/bin"
BINARY_PATH="$INSTALL_DIR/goa"

# Check if GOA CLI is already installed
IS_UPDATE=0
if [ -f "$BINARY_PATH" ]; then
    IS_UPDATE=1
    print_info "GOA CLI is already installed."
    echo -ne "${YELLOW}Do you want to update it? (y/n): ${NC}"
    read -r confirm_update
    
    if [[ $confirm_update != "y" && $confirm_update != "Y" ]]; then
        print_warning "Update cancelled. Exiting..."
        exit 0
    fi
    
    print_step "Updating GOA CLI..."
fi

# Check if git is installed
if ! command -v git &> /dev/null; then
    print_error "git is required but not installed. Please install git and try again."
    print_warning "You can install git using Homebrew: ${BOLD}brew install git${NC}"
    exit 1
fi

# Check if Rust/Cargo is installed
if ! command -v cargo &> /dev/null; then
    print_info "Rust is not installed on your system."
    echo -ne "${YELLOW}Would you like to install Rust now? (y/n): ${NC}"
    read -r install_rust
    
    if [[ $install_rust == "y" || $install_rust == "Y" ]]; then
        print_step "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        
        # Source the cargo environment
        source "$HOME/.cargo/env"
        
        if ! command -v cargo &> /dev/null; then
            print_error "Failed to install Rust. Please install it manually."
            print_warning "Visit ${BOLD}https://www.rust-lang.org/tools/install${NC} for instructions."
            exit 1
        fi
        
        print_success "Rust installed successfully!"
    else
        print_warning "Rust installation declined. Proceeding to download pre-built binary..."
        # Download and run the install script directly
        curl -sSL https://raw.githubusercontent.com/kleeedolinux/goa-cli/master/scripts/install.sh | bash
        exit $?
    fi
fi

# Create a temporary directory for the clone
TEMP_DIR=$(mktemp -d)
print_info "Working in temporary directory: ${BOLD}$TEMP_DIR${NC}"

# Clone the repository
print_step "Cloning GOA CLI repository..."
git clone https://github.com/kleeedolinux/goa-cli.git "$TEMP_DIR"

if [ $? -ne 0 ]; then
    print_error "Failed to clone the repository."
    exit 1
fi

# Change to the repository directory
cd "$TEMP_DIR"

# Build the project
print_step "Building GOA CLI from source..."
echo -e "${CYAN}This may take a few moments...${NC}"
cargo build --release

if [ $? -ne 0 ]; then
    print_error "Failed to build the project."
    print_warning "Falling back to pre-built binary..."
    curl -sSL https://raw.githubusercontent.com/kleeedolinux/goa-cli/master/scripts/install.sh | bash
    exit $?
fi

# Define installation directory
if [ ! -d "$INSTALL_DIR" ]; then
    print_step "Creating installation directory..."
    sudo mkdir -p "$INSTALL_DIR"
fi

# Copy the built binary
print_step "Installing GOA CLI to $BINARY_PATH..."
sudo cp "./target/release/goa" "$BINARY_PATH"
sudo chmod +x "$BINARY_PATH"

if [ $? -ne 0 ]; then
    print_error "Failed to install the GOA CLI binary. Do you have permission to write to $INSTALL_DIR?"
    print_warning "Falling back to install script..."
    curl -sSL https://raw.githubusercontent.com/kleeedolinux/goa-cli/master/scripts/install.sh | bash
    exit $?
fi

# Clean up
print_step "Cleaning up temporary files..."
cd - > /dev/null
rm -rf "$TEMP_DIR"

echo
print_step "Verifying installation..."
progress_bar 1

if [ $IS_UPDATE -eq 1 ]; then
    print_success "GOA CLI has been successfully updated!"
else
    print_success "GOA CLI has been successfully installed!"
fi
echo
print_info "You can now use the '${BOLD}goa${NC}${CYAN}' command from your terminal."
print_info "For help, run: ${BOLD}goa --help${NC}"
echo
echo -e "${BOLD}${GREEN}Thank you for using Go on Airplanes!${NC}" 