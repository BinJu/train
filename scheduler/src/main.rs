const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1";

use train_lib::{artifact::{instance::{Instance, InstanceNumbers}, artifact_dao::ArtifactDao, instance_dao::InstanceDao, Artifact, ArtifactStatus}, error::{self, GeneralError}, *};

//TODO: Think about the timing that enqueue the artifact 
// 1. api call after the artifact creation.
// 2. api call after the target number or the total number was changed.
// 3. After the deploy fail. (enter maintainance once a artifact has more than 5 failed). Which may
//    require another crate `worker`, and which sych the status of each instance.
// 4. api call after the artifact delete.

const DEFAULT_QUEUE_NAME: &str = "train-artifact-01";
fn main() {
    env_logger::init();
    //TODO:
    // 1. scheduler should read the redis list (from the head), if there is not item read, pending.
    // 2. the api should push the new artifact to the end of the list.
    let mut conn = artifact::artifact_dao::connection(DEFAULT_REDIS_URL).expect(&format!("Failed to establish connection to redis server at: {}", DEFAULT_REDIS_URL));
    let queue = queue::Queue::new( DEFAULT_QUEUE_NAME.to_owned());
    loop {
        if let Err(err) = process(&queue, &mut conn) {
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

fn process(queue: &queue::Queue, mut conn: &mut redis::Connection) -> error::Result<()> {
        // Block on reading the head of the list.
    let art_id = queue.block_dequeue(0, conn)?;
    let mut artifact = ArtifactDao::one(&art_id, &mut conn)?;
    let instances = InstanceDao::many(&art_id, &mut conn)?;
    let numbers = statistic_instances(&instances)?;
    let to_deploy = numbers_to_deploy(&artifact, &numbers);
    let rollout_result = rollout_artifact(&mut artifact, to_deploy);
    update_artifact(artifact, to_deploy, rollout_result)?;
    Ok(())
}

//Return instance numbers that are in running, error
fn statistic_instances(_instances: &[Instance]) -> error::Result<InstanceNumbers> {
    Err(error::error("unimplemented yet!"))
}

fn numbers_to_deploy(_artifact: &Artifact, _numbers: &InstanceNumbers) -> i32 {
    0
}

fn rollout_artifact(artifact: &mut Artifact, number: i32) -> error::Result<Vec<Instance>> {
    if number > 0 {
        artifact.build.run(number)
    } else if number < 0 {
        artifact.clean.run(-number)
    } else {
        Ok(Vec::new())
    }
}

fn update_artifact(mut artifact: Artifact, to_deploy: i32, rollout_result: error::Result<Vec<Instance>>) -> error::Result<()> {
    if to_deploy == 0 { return Ok(()) }

    match rollout_result {
        Ok(instances) => update_instances(instances)?,
        Err(err) => {
            let mut rollout = if to_deploy > 0{
                &mut artifact.build
            } else {
                &mut artifact.clean
            };

            match err {
                error::GeneralError::PendingArtRef => rollout.stats = ArtifactStatus::PendingArtRef,
                error::GeneralError::PendingAccount => rollout.stats = ArtifactStatus::PendingAccount,
                _ => rollout.stats = ArtifactStatus::Fail
            };
        }
    }


    Ok(())
}

fn update_instances(_instances: Vec<Instance>) -> error::Result<()> {
    Ok(())
}
