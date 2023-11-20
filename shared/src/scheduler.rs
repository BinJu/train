use crate::artifact::{instance::{Instance, InstanceNumbers, InstanceStatus}, dao::{self, ArtifactDao, InstanceDao, DEFAULT_REDIS_URL}, Artifact};
use crate::queue;
use crate::error;

pub fn process(queue: &queue::Queue) -> error::Result<Vec<String>> {
        // Block on reading the head of the list.
    let mut conn = dao::connection(DEFAULT_REDIS_URL).expect(&format!("Failed to establish connection to redis server at: {}", DEFAULT_REDIS_URL));
    let art_id = queue.block_dequeue(0, &mut conn)?;
    log::info!("Dequeuing the artifact: {} ", art_id);
    let mut artifact = ArtifactDao::one(&art_id, &mut conn)?;
    let instances = InstanceDao::many(&art_id, &mut conn)?;
    let numbers = statistic_instances(&instances)?;
    let to_deploy = numbers_to_deploy(&artifact, &numbers);
    log::info!("{} environments are await to deploy ", to_deploy);
    let rollout_result = rollout_artifact(&mut artifact, to_deploy);
    let (rollout_ok, rollout_err) = multiplex_result(rollout_result);
    //update_artifact(artifact, to_deploy, rollout_result)?;

    if to_deploy > 0 {
        ArtifactDao::update_build_status(&mut artifact, &rollout_err, &mut conn)?;
    } else {
        ArtifactDao::update_clean_status(&mut artifact, &rollout_err, &mut conn)?;
    };

    let mut result = Vec::new();
    if let Some(instances) = rollout_ok {
        for inst in instances {
            result.push(inst.id.clone());
            InstanceDao::save(inst, &mut conn)?;
        }
    }
    if let Some(err) = rollout_err {
        log::warn!("Failed to rollout the deploy: {} ", err);
    }

    Ok(result)
}

fn multiplex_result(r: error::Result<Vec<Instance>>) -> (Option<Vec<Instance>>, Option<error::GeneralError>) {
    match r {
        Ok(instances) => (Some(instances), None),
        Err(err) => (None, Some(err))
    }
}

//Return instance numbers that are in running, error
fn statistic_instances(instances: &[Instance]) -> error::Result<InstanceNumbers> {
    let mut running = 0u32;
    let mut fail = 0u32;
    let mut done_clean = 0u32;
    let mut done_dirt = 0u32;
    for inst in instances {
        match inst.stat {
            InstanceStatus::Running => running += 1,
            InstanceStatus::Failed(_) => fail += 1,
            InstanceStatus::Succeeded=> if !inst.dirt {done_clean += 1} else {done_dirt += 1},
            _ => {}
        }
    }

    Ok(InstanceNumbers{
        running,
        fail,
        done_clean,
        done_dirt
    })
}

fn numbers_to_deploy(artifact: &Artifact, numbers: &InstanceNumbers) -> i32 {
    // Calculate the numbers of instances that is under 'succ'
    // The number to be deploy:
    //  buff = total - ready - in_proc - fail, need = target - ready - in_proc
    // to_deploy = min(buff, need)
    let buff_number = artifact.total as i32 - numbers.done_dirt as i32 - numbers.done_clean as i32 - numbers.fail as i32 - numbers.running as i32;
    let need = artifact.target as i32 - numbers.done_clean as i32 - numbers.running as i32;
    std::cmp::min(buff_number, need)
}

