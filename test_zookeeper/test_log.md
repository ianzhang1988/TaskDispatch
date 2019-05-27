# server config
3 zookeeper server on different server  
> Intel(R) Xeon(R) Gold 6126 CPU @ 2.60GHz  mem 187G  SSD(128G)  
Intel(R) Xeon(R) CPU E5-2620 v3 @ 2.40GHz mem  62G  HHD(2T)  
QEMU Virtual CPU version 2.3.0 (24thread) mem  62G virtual disk


-Xmx10240m -Xms1024m  
maxClientCnxns=6000  
autopurge.purgeInterval=24  

# test Client
[dependencies]  
zookeeper = "0.5"  

my laptop: i5 7200U 16G  
server:  

# Test
## crete empty node
on my laptop  
* 1 client create 10000 node take 1.6 ms/node
* 100 client create 10000 node in parallel. avg take 5.198 ms/node; 0.055307 ms/node in parallel
    >55.307s 100*10000 node created

## delete empty node
on my laptop  
* 1 client delete 10000 node take 1.3 ms/node
* 100 client delete 10000 node in parallel. avg take 5.224 ms/node; 0.0544 ms/node in parallel
    >54.4s 100*10000 node deleted

## set data
on my laptop
* 100 client set 10000 node in parallel, set 10000B data, avg take 94.0 ms/node; 0.94 ms/node in parallel
    >take 945.0s to finish  
    totally 9.3G data, when data get large, write snapshot severely decrease the speed of zookeeper 
    (lower than 10G less than -Xmx10240m)   
    increase snapCount in zoo.cfg may ease the problem
* 100 client set 10000 node in parallel, set 1000B data, avg take 18.4ms/node; 0.184 ms/node in parallel
    > 184s after last test, with 10_000B data
* 100 client set 10000 node in parallel, set 1000B data, avg take 10.8ms/node; 0.108 ms/node in parallel
    > 113.8s after last test, with 1_000B data
    
## delete none empty node
on my laptop
* 100 client delete 10000 node in parallel. avg take 4.837 ms/node; 0.04837 ms/node in parallel
    >59.06s 100*10000 node deleted

## get data

## set and get data