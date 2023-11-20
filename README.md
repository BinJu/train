# Why is the project
This project add a thin layer `artifact` onto `tekton` to manage the deployments on Kubernetes.
`account`s and `secret` are supported to ease the deployment.

# How to work with this project
This project creates an API server based on kubernetes.
Users interacts with API to create, get, list and delete the artifacts.

## Create
### Create an artifact
Users can `POST /api/v1/art with the data ${JSON_DATA}`

The minimum example of the artifact request is as below:

```json
{
  "name": "carv01",
  "total": 1,
  "target": 1,
  "build": {
    "tasks": [
      {
        "name": "opsman-task1",
        "spec": {
          "steps": [
            {
              "name": "step-collectdata",
              "image": "ubuntu",
              "script": "YOUR SCRIPT"
            }
          ]
        }
      }
    ]
  },
  "clean": {
    "tasks": [
      {
        "name": "task1",
        "spec": {
          "steps": [
            {
              "name": "collect-data",
              "image": "ubuntu",
              "script": "echo $(params.name)\necho with art_id:"
            }
          ]
        }
      }
    ]
  }
}
```

And the full example can be found in the file: `src/main.rs`.

### Create a resource
Users can `POST /api/v1/res?params=${JSON_PARAMS}&manifest=${JSON_DATA}`
### Creaet a secret
Users can `POST /api/v1/sec?params=${JSON_PARAMS}&manifest=${JSON_DATA}`



# Access
Each of the artifacts, resourct and secrets limits its access by an white list. And it has only one owner. Only owner or admin has the rigths to delocate it.

# Notice
This is project is still in progress. The first stage is done. So far We can deploy an artifact with `cargo test`. But there is many work to be done. Here is the recently plan:
- Split the code into 2 components:
  - API service
    Which is responsible to response a API call.
  - scheduler
    Schedule and rollout the artifacts, and sync the status of the artifacts between the app and the worker(Tekton).
- Storage.
  I am planning to get the Redis into this project. For the reasons below:
  - Fast operations.
  - Decouple the API service and schedulers by pub/sub model of the Redis.
  - Persist the objects of Artifact, Account, and Secret.