fn rollout_artifact(artifact: &mut Artifact, number: i32) -> error::Result<Vec<Instance>> {
    if number > 0 {
        artifact.build.run(number)
    } else if number < 0 {
        artifact.clean.run(number)
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::{pipeline, ArtifactRequest, ArtifactStatus, dao::{self, ArtifactDao, InstanceDao}};

    #[test]
    fn test_process() {
        let request = r#"{"name":"opsman-process","total":1,"target":1,"refs":[{"name":"mock"}],"build":{"tasks":[{"name":"opsman-task1","spec":{"steps":[{"name":"step-collectdata","image":"ubuntu","script":"echo $(params.name)\necho with inputs: art_id: $(params.art_id)\tinst_id: $(params.inst_id) Done\nls -l /var\necho secret aws-route53\nls -l /var/aws-route53\ncat /var/aws-route53/user_id\ncat var/aws-route53/secret\necho secret pivnet\nls -l /var/pivnet\necho account gcp-environment\nls -l /var/gcp-environment","volumeMounts":[{"name":"aws-route53","mountPath":"/var/aws-route53"},{"name":"pivnet","mountPath":"/var/pivnet"},{"name":"gcp-environment","mountPath":"/var/gcp-environment"}]}],"params":[{"name":"name","type":"string","description":"The username"},{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"volumes":[{"name":"aws-route53","secret":{"secretName":"sec-$(params.art_id)-aws-route53"}},{"name":"pivnet","secret":{"secretName":"sec-$(params.art_id)-pivnet"}},{"name":"gcp-environment","secret":{"secretName":"acnt-$(params.art_id)-$(params.inst_id)-gcp-environment"}}]},"paramValues":[{"name":"name","value":"John"},{"name":"art_id","value":"$(params.art_id)"},{"name":"inst_id","value":"$(params.inst_id)"}]}],"params":[{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"secrets":[{"name":"aws-route53"},{"name":"pivnet"}],"accounts":[{"name":"gcp-environment"}]},"clean":{"tasks":[{"name":"task1","spec":{"steps":[{"name":"collect-data","image":"ubuntu","script":"echo $(params.name)\necho with art_id:"}],"params":[{"name":"name","type":"string","description":"The username"}]},"paramValues":[{"name":"name","value":"John"}]}],"params":[{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"secrets":[{"name":"aws-route53"},{"name":"pivnet"}],"accounts":[{"name":"gcp-environment"}]}}"#;
        let mut artifact_request: ArtifactRequest = serde_json::from_str(request).expect("failed to deserialize the rollout request");
        artifact_request.validate().expect("Failed to validate the request");
        artifact_request.format().expect("Failed to format the artifact request");
        let artifact = Artifact::try_from(artifact_request).expect("failed to deserialize the request to artifact");
        let mut conn = dao::connection(DEFAULT_REDIS_URL).expect(&format!("Failed to establish connection to redis server at: {}", DEFAULT_REDIS_URL));
        let instances = InstanceDao::many(&artifact.id, &mut conn).expect(&format!("Failed to query instances of artifact {}", artifact.id));
        for inst in instances {
            let _result = pipeline::delete_run(&inst.run_name, "train");
            InstanceDao::delete(&inst.id, &artifact.id, &mut conn).expect(&format!("Failed to delete instance {}", inst.id));
        }
        ArtifactDao::delete("opsman-process", &mut conn).expect("Failed to delete artifact:opsman-lib from DB");
        let art_id = artifact.id.clone();
        ArtifactDao::save(artifact, &mut conn).expect("Failed to save artifact");
        let queue = queue::Queue::new(queue::DEFAULT_QUEUE_NAME.to_owned());
        let mut conn = dao::connection(DEFAULT_REDIS_URL).expect(&format!("Failed to establish connection to redis server at: {}", DEFAULT_REDIS_URL));
        queue.reset(&mut conn).unwrap();
        queue.enqueue(&art_id, &mut conn).unwrap();

        let result = process(&queue);
        if let Err(err) = result {
            panic!("Failed to process the artifact, with error: {}", err);
        }
        let artifact = ArtifactDao::one(&art_id, &mut conn).unwrap();
        assert_eq!(artifact.build.stats, ArtifactStatus::Running);
        assert!(chrono::Local::now() - artifact.build.last_sched < chrono::Duration::seconds(5));
        let instances = result.unwrap();
        assert!(instances.len() > 0);

        println!("############## instance art name: {} id: {}", art_id, instances[0]);
        let instance = InstanceDao::one(&instances[0], &art_id, &mut conn).unwrap();
        assert_eq!(instance.stat, InstanceStatus::Running);

    }

    #[test]
    fn test_numbers_to_deploy_simple() {
        let artifact = Artifact::new("art-number-to-dep-test", 1, 1);
        let numbers = InstanceNumbers {
            running: 0,
            fail: 0,
            done_clean: 0,
            done_dirt: 0
        };

        let num = numbers_to_deploy(&artifact, &numbers);
        assert_eq!(num, 1);
    }

    #[test]
    fn test_numbers_to_deploy_build() {
        let artifact = Artifact::new("art-number-to-dep-test", 5, 2);
        let numbers = InstanceNumbers {
            running: 1,
            fail: 1,
            done_clean: 0,
            done_dirt: 1
        };

        let num = numbers_to_deploy(&artifact, &numbers);
        assert_eq!(num, 1);
    }

    #[test]
    fn test_numbers_to_deploy_clean() {
        let artifact = Artifact::new("art-number-to-dep-test", 5, 2);
        let numbers = InstanceNumbers {
            running: 1,
            fail: 1,
            done_clean: 2,
            done_dirt: 1
        };

        let num = numbers_to_deploy(&artifact, &numbers);
        assert_eq!(num, -1);
    }

    #[test]
    fn test_numbers_to_deploy_capped() {
        let artifact = Artifact::new("art-number-to-dep-test", 4, 2);
        let numbers = InstanceNumbers {
            running: 0,
            fail: 1,
            done_clean: 0,
            done_dirt: 2
        };

        let num = numbers_to_deploy(&artifact, &numbers);
        assert_eq!(num, 1);
    }

    fn instance(is_dirt: bool, stat: InstanceStatus) -> Instance {
        Instance {
            id: "inst-1".to_owned(),
            art_id: "art-1".to_owned(),
            run_name: "".to_owned(),
            dirt: is_dirt,
            stat,
            results: None
        }
    }
    #[test]
    fn test_statistic_instances() {
        let instances = vec![
            instance(false, InstanceStatus::Running),
            instance(false, InstanceStatus::Failed(String::new())),
            instance(false, InstanceStatus::Failed(String::new())),
            instance(false, InstanceStatus::Succeeded),
            instance(false, InstanceStatus::Succeeded),
            instance(false, InstanceStatus::Succeeded),
            instance(true, InstanceStatus::Succeeded),
            instance(true, InstanceStatus::Succeeded),
            instance(true, InstanceStatus::Succeeded),
            instance(true, InstanceStatus::Succeeded),
        ];
        let stats = statistic_instances(&instances).unwrap();
        assert_eq!(stats.running, 1);
        assert_eq!(stats.fail, 2);
        assert_eq!(stats.done_clean, 3);
        assert_eq!(stats.done_dirt, 4);
    }
}
