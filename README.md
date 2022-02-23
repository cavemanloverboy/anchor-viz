# anchorviz
A color-coded visualization tool for the instructions of an anchor program. 


![basic_2](https://user-images.githubusercontent.com/93507302/155232816-7c3b34a7-9a89-4d38-abd7-79e302c91d2c.png)
(This is a schematic of `basic-2` from anchor's `examples/tutorial` directory)
# Installation
Via cargo:
```bash
cargo install anchor-viz
```

From source:

To install, run
```bash
git clone https://github.com/cavemanloverboy/anchorviz
cd anchorviz
make
```
to build the executable. On linux/mac, you can run 
```bash
git clone https://github.com/cavemanloverboy/anchorviz
cd anchorviz
make linux-mac
```
to build the executable and then copy it to `/usr/local/bin/` to link the executable (assuming /usr/local/bin is in your `PATH`).

# Usage
To use anchorviz, run `anchorviz` in the root directory of an anchor project or in a program directory. For example,
```bash
anchor init my_project
cd my_project # or cd my_project/programs/my_project
anchorviz
```
Otherwise, specify the program name via `anchorviz -p my_program` while in the root of the anchor project.
