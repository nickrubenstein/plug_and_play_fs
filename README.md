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
On the device it has been deployed to, give the exe (and only the exe) access to run on port 443 without elevated privileges and then run the executable created from deploy.sh. https://superuser.com/questions/710253/allow-non-root-process-to-bind-to-port-80-and-443
```sh
sudo setcap CAP_NET_BIND_SERVICE=+eip /home/pi/plug-and-play-fs

./plug-and-play-fs
```

By default the exposed root folder is the working directory of the plug-and-play-fs.exe. To change that, set the FS_ROOT_FOLDER environment variable to the name of a folder in the same folder as the exe.

## Developing
```
cargo run
```
Starts HTTPS server at https://localhost:8000 using the ```key.pem``` in the top-level ```private``` folder. 
<br><br>
```NOTE:``` To access the website with chrome receiving a ```NET::ERR_CERT_INVALID``` because you generated your own ```key.pem``` click anywhere on the error page and type "thisisunsafe". This is ok because we know we generated the ```key.pem``` ourself.

## TODO Logging in
Currently, only one user is hard coded in. It can be changed in the test function in src > models > user.rs
```
Username: nick
Password: testing
```

## TODO consolidate static into one exe
Currently, there is a static folder that must be passed with the exe. It would be nice to precompile its contents for the exe

