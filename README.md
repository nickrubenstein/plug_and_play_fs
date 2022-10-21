# plug_and_play_fs

Meant to be a simple way of temporarily exposing the file system of a raspberry pi starting at a root folder. 
Able to upload, download, zip, unzip, and move files around freely using a browser instead of requiring ssh and commandline.

My idea is to have a an exe that can be downloaded onto a raspberry pi into any folder wanted. 
When the exe is ran, a file explorer looking website will be served exposing that folder (as the root folder) and all its files and subfolders.
Files and folders outside the root folder the exe is in will not be exposed.

There are faster ways of moving files around on raspberry pis. In my particular case there are a lot of folders with long names and spaces which are very annoying in commandline. This is mostly a simple and attainable goal while learning rust.

## Setup for deployment
### Installation
Nightly for rust is needed, add target for armv7-unknown-linux-gnueabihf, and install the cross compiler for a raspberry pi 2/3/4
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
On the device it has been deployed to, simply run the executable created from deploy.sh.
```sh
./plug-and-play-fs
```

## TODO Logging in
Currently, only one user is hard coded in. It can be changed in the test function in src > models > user.rs
```
Username: nick
Password: testing
```