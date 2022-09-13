# plug_and_play_fs

Meant to be a simple way of temporarily exposing the file system of a raspberry pi starting at a root folder. 
Able to upload, download, and move files around freely using a browser instead of requiring ssh and commandline.

My idea is to have a single exe that can be downloaded onto a raspberry pi into any folder wanted. 
When the exe is ran a file explorer looking website will be served exposing that folder (as the root folder) and all its files and subfolders.
Files and folders outside that root folder the exe is in will not be exposed.

There are faster ways of moving files around on raspberry pies. This is mostly an attainable goal while learning rust and rocket.

## Setup for deployment
### Installation
Nightly for rust is needed for rocket, add target for armv7-unknown-linux-gnueabihf, and install cross compiler
```sh
rustup install nightly

rustup target add armv7-unknown-linux-gnueabihf

brew install arm-linux-gnueabihf-binutils
```
### Modify deploy.sh
Edit deploy.sh changing out the address of the target raspberry pi and the folder to put the executable in
### Run Deployment
```sh
./deploy.sh
```

## Starting the server
The address needs to be set as an env var in the process that starts the server. 0.0.0.0 makes the server accessible from other devices as http://[pi_address]:8000. Rocket defaults binding to 127.0.0.1 which is only accessible from the device it is ran on.
```sh
ROCKET_ADDRESS=0.0.0.0 ./plug-and-play-fs
```