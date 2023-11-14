use train_lib::{scheduler, queue};

//TODO: Think about the timing that enqueue the artifact 
// 1. api call after the artifact creation.
// 2. api call after the target number or the total number was changed.
// 3. After the deploy fail. (enter maintainance once a artifact has more than 5 failed). Which may
//    require another crate `worker`, and which sych the status of each instance.
// 4. api call after the artifact delete.

fn main() {
    env_logger::init();
    //TODO:
    // 1. scheduler should read the redis list (from the head), if there is not item read, pending.
    // 2. the api should push the new artifact to the end of the list.
    let queue = queue::Queue::new(queue::DEFAULT_QUEUE_NAME.to_owned());
    loop {
        if let Err(err) = scheduler::process(&queue) {
            log::warn!("Fail to dequeue artifact with the error: {}", err);
        }

        // Once a item(the artifact record id) is read
        // Read the hashset 'artifact:{record_id}:total' to total
        // Read the hashset 'artifact:{record_id}:target' to target
        // Read the hashset 'artifact:{art_id}:instance' to get the instance list
        // Read the hashset 'artifact:{art_id}:instance:{inst_id}' to get the instance info.
        // Iterate each of the instances of 'artifact:{art_id}:instance:{inst_id}' of 'artifact:{art_id}:instance', get the status of these instances
        // Calculate the numbers of instances that is under 'succ'
        // The number to be deploy:
        //  buff = total - ready - in_proc - fail, need = target - ready - in_proc
        // to_deploy = min(buff, need)
        // Deploy '{to_deploy}' copies to tekton by calling rollout method of 'Artifact'

    }
}

