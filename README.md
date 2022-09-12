# plug_and_play_fs

Meant to be a simple way of temporarily exposing the file system of a raspberry pi starting at a root folder. 
Able to upload, download, and move files around freely using a browser instead of requiring ssh and commandline.

My idea is to have a single exe that can be downloaded onto a raspberry pi into any folder wanted. 
When the exe is ran a file explorer looking website will be served exposing that folder (as the root folder) and all its files and subfolders.
Files and folders outside that root folder the exe is in will not be exposed.
