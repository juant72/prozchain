#!/bin/bash

# ProzChain installation script

set -e  # Exit on error

INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/prozchain"
DATA_DIR="/var/lib/prozchain"

# Print banner
echo "======================================="
echo "ProzChain Node Installation"
echo "======================================="
echo

# Check if running as root
if [ "$(id -u)" != "0" ]; then
   echo "This script must be run as root" 
   exit 1
fi

echo "Installing ProzChain to $INSTALL_DIR..."

# Build if needed
if [ ! -f "target/release/prozchain" ]; then
    echo "Building ProzChain from source..."
    cargo build --release
fi

# Create directories
mkdir -p "$CONFIG_DIR"
mkdir -p "$DATA_DIR"

# Copy binary
cp target/release/prozchain "$INSTALL_DIR"
chmod +x "$INSTALL_DIR/prozchain"

# Copy configuration
cp -r config/* "$CONFIG_DIR"

# Create systemd service file
cat > /etc/systemd/system/prozchain.service <<EOF
[Unit]
Description=ProzChain Node
After=network.target

[Service]
User=prozchain
Group=prozchain
WorkingDirectory=$DATA_DIR
ExecStart=$INSTALL_DIR/prozchain --config $CONFIG_DIR/default.toml
Restart=on-failure
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

# Create user if it doesn't exist
if ! id -u prozchain > /dev/null 2>&1; then
    useradd --system --shell /sbin/nologin --home-dir $DATA_DIR prozchain
fi

# Set permissions
chown -R prozchain:prozchain "$CONFIG_DIR"
chown -R prozchain:prozchain "$DATA_DIR"

# Reload systemd, enable and start service
systemctl daemon-reload
systemctl enable prozchain.service

echo
echo "Installation complete!"
echo
echo "To start ProzChain:"
echo "  systemctl start prozchain"
echo
echo "To check status:"
echo "  systemctl status prozchain"
echo
echo "Configuration files are in $CONFIG_DIR"
echo "Data files will be stored in $DATA_DIR"
echo
echo "======================================="
