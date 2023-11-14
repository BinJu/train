use redis::ConnectionLike;
use super::super::instance::Instance;
use crate::error;

use std::collections::HashMap;

pub struct InstanceDao;

impl InstanceDao {
    pub fn one(id: &str, art_id: &str, conn: &mut dyn ConnectionLike) -> error::Result<Instance> {
        let idx_existed: bool = redis::Cmd::sismember(format!("instance:{}", art_id), id).query(conn)?;
        if !idx_existed { return Err(error::error(&format!("record index id: {} does not exist in instance:{}", id, art_id)))}

        let results: Option<HashMap<String,String>> = redis::Cmd::hgetall(format!("instance:{}:{}:results", art_id, id)).query(conn)?;
        let record: Option<HashMap<String,String>> = redis::Cmd::hgetall(format!("instance:{}:{}", art_id, id)).query(conn)?;
        
        if let Some(r) = record {
            Ok(Instance {
                id: r["id"].clone(),
                art_id: r["art_id"].clone(),
                run_name: r["run_name"].clone(),
                dirt: r["dirt"].parse().map_err(|err| error::error(&format!("Failed to parse value to bool: {}", err)))?,
                stat: r["stat"].clone().into(),
                results
            })
        } else {
            Err(error::error(&format!("failed to find record: instance:{}:{}", art_id, id)))
        }
    }

    pub fn many(art_id: &str, conn: &mut dyn ConnectionLike) -> error::Result<Vec<Instance>> {
        let inst_ids: Option<Vec<String>> = redis::Cmd::smembers(format!("instance:{}", art_id)).query(conn)?;
        if inst_ids.is_none() { return Ok(Vec::new()) }
        let inst_ids = inst_ids.unwrap();
        let mut result = Vec::new();
        for id in inst_ids {
            let inst = Self::one(&id, art_id, conn)?;
            result.push(inst);
        }
        Ok(result)
    }

    pub fn delete(id: &str, art_id: &str, conn: &mut dyn ConnectionLike) -> error::Result<()> {
        redis::pipe()
            .del(format!("instance:{}:{}:results", art_id, id)).ignore()
            .del(format!("instance:{}:{}", art_id, id)).ignore()
            .srem(format!("instance:{}", art_id), id).ignore()
            .execute(conn);
        Ok(())
    }

    pub fn update(instance: Instance, conn: &mut dyn ConnectionLike) -> error::Result<()> {
        Self::persist(instance, true, conn)
    }

    pub fn update_stats(id: &str, art_id: &str, stats: &str, conn: &mut dyn ConnectionLike) -> error::Result<()> {
        redis::Cmd::hset(format!("instance:{}:{}", art_id, id), "stat", stats).execute(conn);
        Ok(())
    }

    pub fn save(instance: Instance, conn: &mut dyn ConnectionLike) -> error::Result<()> {
        Self::persist(instance, false, conn)
    }

    fn persist(instance: Instance, overwritable: bool, conn: &mut dyn ConnectionLike) -> error::Result<()> {
        let idx_existed = redis::Cmd::sismember(format!("instance:{}", instance.art_id), instance.id.clone()).query(conn)?;
        if idx_existed && !overwritable { return Err(error::error(&format!("record index instance:{} exists", instance.art_id)))}
        let record_existed: bool = redis::Cmd::exists(format!("instance:{}:{}", instance.art_id, instance.id)).query(conn)?;
        if record_existed && !overwritable { return Err(error::error(&format!("record instance:{}:{} exists", instance.art_id, instance.id)))}

        redis::Cmd::hset_multiple(format!("instance:{}:{}", instance.art_id, instance.id.clone()), &[
                                  ("id", instance.id.clone()),
                                  ("art_id", instance.art_id.clone()),
                                  ("run_name", instance.run_name),
                                  ("dirt", instance.dirt.to_string()),
                                  ("stat", instance.stat.to_string())]).execute(conn);

        if let Some(results) = instance.results {
            let mut kvs: Vec<(&str,&str)> = Vec::new();
            for (k,v) in results.iter() {
                kvs.push((k,v))
            }
            if kvs.len() > 0 {
                redis::Cmd::hset_multiple(format!("instance:{}:{}:results", instance.art_id, instance.id), &kvs).execute(conn);
            }
        }

        redis::Cmd::sadd(format!("instance:{}", instance.art_id), instance.id.clone()).execute(conn);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::instance::InstanceStatus;

    #[test]
    fn test_save_and_load_instance() {
        let mut conn = redis::Client::open("redis://127.0.0.1").unwrap().get_connection().unwrap();
        InstanceDao::delete("cold-1234", "opsman", &mut conn).expect("Failed to delete instance opsman:cold-1234");
        let instance = Instance {
            id: "cold-1234".to_owned(),
            art_id: "opsman".to_owned(),
            run_name: "opsman-cold-1234-txj0".to_owned(),
            dirt: true,
            stat: InstanceStatus::Running,
            results: Some(HashMap::from([("url".to_owned(), "https://cold-1234.cf-app.com".to_owned()), ("username".to_owned(), "pivotalAA".to_owned()), ("password".to_owned(), "tjax21".to_owned())]))
        };
        InstanceDao::save(instance.clone(), &mut conn).expect("failed to save the instance");
        let new_instance = InstanceDao::one("cold-1234", "opsman", &mut conn).expect("failed to load instance");
        assert_eq!(instance, new_instance);
    }
}
