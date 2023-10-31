# Why is the project
This project help to create, manage, and release artifacts.

# How to work with this project
This project creates an API server based on kubernetes.
Users interacts with API to create, get, list and delete the artifacts.

## Create
### Create an artifact
Users can `POST /api/v1/art?manifest=${JSON_DATA}`

The format of the manifest is as below:

```json
{
  "version": "0.1",
  "tasks": {
    "name": "task-1"
      "params": [{
			  "param1": "value1",
				"type": "string",
				"description": "descripton of this param"
      },
			{
			  "param2": "value1",
				"type": "string",
				"description": "descripton of this param"
      }],
			"steps": [{

			}]
  }

}```

### Create a resource
Users can `POST /api/v1/res?params=${JSON_PARAMS}&manifest=${JSON_DATA}`
### Creaet a secret
Users can `POST /api/v1/sec?params=${JSON_PARAMS}&manifest=${JSON_DATA}`



# Access
Each of the artifacts, resourct and secrets limits its access by an white list. And it has only one owner. Only owner or admin has the rigths to delocate it.


