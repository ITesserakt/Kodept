#!/bin/bash
rm -r build
rm -r rls
mkdir build
cd build
cmake ../ -DCOMSDK_INST=/home/semyon/installed_comsdk;
make
