apt update
apt install -y python3 python3-dev python3-pip

export PYO3_PYTHON=/usr/bin/python3

# Install Python dependencies globally
pip3 install --break-system-packages pypdfform>=3.6.3
