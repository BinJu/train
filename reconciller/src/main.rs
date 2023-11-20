use std::thread::{self, JoinHandle};

use train_lib::{artifact::{dao::{ArtifactDao, InstanceDao, DEFAULT_REDIS_URL}, ArtifactStatus, instance::InstanceStatus}, error, *};

//TODO: Reconciller's main tasks:
//1. Scan all the artifacts that are under status:
//   Fail/PendingArtRef/PendingAccount
//   Please note that we should protect the system by:
//   a. Not schedule the failed artfact too frequently. We could tell that easily by reading
//   the last_sched field.
//   b. Not schedule the artifact that has too many failed instances. Like 20%?
//2. Sync up the status of the instances by reading the Tekton PipelineRun/TaskRun. And
//   populate the 'results' back to the Instance.
//3. Clean up Tekton resources. Such as the finished TaskRun/PipelineRun, secrets, Pipeline/Task,
//   etc.
fn main() {
    env_logger::init();
    let instances = run_task("sync instances", 1, sync_instances);
    let reschedules = run_task("artifact reschedule", 1, reschedule_artifacts);
    instances.join().expect("Could't join on the instance synch thread");
    reschedules.join().expect("Could not join the reschedule thread");
}

fn sync_instances() -> error::Result<()> {
    sync_instances_with_stats_callback(|run_name, namespace| {
        let stats = artifact::pipeline::pipeline_run_stats(run_name, namespace)?;
        Ok(stats)
    })
}

fn sync_instances_with_stats_callback(callback: impl Clone+FnOnce(&str,&str)->error::Result<String>) -> error::Result<()> {
    // Load artifacts
    log::info!("Start synchronizing tekton run status");
    let mut conn = redis::Client::open(DEFAULT_REDIS_URL)?.get_connection()?;
    let art_ids = ArtifactDao::all_ids(&mut conn)?;
    for id in art_ids {
        log::info!("Querying instances ...");
        let instances = InstanceDao::many(&id, &mut conn)?;
        let mut build_statuses = Vec::new();
        let mut clean_statuses = Vec::new();
        log::info!("Instances Query done.");
        for mut inst in instances {
            let run_name = &inst.run_name;
            log::info!("Checking status of tekton run: {}.", run_name);
            let callback_fn = callback.clone();
            let stat = callback_fn(run_name, artifact::DEFAULT_NAMESPACE)?;
            log::info!("Status of tekton run: {} is {}.", run_name, stat);
            inst.stat = stat.into();
            if run_name.starts_with("build-") {
                build_statuses.push(inst.stat.clone());
            } else {
                clean_statuses.push(inst.stat.clone());
            }
            InstanceDao::update(inst, &mut conn)?;
            log::info!("Status updated");
        }
        let build_failed = build_statuses.iter().any(|v| v.is_failed());
        let build_succeeded = build_statuses.iter().all(|v| *v == InstanceStatus::Succeeded);
        if build_failed {
            ArtifactDao::update_rollout_status(&id, "build", ArtifactStatus::Failed, &mut conn)?;
        } else if build_succeeded && build_statuses.len() > 0 {
            ArtifactDao::update_rollout_status(&id, "build", ArtifactStatus::Succeeded, &mut conn)?;
        };

        let clean_failed = clean_statuses.iter().any(|v| v.is_failed());
        let clean_succeeded = clean_statuses.iter().all(|v| *v == InstanceStatus::Succeeded);
        if clean_failed {
            ArtifactDao::update_rollout_status(&id, "clean", ArtifactStatus::Failed, &mut conn)?;
        } else if clean_succeeded && clean_statuses.len() > 0 {
            ArtifactDao::update_rollout_status(&id, "clean", ArtifactStatus::Succeeded, &mut conn)?;
        };
    }
    log::info!("synchronization is done.");
    Ok(())
    // Then load instances, with the 'run_name' we could query the TaskRun/PipelineRun status. If
    // it is done, we should populate the 'results'.
    //
    //TODO: handle the results if it is succeeded.

}

