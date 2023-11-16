VERSION 0.7

image:
    BUILD ./api+image
    BUILD ./scheduler+image
    BUILD ./reconciller+image

deploy-to-kind:
    LOCALLY
    WAIT
        BUILD +image
    END
    WAIT
        RUN docker tag api localhost:5000/api \
            && docker tag scheduler localhost:5000/scheduler \
            && docker tag reconciller localhost:5000/reconciller
    END
    WAIT
        RUN  docker push localhost:5000/api \
            && docker push localhost:5000/scheduler \
            && docker push localhost:5000/reconciller
    END

    RUN kubectl apply -f deployment/train-redis.yml
    RUN kubectl apply -f deployment/train-api.yml
    RUN kubectl apply -f deployment/train-scheduler.yml
    RUN kubectl apply -f deployment/train-reconciller.yml

deploy-to-kube:

start-kind-cluster:
    LOCALLY
    RUN deployment/tekton_in_kind.sh -k
