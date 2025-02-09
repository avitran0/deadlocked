# deadlocked

simple cs2 esp, for linux only

aimbot will be re-made later, this rewrite should fix esp/overlay problems

## setup

- add your user to the `input` group: `sudo usermod -aG input USERNAME` (replace USERNAME with your actual username)
- restart your machine (this will ***not*** work without a restart!)

## running

- clone the repository: `git clone --branch cpp --recursive https://github.com/avitran0/deadlocked`
- run with script: `./run.sh` (need to have a c++ compiler and cmake installed)