// Load  artifacts once it is failed/pendingArtRef/pendingAccount, We should give it another
// change to reschedule. But we should avoid to reschedule the same artifact to frequently. And
// if the failure rate is too high ( Like >= 20% ), We should never reschedule it until the
// manifest of the artifact is updated.
fn reschedule_artifacts() -> error::Result<()> {
    Ok(())
}

fn run_task<T>(name: &str, interval_secs: u64, task: T) -> JoinHandle<()>
where T: 'static + FnOnce()-> error::Result<()> + Sync + Send + Clone{
    let task_name = name.to_owned();
    thread::spawn( move ||{
        loop {
            let task_func = task.clone();
            let result = task_func();
            if let Err(err) = result {
                log::warn!("Fail to run the task {} with the error: {}", task_name.clone(), err);
            }

            thread::sleep(std::time::Duration::from_secs(interval_secs));
        }
    })

}

#[cfg(test)]
mod tests {
    use train_lib::{artifact::{Artifact, ArtifactRequest, pipeline, dao::{self, ArtifactDao, InstanceDao}, instance::InstanceStatus}, scheduler, queue::Queue};
    use serde_json;

    use crate::sync_instances_with_stats_callback;
    #[test]
    fn test_sync_instances() {
        pipeline::delete_run("--all", "train").unwrap();
        let art_id = "recon-sample";
        let request = r#"{"name":"recon-sample","total":1,"target":1,"refs":[{"name":"mock"}],"build":{"tasks":[{"name":"arttest-task1","spec":{"steps":[{"name":"step1","image":"ubuntu","script":"echo $(params.name)\necho with art_id:"}],"params":[{"name":"name","type":"string","description":"The username"}]},"paramValues":[{"name":"name","value":"John"}]}],"params":[{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"secrets":[{"name":"aws-route53"},{"name":"pivnet"}],"accounts":[{"name":"gcp-environment"}]},"clean":{"tasks":[{"name":"task1","spec":{"steps":[{"name":"step1","image":"ubuntu","script":"echo $(params.name)\necho with art_id:"}],"params":[{"name":"name","type":"string","description":"The username"}]},"paramValues":[{"name":"name","value":"John"}]}],"params":[{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"secrets":[{"name":"aws_route53"},{"name":"pivnet"}],"accounts":[{"name":"gcp_environment"}]}}"#;

        let mut conn = dao::connection(dao::DEFAULT_REDIS_URL).expect(&format!("Failed to establish connection to redis server at: {}", dao::DEFAULT_REDIS_URL));
        ArtifactDao::delete(art_id, &mut conn).unwrap();
        let instances = InstanceDao::many(art_id, &mut conn).unwrap();
        for inst in instances {
            InstanceDao::delete(&inst.id, art_id, &mut conn).unwrap();
        }
        let mut request: ArtifactRequest = serde_json::from_str(request).expect("Failed to deserialize the payload to ArtifactRequest object");
        request.format().expect("Failed to format the artifact request");

        let artifact = Artifact::try_from(request).expect("Failed to deserialize artifact from artifact request");
        ArtifactDao::save(artifact, &mut conn).unwrap();
        //schedule the artifact by calling schedule.process
        //let instances = artifact.build.run(1).expect("Failed to roll out the artifct");
        let queue = Queue::new("reconciller-test".to_owned());
        queue.reset(&mut conn).unwrap();
        queue.enqueue(art_id, &mut conn).unwrap();

        let instance_ids = scheduler::process(&queue).unwrap();
        sync_instances_with_stats_callback(|_run_name, _ns| Ok("Succeeded".to_owned())).unwrap();

        let instance = InstanceDao::one(&instance_ids[0], "recon-sample", &mut conn).unwrap();
        assert_eq!(instance.stat, InstanceStatus::Succeeded);
    }
}
