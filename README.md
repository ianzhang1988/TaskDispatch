# TaskDispatch
A task dispatch system base on zookeeper.

use zookeeper for coordination, election and "queue"

# Goal Changed 2019-08-12

Originally I want to implement this using rust (as a process leaning rust), but at present, support of zookeeper client rust-zookeeper is not complete ( for example, no reconnect ). So the idea of learning rust and building a dispatch system got in each other's way. Thus I got in between, it's becoming clear that I should not go for two goal at same time. So I put the ideal of building a dispatch system in a new project [PyTaskDispatch](https://github.com/ianzhang1988/PyTaskDispatch.git). This project would try finishing the same idea in experimental way (consider less about error handling) .



