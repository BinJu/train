use redis::ConnectionLike;

use crate::error;
use std::collections::HashMap;

pub struct RedisCmd<'a> {
    pub conn: &'a mut dyn ConnectionLike
}

pub struct ArtifactDao {
   pub id: String,
   pub tags: HashMap<String,String>,
   pub total: u32,
   pub target: u32,
   pub art_refs: Vec<String>
}

//docker run --name train-redis --network bridge -h redis-server -p 127.0.0.1:6379:6379 -d redis
//docker run -it --rm --network bridge redis-cli -h IP
impl <'a>RedisCmd<'a> {
    pub fn query<T: redis::FromRedisValue>(&mut self, cmd: &[&'a str]) -> error::Result<T> {
        redis::Cmd::new().arg("SET").arg("my_key").arg("Toronto").execute(self.conn);
        redis::Cmd::new().arg("GET").arg("my_key").query(self.conn).map_err(|err|err.into())
        
    }
}

impl ArtifactDao {
    pub fn one(id: &str, conn: &mut dyn redis::ConnectionLike) -> error::Result<Self> {
        Ok(ArtifactDao {
            id: String::from("Opsman"),
            tags: HashMap::new(),
            total: 1,
            target: 1,
            art_refs: Vec::new()
        })
    }
    pub fn many(ids: &[&str], conn: &mut dyn redis::ConnectionLike) -> error::Result<Vec<Self>> {
        Ok(vec![ArtifactDao {
            id: String::from("Opsman"),
            tags: HashMap::new(),
            total: 1,
            target: 1,
            art_refs: Vec::new()
        }])
    }

    pub fn delete() {

    }

    pub fn update() {
        // if exist record_id in artifact:name; then
        //    update {
        //        update hset fields to artifact:{record_id}
        //    }
        // else
        //    error
        // fi
    }

    pub fn save() {
        //    create a new record {
        //        redis incr artifact:last_record_id
        //        read new_id = artifact:last_record_id
        //        if exist hset artifact:{new_id} then
        //            error
        //        fi
        //        update hset fields to artifact:{record_id}
        //    }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_to_redis() {
        redis::Client::open("redis://127.0.0.1").unwrap().get_connection().unwrap();
    }

    #[test]
    fn test_load_artifact() {
        let mut conn = redis::Client::open("redis://127.0.0.1").unwrap().get_connection().unwrap();
        let mut redis_cmd = RedisCmd{conn: &mut conn};
        let value: String = redis_cmd.query(&["test"]).expect("failed to query redis");
        assert_eq!(value, String::from("Toronto"));
    }

    #[test]
    fn test_save_artifact() {
    }
}
