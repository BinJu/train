fn main() {
    println!("Hello, world!");
    //TODO: Reconciller's main tasks:
    //1. Scan all the artifacts that are under status:
    //   Fail/PendingArtRef/PendingAccount
    //   Please note that we should protect the system by:
    //   a. Not schedule the failed artfact too frequently. We could tell that easily by reading
    //   the last_sched field.
    //   b. Not schedule the artifact that has too many failed instances. Like 20%?
    //2. Sync up the status of the instances by reading the Tekton PipelineRun/TaskRun. And
    //   populate the 'results' back to the Instance.
}
