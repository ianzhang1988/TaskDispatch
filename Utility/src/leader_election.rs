// #![deny(unused_mut)]
use zoo_lock::ZkDistrLock;

/// use ZkDistrLock to implement leader election
/// when ZkDistrLock::acquire returns ok, win leadership
/// watch

struct LeadElection {
    cs : ZkDistrLock,

}

mod tests{
    use super::*;

    #[test]
    fn chose_leader_process(){

        /// start multiple process

        let func = ||{
            let mut cs = None;
            let mut le = LeadElection::new(cs);
            let res = le.elect();
            // do leader job

        };
    }
}