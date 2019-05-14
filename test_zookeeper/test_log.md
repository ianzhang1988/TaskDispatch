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

## set data

## delete none empty node

## get data

## set and get data