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

pub fn run<'a>(id: &'a str, namespace: &'a str, params: Vec<&'a str>) -> Result<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_a_pipeline(name: &str, namespace: &str) -> Result<()> {
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
"#, name);
        apply(pipeline_yaml, namespace)
    }



    #[test]
    fn test_list_pipeline() {
        create_a_pipeline("sample-1", "train").unwrap();
        let pipelines = list("train");
        assert!(pipelines.is_ok());
        let pipelines = pipelines.unwrap();
        assert!(pipelines.contains(&"sample-1".to_string()));
        let result = delete("sample-1", "train");
        assert!(result.is_ok());
    }

}
