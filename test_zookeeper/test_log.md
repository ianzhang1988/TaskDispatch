
# Test1

## server config
3 zookeeper server on different server  
> Intel(R) Xeon(R) Gold 6126 CPU @ 2.60GHz  mem 187G  SSD(128G)  
Intel(R) Xeon(R) CPU E5-2620 v3 @ 2.40GHz mem  62G  HHD(2T)  
QEMU Virtual CPU version 2.3.0 (24thread) mem  62G virtual disk

conf/java.env: -Xmx10240m -Xms1024m  
maxClientCnxns=6000  
autopurge.purgeInterval=24  

## test Client
[dependencies]  
zookeeper = "0.5"  
my laptop: i5 7200U 16G    

## test item
### crete empty node
* 1 client create 10000 node take 1.6 ms/node
* 100 client create 10000 node in parallel. avg take 5.198 ms/node; 0.055307 ms/node in parallel
    >55.307s 100*10000 node created

### delete empty node
* 1 client delete 10000 node take 1.3 ms/node
* 100 client delete 10000 node in parallel. avg take 5.224 ms/node; 0.0544 ms/node in parallel
    >54.4s 100*10000 node deleted

### set data
* 100 client set 10000 node in parallel, set 10000B data, avg take 94.0 ms/node; 0.94 ms/node in parallel
    >take 945.0s to finish  
    totally 9.3G data, when data get large, write snapshot severely decrease the speed of zookeeper 
    (lower than 10G less than -Xmx10240m)   
    increase snapCount in zoo.cfg may ease the problem
* 100 client set 10000 node in parallel, set 1000B data, avg take 18.4ms/node; 0.184 ms/node in parallel
    > 184s after last test, with 10_000B data
* 100 client set 10000 node in parallel, set 1000B data, avg take 10.8ms/node; 0.108 ms/node in parallel
    > 113.8s after last test, with 1_000B data
    
### delete none empty node
* 100 client delete 10000 node in parallel. avg take 4.837 ms/node; 0.04837 ms/node in parallel
    >59.06s 100*10000 node deleted

### get data
* 100 client get 10000 node in parallel. avg take 5.103 ms/node to get 1K down; 0.05103 ms/node in parallel
    >55.47s 100*10000 node got

### set and get data
* 100 client random get or set 10000 node in parallel. avg take 8.65 ms/node; 0.08.65 ms/node in parallel
    >90.48s 100*10000 node 
  
## conclusion
Write snapshot would hurt performance, especially when zookeeper hold large data(like 10G),
that is zookeeper is not cut for system which in need of coordinating lots of data.
But of cause One can divide those data and put it on some other storage system, leave a smaller
sub set of 'core' state (or path to some data) in zookeeper.  

  
# Test2
But for easy of use, let's put all of our data in zookeeper,and tune zookeeper a bit,
so that it can hold the data little better. And see if we can find some balance here.

## server config
3 zookeeper server on different server  (same as before)
> Intel(R) Xeon(R) Gold 6126 CPU @ 2.60GHz  mem 187G  SSD(128G)  
Intel(R) Xeon(R) CPU E5-2620 v3 @ 2.40GHz mem  62G  HHD(2T)  
QEMU Virtual CPU version 2.3.0 (24thread) mem  62G virtual disk

conf/java.env: -Xmx20480m -Xms1024m  
maxClientCnxns=6000  
snapCount=5000000 (default 100,000)  
autopurge.snapRetainCount=4  
autopurge.purgeInterval=1

## test Client
[dependencies]  
zookeeper = "0.5"  
server:   
Intel(R) Xeon(R) CPU E5-2650 v2 @ 2.60GHz 64G  

## retry set data, after zookeeper config change
### on my laptop

* 100 client set 10000 node in parallel, set 10000B data, avg take 52.64 ms/node; 0.5264 ms/node in parallel
    >take 529.15s to finish  
    faster than Test1
 
 ### on server
 * 100 client set 10000 node in parallel, set 10000B data, avg take 12.725 ms/node; 0.1272 ms/node in parallel
    >take 150.298 to finish  
    server performance is better, that means my laptop reach it's limits
    
