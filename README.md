## Setup
### VM setup
VirtualBox is the preferred virtualizer, although any other virtualizer that can provided shared folders and port forwarding will also work. 
Docker might be considered as a replacement in the future.

**Prerequsites:**
* Insert Guest Additions CD image

```bash
# Initial dependencies
sudo apt-get install curl

# rustup
curl https://sh.rustup.rs -sSf > rustup.sh
chmod +x rustup.sh
./rustup.sh -y
rm rustup.sh


# Add Raspberry Pi 3 arch
sudo dpkg --add-architecture armhf

# Cross-compile dependencies
sudo apt-get install -y build-essential crossbuild-essential-armhf

# Guest additions build dependencies
sudo apt-get install -y module-assistant dkms

# Prepare system to build kernel modules
sudo m-a prepare

# Mount Guest additions CD and build
sudo mount /dev/sr0 /mnt
cd /mnt
sudo ./VBoxLinuxAdditions.run

# Allow current user to access shared folders
sudo adduser "$USER" vboxsf

# Allow ssh user environments
sudo sh -c 'echo "PermitUserEnvironment yes" >> /etc/ssh/sshd_config'

# Reboot
sudo reboot
```

**SSH keys** are **recommended**.

## Cross-compiling
The included Makefile cross-compiles the project on the Linux VM and transfers it onto a target system (ARMv7 hard-float based). 
Environment variables are used for configuration.

### Environment Variables
* `TARGET_ADDRESS`  
	Address of machine that the compiled binary should be deployed on.  
	Example: `pigeon.local`
* `TARGET_BIN_LOCATION`  
	Target location of the deployed binary.  
	Example: `~`
* `TARGET_USER`  
	Username on target system.  
	Example: `philip`
* `VM_PORT`  
	SSH port of cross-compile VM.  
	Example: `2222`
* `VM_PROJECT_LOCATION`  
	Shared project folder location.  
	Example: `/media/sf_pigeond`
* `VM_USER`  
	Username on cross-compile VM.  
	Example: `philip`
* `CONFIGURATION`  
	Defaults to `debug`, set to `release` for optimized builds.  
	Example: `release`

