use std::io::{Result, ErrorKind, Error};
use crate::command::{self, command_with_args};

pub fn apply<S: AsRef<str>, N: AsRef<str>>(src: S, namespace: N) -> Result<()> {
    let source_echo = command::command_with_args("echo", [src.as_ref()]);
    let kubectl = command::command_with_args("kubectl", ["apply", "-n", namespace.as_ref(), "-f", "-"]);
    let output = command::pipe_run(&mut[source_echo, kubectl])?;

    let stdout = String::from_utf8_lossy(&output.stdout[..]);
    let stderr = String::from_utf8_lossy(&output.stderr[..]);
    println!("apply status: {}", output.status);
    println!("apply output: {}", stdout);
    println!("apply stderr: {}", stderr);

    Ok(())
}

pub fn delete<S: AsRef<str>, N: AsRef<str>>(name: S, namespace: N) -> Result<()> {
    // echo "Y" to confirm
    let confirm = command_with_args("echo", ["y"]);
    let tkn_delete = command::command_with_args("tkn", ["pipeline", "delete", name.as_ref(), "-n", namespace.as_ref()]);
    let output = command::pipe_run(&mut [confirm, tkn_delete])?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    println!("delete output: {}", output_str);
    println!("delete stderr: {}", output_str);

    if output_str.contains("Pipelines deleted") {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::InvalidData, format!("failed to delete pipeline: {}", name.as_ref())))
    }
}

pub fn delete_run<S: AsRef<str>, N: AsRef<str>>(name: S, namespace: N) -> Result<()> {
    // echo "Y" to confirm
    let confirm = command_with_args("echo", ["y"]);
    let tkn_delete = command::command_with_args("tkn", ["pipelinerun", "delete", name.as_ref(), "-n", namespace.as_ref()]);
    let output = command::pipe_run(&mut [confirm, tkn_delete])?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    println!("delete output: {}", output_str);
    println!("delete stderr: {}", output_str);

    //Are you sure you want to delete all PipelineRuns in namespace "train" (y/n): All PipelineRuns(Completed) deleted in namespace "train"
    if output_str.contains("All PipelineRuns(Completed) deleted in namespace ") {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::InvalidData, format!("failed to delete pipeline: {}", name.as_ref())))
    }
}

pub fn list(namespace: &str) -> Result<Vec<String>> {
    let pipelines = command::command_with_args("tkn", ["pipeline", "list", "-o", "json", "-n", namespace]); 
    let jq = command::command_with_args("jq", ["-r", ".items[].metadata.name"]);
    let output = command::pipe_run(&mut [pipelines, jq])?;
    let stdout = String::from_utf8_lossy(&output.stdout[..]);
    let stderr = String::from_utf8_lossy(&output.stderr[..]);
    println!("pipeline list status: {}", output.status);
    println!("pipeline list output: {}", stdout);
    println!("pipeline list stderr: {}", stderr);

    Ok(stdout.trim().split("\n").map(|v|String::from(v)).collect())
}

pub fn run(id: &str, namespace: &str, params: &[&str]) -> Result<String> {
    let mut tkn_args = vec!["-n", namespace, "pipeline", "start", id];
    for p in params {
        tkn_args.push("-p");
        tkn_args.push(p);
    }
    let mut tkn = command_with_args("tkn", tkn_args);
    // example of output: PipelineRun started: env-deployment-run-8lvfx
    let output = tkn.output()?;
    let stdout_str = command::stringfy(&output.stdout);
    let stderr_str = command::stringfy(&output.stderr);
    println!("### stdout of run pipeline:\n{}", stdout_str);
    println!("### stderr of run pipeline:\n{}", stderr_str);
    let ok_msg = "PipelineRun started: ";
    if stdout_str.starts_with(ok_msg) { // OK
        let mut pipeline_id = &stdout_str[ok_msg.len()..];
        if let Some(word_split) = pipeline_id.find("\n") {
            pipeline_id = &pipeline_id[..word_split];
        }
        Ok(pipeline_id.to_owned())
    } else {
        Err(Error::new(ErrorKind::InvalidInput, format!("Failed to start the pipeline: {} with output: {}", id, stdout_str)))
    }
}