## test item
### crete empty node
* 1000 client create 10000 node in parallel
    >avrage call time 24.78473003003003  
test_create run time 250.299966711s

### set data
* 1000 client set 10000 node in parallel, set 300B data
    > avrage call time 20.49542122122122  
test set data run time 206.434188481s  
avrage call time 19.753284784784785  
test set data run time 199.77771862s  

### get data
* 1000 client get 10000 node in parallel. avg take 5.103 ms/node to get 1K down; 0.05103 ms/node in parallel
    >avrage call time 10.182281981981982  
test get data run time 103.589393736s  

### set and get data
* 1000 client random get or set 10000 node in parallel. avg take 8.65 ms/node; 0.08.65 ms/node in parallel
    >avrage call time 15.760466866866867  
test set and get data run time 158.562917328s  

## conclusion
encounter connection lost, I think It's cause by snapCount being set too big, 
when try generate snapshot can't finish it while so much data be send to sever.  

one prof would be that after zookeeper purge, no connection lost encountered.


# Test3
But for easy of use, let's put all of our data in zookeeper,and tune zookeeper a bit,
so that it can hold the data little better. And see if we can find some balance here.

## server config 
3 zookeeper server on different server  (same as before)
> Intel(R) Xeon(R) Gold 6126 CPU @ 2.60GHz  mem 187G  SSD(128G)  
Intel(R) Xeon(R) CPU E5-2620 v3 @ 2.40GHz mem  62G  HHD(2T)  
QEMU Virtual CPU version 2.3.0 (24thread) mem  62G virtual disk

conf/java.env: -Xmx20480m -Xms1024m  
maxClientCnxns=6000  
snapCount=1000000 (default 100,000) (only change differ from last test)  
autopurge.snapRetainCount=4  
autopurge.purgeInterval=1

## test Client
[dependencies]  
zookeeper = "0.5"  
server:   
Intel(R) Xeon(R) CPU E5-2650 v2 @ 2.60GHz 64G  

## test item

5000 client 10000 node, create take more than 300s,but set data is too slow,
my be due to too match data been write to disk.

so follow test would be change to 5000 client with 2000 node

### crete empty node
* 1000 client create 10000 node in parallel
    >avrage call time 24.78473003003003    
test_create run time 250.299966711s

* 5000 client create 2000 node in parallel
    > avrage call time 124.15642158431686  
test_create run time 262.427422221s  


### set data
* 1000 client set 10000 node in parallel, set 300B data
    > avrage call time 20.49542122122122  
test set data run time 206.434188481s  
* 5000 client set 2000 node in parallel, set 300B data
    >avrage call time 107.53183786757351  
test set data run time 227.344267859s


### get data
* 1000 client get 10000 node in parallel
    >avrage call time 10.182281981981982  
test get data run time 103.589393736s  
* 5000 client get 2000 node in parallel, get 300B data
    >avrage call time 63.20470874174835  
test get data run time 132.583801305s

### set and get data
* 1000 client random get or set 10000 node in parallel
    >avrage call time 18.173901701701702
test set and get data run time 183.517155866s
* 5000 client random get or set 2000 node in parallel
    >avrage call time 94.30373564712943  
test set and get data run time 192.516689336s
    
### delete none empty node
* 1000 client delete 10000 node in parallel
    >59.06s 100*10000 node deleted
    
## conclusion
There still are connection lost error, so snapCount is not the cause.
all test show similar data transfer rate. with smaller snapCount, disk is eaten up
quite fast.

Take zookeeper's transaction log under consideration, It's not for the scenario which
data rapidly changing. Because every change would be recorded in log, and write to
disk, that is not very efficient, and would consume lots of disk space. read on the
other hand not limited by disk.

At 5000 client and snapCount equal 1000000, that is 200 op from 1 clint would
 generate a snapshot. Maybe increase snapCount is needed after all.