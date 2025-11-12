git clone https://github.com/devnull03/ufv-timesheet-util.git services/ufv-timesheet-util
rm -rf services/ufv-timesheet-util/.git  # Remove git directory to avoid conflicts

apt update
apt install -y python3 python3-dev python3-pip

export PYO3_PYTHON=/usr/bin/python3

# Install Python dependencies globally
pip3 install --break-system-packages pypdfform>=3.6.3