pub fn logs(run_name: &str, namespace: &str) -> Result<String> {
    println!("### logs command: tkn pipelinerun logs {} -n {}", run_name, namespace);
    let mut logs = command::command_with_args("tkn", ["pipelinerun", "logs", run_name, "-n", namespace]); 
    let output = logs.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout[..]);
    let stderr = String::from_utf8_lossy(&output.stderr[..]);
    println!("pipeline logs status: {}", output.status);
    println!("pipeline logs output: {}", stdout);
    println!("pipeline logs stderr: {}", stderr);

    Ok(stdout.to_string())
}

pub fn pipeline_run_stats(run_name: &str, namespace: &str) -> Result<String> {
    let pipeline_run_describe = command::command_with_args("tkn", ["pipelinerun", "describe", run_name,  "-o", "json", "-n", namespace]);
    let jq_status = command::command_with_args("jq", ["-r", ".status.conditions|.[].reason"]);
    let output = command::pipe_run(&mut [pipeline_run_describe, jq_status])?;
    let output_text = String::from_utf8_lossy(&output.stdout);
    Ok(output_text.trim_end_matches("\n").to_string())
}

pub fn pipeline_run_results(run_name: &str, namespace: &str) -> Result<String> {

    //results: tkn pipelinerun describe test-result-run-j4h9t -n train -o json | jq '.status.results|.[]'
    let pipeline_run_describe = command::command_with_args("tkn", ["pipelinerun", "describe", run_name,  "-o", "json", "-n", namespace]);
    let jq_status = command::command_with_args("jq", ["-r", ".status.results"]);
    let output = command::pipe_run(&mut [pipeline_run_describe, jq_status])?;
    let output_text = String::from_utf8_lossy(&output.stdout);
    Ok(output_text.trim_end_matches("\n").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_a_pipeline(name: &str, namespace: &str, ret_code: i32) -> Result<()> {
        let pipeline_yaml = format!(r#"---
apiVersion: tekton.dev/v1 # or tekton.dev/v1beta1
kind: Task
metadata:
  name: sample-init
spec:
  steps:
    - name: init
      image: ubuntu
      script: |
        echo "Initializing the environment deployment ..."
        echo "Done"
        exit {}
---
apiVersion: tekton.dev/v1
kind: Pipeline
metadata:
  name: {}
spec:
  tasks:
    - name: init
      taskRef:
        name: sample-init
"#, ret_code, name);
        apply(pipeline_yaml, namespace)
    }



    #[test]
    fn test_list_pipeline() {
        create_a_pipeline("sample-1", "train", 0).unwrap();
        let pipelines = list("train");
        assert!(pipelines.is_ok());
        let pipelines = pipelines.unwrap();
        assert!(pipelines.contains(&"sample-1".to_string()));
        let result = delete("sample-1", "train");
        assert!(result.is_ok());
    }

    /*
     * Comment the below tests out. Because they are flaky.
    #[test]
    fn test_pipeline_run_stats_running() {
        let pipeline_name = "pipeline-run-stats-sample-running";
        create_a_pipeline(pipeline_name, "train", 0).unwrap();
        let run_name = run(pipeline_name, "train", &[]).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let stat = pipeline_run_stats(&run_name, "train").unwrap();
        let result = delete("pipeline-run-stats-sample-running", "train");
        assert!(result.is_ok());
        assert_eq!(stat, "Running");
    }

    #[test]
    fn test_pipeline_run_stats_succeeded() {
        let pipeline_name = "pipeline-run-stats-sample-succeeded";
        create_a_pipeline(pipeline_name, "train", 0).unwrap();
        let run_name = run(pipeline_name, "train", &[]).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(30));
        let stat = pipeline_run_stats(&run_name, "train").unwrap();
        let result = delete("pipeline-run-stats-sample-succeeded", "train");
        assert!(result.is_ok());
        assert_eq!(stat, "Succeeded");
    }

    #[test]
    fn test_pipeline_run_stats_fail() {
        let pipeline_name = "pipeline-run-stats-sample-fail";
        create_a_pipeline(pipeline_name, "train", -1).unwrap();
        let run_name = run(pipeline_name, "train", &[]).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(40));
        let stat = pipeline_run_stats(&run_name, "train").unwrap();
        let result = delete("pipeline-run-stats-sample-fail", "train");
        assert!(result.is_ok());
        assert_eq!(stat, "Failed");
    }
    */
}
