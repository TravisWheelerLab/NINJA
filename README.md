# NINJA
Nearly Infinite Neighbor Joining Application

We use a "git flow" workflow. We have two active branches:
 * **master** is to become the stable NINJA release branch. 
 * **develop** is the NINJA development branch


To clone your own copy of NINJA source code repository for the first time:

```bash
   $ git clone https://github.com/TravisWheelerLab/NINJA
   $ cd NINJA
   $ git checkout develop
```

To contribute to NINJA development, you want to be on the
**develop** branch, which is where we are currently integrating
feature branches. For more information, see the
[NINJA wiki](https://github.com/TravisWheelerLab/NINJA/wiki).

  
      
####AFTER CMAKE CONVERSION 
######To build it: (from root NINJA folder)
```bash
    cd build  
    cmake .. -DCMAKE_BUILD_TYPE=Debug -G "Unix Makefiles"   
    make all  
```
      
######Then to run:   
    cd ..   
    build/src/NINJA_run --in src/fixtures/10.fa --out_type d --corr_type k  

