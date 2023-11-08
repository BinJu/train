use crate::error;
use std::collections::HashMap;
use redis::{ConnectionLike, Connection};
use super::{Artifact, ArtifactRef, Rollout, AccountRef, SecretRef};

pub struct ArtifactDao { 
    pub conn: Connection 
}

pub fn connection(url: &str) -> error::Result<redis::Connection> {
    let conn = redis::Client::open(url)?.get_connection()?;
    Ok(conn)
}

//docker run --name train-redis --network bridge -h redis-server -p 127.0.0.1:6379:6379 -d redis
//docker run -it --rm --network bridge redis redis-cli -h IP
//The IP could be acquired by running `docker network inspect bridge`

impl ArtifactDao {
    pub fn one(id: &str, conn: &mut dyn redis::ConnectionLike) -> error::Result<Artifact> {
        let rec_id: Option<u32> = Self::rec_id(id, conn)?;
        if rec_id.is_none() {return Err(error::error(&format!("failed to find the artifact id: {id} from DB")))};
        // read the record
        let rec_id = rec_id.unwrap();
        let record: Option<HashMap<String,String>> = Self::record(rec_id, conn)?;
        if record.is_none() {return Err(error::error(&format!("failed to find the artifact record: {rec_id} from DB")))};
        let record = record.unwrap();

        let  build: Option<HashMap<String,String>> = redis::Cmd::new().arg("HGETALL").arg(&format!("artifact:{rec_id}:build")).query(conn)?;
        if build.is_none() {return Err(error::error(&format!("failed to find the artifact build: artifact:{rec_id}:build from DB")))};
        let build = build.unwrap();

        let build_accounts: Option<Vec<String>> = redis::Cmd::new().arg("SMEMBERS").arg(&format!("artifact:{rec_id}:build:accounts")).query(conn)?;
        let build_secrets: Option<Vec<String>> = redis::Cmd::new().arg("SMEMBERS").arg(&format!("artifact:{rec_id}:build:secrets")).query(conn)?;
        let build_art_refs: Option<Vec<String>> = redis::Cmd::new().arg("SMEMBERS").arg(&format!("artifact:{rec_id}:build:art_refs")).query(conn)?;
        let build_rollout = Rollout{
            name: build["name"].clone(),
            accounts: build_accounts.map_or(Vec::new(), |v| v.into_iter().map(|vi| AccountRef {name: vi}).collect()),
            secrets: build_secrets.map_or(Vec::new(), |v| v.into_iter().map(|vi| SecretRef{name: vi}).collect()),
            art_refs: build_art_refs.map_or(Vec::new(), |v| v.into_iter().map(|vi| ArtifactRef {name: vi}).collect()),
            manifest: build["manifest"].clone()
        };

        let clean: Option<HashMap<String,String>> = redis::Cmd::new().arg("HGETALL").arg(&format!("artifact:{rec_id}:clean")).query(conn)?;
        if clean.is_none() {return Err(error::error(&format!("failed to find the artifact clean: artifact:{rec_id}:clean from DB")))};
        let clean = clean.unwrap();

        let clean_accounts: Option<Vec<String>> = redis::Cmd::new().arg("SMEMBERS").arg(&format!("artifact:{rec_id}:clean:accounts")).query(conn)?;
        let clean_secrets: Option<Vec<String>> = redis::Cmd::new().arg("SMEMBERS").arg(&format!("artifact:{rec_id}:clean:secrets")).query(conn)?;
        let clean_art_refs: Option<Vec<String>> = redis::Cmd::new().arg("SMEMBERS").arg(&format!("artifact:{rec_id}:clean:art_refs")).query(conn)?;
        let clean_rollout = Rollout{
            name: clean["name"].clone(),
            accounts: clean_accounts.map_or(Vec::new(), |v| v.into_iter().map(|vi| AccountRef {name: vi}).collect()),
            secrets: clean_secrets.map_or(Vec::new(), |v| v.into_iter().map(|vi| SecretRef{name: vi}).collect()),
            art_refs: clean_art_refs.map_or(Vec::new(), |v| v.into_iter().map(|vi|ArtifactRef {name: vi}).collect()),
            manifest: clean["manifest"].clone()
        };


        Ok(Artifact {
            id: id.to_owned(),
            tags: HashMap::new(),
            total: record["total"].parse::<u32>().unwrap_or(0u32),
            target: record["target"].parse::<u32>().unwrap_or(0u32),
            build: build_rollout,
            clean: clean_rollout
        })
    }

    pub fn many(ids: &[&str], conn: &mut dyn redis::ConnectionLike) -> error::Result<Vec<Self>> {
        Err(error::error("unimplemented yet"))
    }

    pub fn delete() {

    }

    pub fn update(&mut self, artifact: Artifact) -> error::Result<()> {
        self.persist(artifact, true)
    }

