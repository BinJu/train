use redis::{ConnectionLike, Connection};
use super::instance::{Instance, InstanceStatus};
use crate::error;

use std::collections::HashMap;

pub struct InstanceDao {
    pub conn: Connection
}

impl InstanceDao {
    pub fn one(id: &str, art_id: &str, conn: &mut dyn redis::ConnectionLike) -> error::Result<Instance> {
        let idx_existed: bool = redis::Cmd::sismember(format!("instance:{}", art_id), id).query(conn)?;
        if !idx_existed { return Err(error::error(&format!("record index id: {} exists in instance:{}", id, art_id)))}

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

    pub fn many(art_ids: &str, _conn: &mut dyn redis::ConnectionLike) -> error::Result<Vec<Instance>> {
        Err(error::error("unimplemented yet"))
    }

    pub fn delete(id: &str, art_id: &str, conn: &mut dyn redis::ConnectionLike) -> error::Result<()> {
        redis::pipe()
            .del(format!("instance:{}:{}:results", art_id, id)).ignore()
            .del(format!("instance:{}:{}", art_id, id)).ignore()
            .srem(format!("instance:{}", art_id), id).ignore()
            .execute(conn);
        Ok(())
    }

    pub fn update(&mut self, instance: Instance) -> error::Result<()> {
        self.persist(instance, true)
    }

    fn record(rec_id: u32, conn: &mut dyn ConnectionLike) -> error::Result<Option<HashMap<String,String>>> {
        let record: Option<HashMap<String,String>> = redis::Cmd::new().arg("HGETALL").arg(format!("artifact:{rec_id}")).query(conn)?;
        Ok(record)
    }

    fn update_list(key: &str, members: &[String], conn: &mut dyn ConnectionLike) {
        if members.is_empty() {
            return
        }
        redis::Cmd::sadd(key, members).execute(conn);
    }

    pub fn save(&mut self, instance: Instance) -> error::Result<()> {
        self.persist(instance, false)
    }

    fn persist(&mut self, instance: Instance, overwritable: bool) -> error::Result<()> {
        let idx_existed = redis::Cmd::sismember(format!("instance:{}", instance.art_id), instance.id.clone()).query(&mut self.conn)?;
        if idx_existed && !overwritable { return Err(error::error(&format!("record index instance:{} exists", instance.art_id)))}
        let record_existed: bool = redis::Cmd::exists(format!("instance:{}:{}", instance.art_id, instance.id)).query(&mut self.conn)?;
        if record_existed && !overwritable { return Err(error::error(&format!("record instance:{}:{} exists", instance.art_id, instance.id)))}

        redis::Cmd::hset_multiple(format!("instance:{}:{}", instance.art_id, instance.id.clone()), &[
                                  ("id", instance.id.clone()),
                                  ("art_id", instance.art_id.clone()),
                                  ("run_name", instance.run_name),
                                  ("dirt", instance.dirt.to_string()),
                                  ("stat", instance.stat.to_string())]).execute(&mut self.conn);

        if let Some(results) = instance.results {
            let mut kvs: Vec<(&str,&str)> = Vec::new();
            for (k,v) in results.iter() {
                kvs.push((k,v))
            }
            redis::Cmd::hset_multiple(format!("instance:{}:{}:results", instance.art_id, instance.id), &kvs).execute(&mut self.conn);
        }

        redis::Cmd::sadd(format!("instance:{}", instance.art_id), instance.id.clone()).execute(&mut self.conn);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut dao = InstanceDao{conn};
        dao.save(instance.clone()).expect("failed to save the instance");
        let new_instance = InstanceDao::one("cold-1234", "opsman", &mut dao.conn).expect("failed to load instance");
        assert_eq!(instance, new_instance);
    }
}
