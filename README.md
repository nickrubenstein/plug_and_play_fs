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
Add cert and key for TLS in the folder named *private* each named cert.pem and key.pem.
To create self signed files for testing run the following:
```
openssl req -x509 -newkey rsa:4096 -nodes -keyout private/key.pem -out private/cert.pem -days 365 -subj '/CN=plug_and_play_fs_dev'
````
Edit deploy.sh changing out the address of the target raspberry pi and the folder to put the executable in. Taken from this medium article. <https://medium.com/swlh/compiling-rust-for-raspberry-pi-arm-922b55dbb050>
### Run Deployment
```sh
./deploy.sh
```

## Starting the server
On the device it has been deployed to simply run the executable created from deploy.sh.
```sh
./plug-and-play-fs
```