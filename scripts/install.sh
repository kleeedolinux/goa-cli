#!/bin/bash

# GOA CLI Linux Installer
# This script downloads and installs the GOA CLI from the mirror service

echo "GOA CLI Installer for Linux"
echo "=========================="
echo

# Define installation directory
INSTALL_DIR="/usr/local/bin"
if [ ! -d "$INSTALL_DIR" ]; then
    echo "Creating installation directory..."
    sudo mkdir -p "$INSTALL_DIR"
fi

# Check if curl is installed
if ! command -v curl &> /dev/null; then
    # Try wget as an alternative
    if ! command -v wget &> /dev/null; then
        echo "Error: Either curl or wget is required but neither is installed."
        echo "Please install curl or wget and try again."
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
echo "Downloading GOA CLI from mirror service..."
TMP_FILE=$(mktemp)

# Try primary URL
if [ $USE_WGET -eq 1 ]; then
    # Using wget with automatic redirect handling
    wget -q --show-progress --user-agent="GOA-CLI-Installer" "$DOWNLOAD_URL" -O "$TMP_FILE"
    DOWNLOAD_STATUS=$?
else
    # Using curl with automatic redirect handling (-L flag)
    curl -L --user-agent "GOA-CLI-Installer" -# "$DOWNLOAD_URL" -o "$TMP_FILE"
    DOWNLOAD_STATUS=$?
fi

# If primary download fails, try fallback URL
if [ $DOWNLOAD_STATUS -ne 0 ] || [ ! -s "$TMP_FILE" ]; then
    echo "Primary download failed, trying fallback method..."
    
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
    echo "Error: Failed to download the GOA CLI binary."
    rm -f "$TMP_FILE"
    exit 1
fi

# Check if file is empty
if [ ! -s "$TMP_FILE" ]; then
    echo "Error: Download completed but the file is empty."
    rm -f "$TMP_FILE"
    exit 1
fi

# Make the binary executable
chmod +x "$TMP_FILE"

# Move to installation directory
echo "Installing GOA CLI to $INSTALL_DIR/goa..."
sudo mv "$TMP_FILE" "$INSTALL_DIR/goa"

if [ $? -ne 0 ]; then
    echo "Error: Failed to install the GOA CLI binary. Do you have permission to write to $INSTALL_DIR?"
    exit 1
fi

echo
echo "GOA CLI has been successfully installed!"
echo "You can now use the 'goa' command from your terminal."
echo "For help, run: goa --help"
echo
echo "Thank you for using Go on Airplanes!" 