    fn rec_id(art_id: &str, conn: &mut dyn ConnectionLike) -> error::Result<Option<u32>> {
        let rec_id: Option<u32> = redis::Cmd::new().arg("HGET").arg(format!("artifact:id")).arg(art_id).query(conn)?;
        Ok(rec_id)
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

    pub fn save(&mut self, artifact: Artifact) -> error::Result<()> {
        self.persist(artifact, false)
        //    create a new record {
        //        redis incr artifact:last_record_id
        //        read new_id = artifact:last_record_id
        //        if exist hset artifact:{new_id} then
        //            error
        //        fi
        //        update hset fields to artifact:{record_id} {
        //            update {
        //               id
        //               total
        //               target
        //            }
        //        }
        //        update hset fileds to artifact:{record_id}:build {
        //            update {
        //               name
        //               manifest
        //            }
        //        }
        //        update lset list to artifact:{record_id}:build:accounts {
        //            update accounts
        //        }
        //
        //        update lset list to artifact:{record_id}:build:accounts {
        //            update secrets
        //        }
        //
        //        update lset list to artifact:{record_id}:build:art_refs{
        //            update art_refs
        //        }
        //    }
        //        update hset fileds to artifact:{record_id}:clean{
        //            update {
        //               name
        //               manifest
        //            }
        //        }
        //        update lset list to artifact:{record_id}:clean:accounts {
        //            update accounts
        //        }
        //
        //        update lset list to artifact:{record_id}:clean:accounts {
        //            update secrets
        //        }
        //
        //        update lset list to artifact:{record_id}:clean:art_refs{
        //            update art_refs
        //        }
        //    }

    }

    pub fn notify(&self, conn: &mut dyn redis::ConnectionLike, channel: &str) -> error::Result<()> {
        Ok(())
    }

    fn persist(&mut self, artifact: Artifact, overwritable: bool) -> error::Result<()> {
        let record_id: u32 = redis::Cmd::incr("artifact:last_record_id", 1).query(&mut self.conn)?;
        let record_existed: bool = redis::Cmd::exists(format!("artifact:{record_id}")).query(&mut self.conn)?;
        if record_existed && !overwritable { return Err(error::error(&format!("record artifact:{record_id} exists"))); }

        let idx_existed: bool = redis::Cmd::hexists("artifact:id", artifact.id.clone()).query(&mut self.conn)?;
        if idx_existed && !overwritable { return Err(error::error(&format!("record index '{}' exists in artifact:id", artifact.id))); }

        redis::Cmd::hset("artifact:id", artifact.id.clone(), record_id).execute(&mut self.conn);
        redis::Cmd::hset_multiple(format!("artifact:{record_id}"), &[
                                  ("id", artifact.id),
                                  ("total", artifact.total.to_string()),
                                  ("target", artifact.target.to_string()) ]).execute(&mut self.conn);
        redis::Cmd::hset_multiple(format!("artifact:{record_id}:build"), &[
                                  ("name", artifact.build.name),
                                  ("manifest", artifact.build.manifest) ]).execute(&mut self.conn);
        Self::update_list(&format!("artifact:{record_id}:build:accounts"), &artifact.build.accounts.into_iter().map(|v| v.name).collect::<Vec<String>>(), &mut self.conn);
        Self::update_list(&format!("artifact:{record_id}:build:secrets"), &artifact.build.secrets.into_iter().map(|v| v.name).collect::<Vec<String>>(), &mut self.conn);
        Self::update_list(&format!("artifact:{record_id}:build:art_refs"), &artifact.build.art_refs.into_iter().map(|v| v.name).collect::<Vec<String>>(), &mut self.conn);

        redis::Cmd::hset_multiple(format!("artifact:{record_id}:clean"), &[
                                  ("name", artifact.clean.name),
                                  ("manifest", artifact.clean.manifest) ]).execute(&mut self.conn);
        Self::update_list(&format!("artifact:{record_id}:clean:accounts"), &artifact.clean.accounts.into_iter().map(|v| v.name).collect::<Vec<String>>(), &mut self.conn);
        Self::update_list(&format!("artifact:{record_id}:clean:secrets"), &artifact.clean.secrets.into_iter().map(|v| v.name).collect::<Vec<String>>(), &mut self.conn);
        Self::update_list(&format!("artifact:{record_id}:clean:art_refs"), &artifact.clean.art_refs.into_iter().map(|v| v.name).collect::<Vec<String>>(), &mut self.conn);

        Ok(())
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
    fn test_save_and_load_artifact() {
        let conn = redis::Client::open("redis://127.0.0.1").unwrap().get_connection().unwrap();
        let artifact = Artifact {
            id: "Toronto1".to_owned(),
            tags: HashMap::new(),
            total: 3,
            target: 2,
            build: Rollout {
                name: "build-toronto1".to_owned(),
                accounts: vec![AccountRef{name: "gcp".to_owned()}, AccountRef{name: "basic".to_owned()}],
                secrets: vec![SecretRef{name: "awsroute-53".to_owned()}, SecretRef{name: "pivnet".to_owned()}],
                art_refs: vec![ArtifactRef{name: "opsman".to_owned()}, ArtifactRef{name: "gcp-basic".to_owned()}],
                manifest: "{\"name\": \"manifest\"}".to_owned()
            },
            clean: Rollout {
                name: "clean-toronto1".to_owned(),
                accounts: vec![AccountRef{name: "gcp".to_owned()}, AccountRef{name: "basic".to_owned()}],
                secrets: vec![SecretRef{name: "awsroute-53".to_owned()}, SecretRef{name: "pivnet".to_owned()}],
                art_refs: vec![ArtifactRef{name: "opsman".to_owned()}, ArtifactRef{name: "gcp-basic".to_owned()}],
                manifest: "{\"name\": \"manifest\"}".to_owned()
            }
        };

        let mut dao = ArtifactDao {
            conn
        };
        dao.save(artifact.clone()).expect("failed to save artifact dao");

        let loaded_artifact = ArtifactDao::one("Toronto1", &mut dao.conn).expect("Failed to load Artifact");
        assert_eq!(artifact, loaded_artifact);

    }
}
