#!/bin/bash

# GOA CLI Universal Installer
# This script detects the OS and runs the appropriate installation method

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

# Detect the operating system
detect_os() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macOS"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "Linux"
    else
        echo "Unknown"
    fi
}

# Display a spinner during operations
spinner() {
    local pid=$1
    local delay=0.1
    local spinstr='|/-\'
    while [ "$(ps a | awk '{print $1}' | grep $pid)" ]; do
        local temp=${spinstr#?}
        printf " [%c]  " "$spinstr"
        local spinstr=$temp${spinstr%"$temp"}
        sleep $delay
        printf "\b\b\b\b\b\b"
    done
    printf "    \b\b\b\b"
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
    echo -e "${BOLD}${CYAN}Universal Installer${NC}\n"
}

# Main script starts here
clear
display_banner

OS=$(detect_os)
print_info "Detected operating system: ${BOLD}$OS${NC}"
echo

# For macOS, use the macuser.sh script
if [[ "$OS" == "macOS" ]]; then
    print_step "Running macOS-specific installation..."
    
    # Check if the macuser.sh script exists locally
    if [ -f "$(dirname "$0")/macuser.sh" ]; then
        bash "$(dirname "$0")/macuser.sh"
        exit $?
    else
        # Download and run the macuser.sh script
        print_info "Downloading macOS installer script..."
        curl -sSL https://raw.githubusercontent.com/kleeedolinux/goa-cli/master/scripts/macuser.sh | bash
        exit $?
    fi
fi

# Continue with Linux installation
if [[ "$OS" == "Linux" ]]; then
    print_step "Running Linux installation..."
    
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
    
    if [ ! -d "$INSTALL_DIR" ]; then
        print_step "Creating installation directory..."
        sudo mkdir -p "$INSTALL_DIR"
    fi

    # Check if curl is installed
    if ! command -v curl &> /dev/null; then
        # Try wget as an alternative
        if ! command -v wget &> /dev/null; then
            print_error "Either curl or wget is required but neither is installed."
            print_error "Please install curl or wget and try again."
            exit 1
        fi
        USE_WGET=1
    else
        USE_WGET=0
    fi

    # Primary download URL with HTTPS
    DOWNLOAD_URL="https://re.juliaklee.wtf/linux"
    # Fallback URL as direct GitHub download
    FALLBACK_URL="https://raw.githubusercontent.com/goonairplanes/goa-cli/main/dist/goa-linux"

    # Download the binary
    print_step "Downloading GOA CLI from mirror service..."
    TMP_FILE=$(mktemp)

    # Try primary URL
    if [ $USE_WGET -eq 1 ]; then
        print_info "Using wget to download binary..."
        wget -q --show-progress --user-agent="GOA-CLI-Installer" "$DOWNLOAD_URL" -O "$TMP_FILE"
        DOWNLOAD_STATUS=$?
    else
        print_info "Using curl to download binary..."
        curl -L --user-agent "GOA-CLI-Installer" -# "$DOWNLOAD_URL" -o "$TMP_FILE"
        DOWNLOAD_STATUS=$?
    fi

    # If primary download fails, try fallback URL
    if [ $DOWNLOAD_STATUS -ne 0 ] || [ ! -s "$TMP_FILE" ]; then
        print_warning "Primary download failed, trying fallback method..."
        
        if [ $USE_WGET -eq 1 ]; then
            wget -q --show-progress --user-agent="GOA-CLI-Installer" "$FALLBACK_URL" -O "$TMP_FILE"
            DOWNLOAD_STATUS=$?
        else
            curl -L --user-agent "GOA-CLI-Installer" -# "$FALLBACK_URL" -o "$TMP_FILE"
            DOWNLOAD_STATUS=$?
        fi
    fi

    # Final check
    if [ $DOWNLOAD_STATUS -ne 0 ]; then
        print_error "Failed to download the GOA CLI binary."
        rm -f "$TMP_FILE"
        exit 1
    fi

    # Check if file is empty
    if [ ! -s "$TMP_FILE" ]; then
        print_error "Download completed but the file is empty."
        rm -f "$TMP_FILE"
        exit 1
    fi

    # Make the binary executable
    print_step "Setting up permissions..."
    chmod +x "$TMP_FILE"

    # Move to installation directory
    print_step "Installing GOA CLI to $BINARY_PATH..."
    sudo mv "$TMP_FILE" "$BINARY_PATH"

    if [ $? -ne 0 ]; then
        print_error "Failed to install the GOA CLI binary. Do you have permission to write to $INSTALL_DIR?"
        exit 1
    fi

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
    exit 0
fi

# If we get here, it's an unsupported OS
if [[ "$OS" == "Unknown" ]]; then
    print_error "Unsupported operating system."
    print_error "This installer supports Linux and macOS only."
    print_warning "For Windows, please use the PowerShell installer (install.ps1)."
    exit 1
fi 
