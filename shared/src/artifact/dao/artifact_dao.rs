use crate::error;
use std::collections::HashMap;
use redis::ConnectionLike;
use super::super::{Artifact, ArtifactRef, Rollout, AccountRef, SecretRef, ArtifactStatus};
use chrono::{DateTime, Local};

pub struct ArtifactDao;

//docker run --name train-redis --network bridge -h redis-server -p 127.0.0.1:6379:6379 -d redis
//docker run -it --rm --network bridge redis redis-cli -h IP
//The IP could be acquired by running `docker network inspect bridge`

impl ArtifactDao {
    pub fn one(id: &str, conn: &mut dyn ConnectionLike) -> error::Result<Artifact> {
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
            stats: build["stats"].clone().into(),
            last_sched: build["last_sched"].parse::<DateTime<Local>>().map_err(|err| error::error(&format!("Fail to convert last schedule time to DateTime: {}", err)))?,
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
            stats: clean["stats"].clone().into(),
            last_sched: clean["last_sched"].parse::<DateTime<Local>>().map_err(|err| error::error(&format!("Fail to convert last schedule time to DateTime: {}", err)))?,
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

    pub fn all_ids(conn: &mut dyn ConnectionLike) -> error::Result<Vec<String>> {
        let ids: Option<Vec<String>> = redis::Cmd::hkeys("artifact:id").query(conn)?;
        match ids {
            Some(art_ids) => Ok(art_ids),
            None => Ok(Vec::new())
        }
    }

    pub fn update_build_status(artifact: &mut Artifact, rollout_error: &Option<error::GeneralError>, conn: &mut dyn ConnectionLike ) -> error::Result<()> {
        match rollout_error {
            None => {
                artifact.build.stats = ArtifactStatus::Running;
            },
            Some(err) => {
                match err {
                    error::GeneralError::PendingArtRef => artifact.build.stats = ArtifactStatus::PendingArtRef,
                    error::GeneralError::PendingAccount => artifact.build.stats = ArtifactStatus::PendingAccount,
                    _ => artifact.build.stats = ArtifactStatus::Failed
                };
            }
        }

        Self::update_hash_fields(&artifact.id, "build", &[("stats", &artifact.build.stats.to_string()), ("last_sched", &artifact.build.last_sched.to_string())], conn)
    }

    pub fn update_clean_status(artifact: &mut Artifact, rollout_error: &Option<error::GeneralError>, conn: &mut dyn ConnectionLike) -> error::Result<()> {
        match rollout_error {
            None => {
                artifact.clean.stats = ArtifactStatus::Running;
            },
            Some(err) => {
                match err {
                    error::GeneralError::PendingArtRef => artifact.clean.stats = ArtifactStatus::PendingArtRef,
                    error::GeneralError::PendingAccount => artifact.clean.stats = ArtifactStatus::PendingAccount,
                    _ => artifact.clean.stats = ArtifactStatus::Failed
                };
            }
        }

        Self::update_hash_fields(&artifact.id, "clean", &[("stats", &artifact.clean.stats.to_string()), ("last_sched", &artifact.build.last_sched.to_string())], conn)
    }

    pub fn delete(id: &str, conn: &mut dyn ConnectionLike) -> error::Result<()> {
        let rec_id: Option<u32> = Self::rec_id(id, conn)?;
        if let Some(rec_id) = rec_id {
            redis::pipe()
                .del(format!("artifact:{}:clean:accounts", rec_id)).ignore()
                .del(format!("artifact:{}:clean:secrets", rec_id)).ignore()
                .del(format!("artifact:{}:clean:art_refs", rec_id)).ignore()
                .del(format!("artifact:{}:clean", rec_id)).ignore()
                .del(format!("artifact:{}:build:accounts", rec_id)).ignore()
                .del(format!("artifact:{}:build:secrets", rec_id)).ignore()
                .del(format!("artifact:{}:build:art_refs", rec_id)).ignore()
                .del(format!("artifact:{}:build", rec_id)).ignore()
                .del(format!("artifact:{}", rec_id)).ignore()
                .hdel("artifact:id", id).ignore()
                .execute(conn);
        }
        Ok(())
    }

    pub fn update(artifact: Artifact, conn: &mut dyn ConnectionLike) -> error::Result<()> {
        Self::persist(artifact, true, conn)
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

    fn update_hash_fields(art_id: &str, sub_field_id: &str, kvs: &[(&str,&str)], conn: &mut dyn ConnectionLike) -> error::Result<()> {
        let rec_id = Self::rec_id(art_id, conn)?;
        if let Some(rec_id) = rec_id {
            let hkey = if sub_field_id.is_empty() {
                format!("artifact:{rec_id}")
            } else {
                format!("artifact:{rec_id}:{}", sub_field_id)
            };

            redis::Cmd::hset_multiple(hkey, kvs).execute(conn);
            Ok(())
        } else {
            Err(error::error(&format!("art_id: {} does not exist", art_id)))
        }
    }

    pub fn save(artifact: Artifact, conn: &mut dyn ConnectionLike) -> error::Result<()> {
        Self::persist(artifact, false, conn)
    }

    fn persist(artifact: Artifact, overwritable: bool, conn: &mut dyn ConnectionLike) -> error::Result<()> {
        let idx_existed: bool = redis::Cmd::hexists("artifact:id", artifact.id.clone()).query(conn)?;
        if idx_existed && !overwritable { return Err(error::error(&format!("record index '{}' exists in artifact:id", artifact.id))); }

        // let record_existed: bool = redis::Cmd::exists(format!("artifact:{record_id}")).query(conn)?;
        // if record_existed && !overwritable { return Err(error::error(&format!("record artifact:{record_id} exists"))); }

        let record_id: Option<u32> = if idx_existed {
            Self::rec_id(&artifact.id, conn)?
        } else {
            redis::Cmd::incr("artifact:last_record_id", 1).query(conn)?
        };

        if let Some(record_id) = record_id {
            Self::write_record(record_id, artifact, conn)
        } else {
            Err(error::error(&format!("Failed to acquire record id for the artifact: {}", &artifact.id)))
        }
    }

    fn write_record(record_id: u32, artifact: Artifact, conn: &mut dyn ConnectionLike) -> error::Result<()> {
        redis::Cmd::hset("artifact:id", &artifact.id, record_id).execute(conn);
        redis::Cmd::hset_multiple(format!("artifact:{record_id}"), &[
                                  ("id", artifact.id),
                                  ("total", artifact.total.to_string()),
                                  ("target", artifact.target.to_string()) ]).execute(conn);
        redis::Cmd::hset_multiple(format!("artifact:{record_id}:build"), &[
                                  ("name", artifact.build.name),
                                  ("stats", artifact.build.stats.to_string()),
                                  ("last_sched", artifact.build.last_sched.to_string()),
                                  ("manifest", artifact.build.manifest) ]).execute(conn);
        Self::update_list(&format!("artifact:{record_id}:build:accounts"), &artifact.build.accounts.into_iter().map(|v| v.name).collect::<Vec<String>>(), conn);
        Self::update_list(&format!("artifact:{record_id}:build:secrets"), &artifact.build.secrets.into_iter().map(|v| v.name).collect::<Vec<String>>(), conn);
        Self::update_list(&format!("artifact:{record_id}:build:art_refs"), &artifact.build.art_refs.into_iter().map(|v| v.name).collect::<Vec<String>>(), conn);

        redis::Cmd::hset_multiple(format!("artifact:{record_id}:clean"), &[
                                  ("name", artifact.clean.name),
                                  ("stats", artifact.clean.stats.to_string()),
                                  ("last_sched", artifact.clean.last_sched.to_string()),
                                  ("manifest", artifact.clean.manifest) ]).execute(conn);
        Self::update_list(&format!("artifact:{record_id}:clean:accounts"), &artifact.clean.accounts.into_iter().map(|v| v.name).collect::<Vec<String>>(), conn);
        Self::update_list(&format!("artifact:{record_id}:clean:secrets"), &artifact.clean.secrets.into_iter().map(|v| v.name).collect::<Vec<String>>(), conn);
        Self::update_list(&format!("artifact:{record_id}:clean:art_refs"), &artifact.clean.art_refs.into_iter().map(|v| v.name).collect::<Vec<String>>(), conn);

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
        let mut conn = redis::Client::open("redis://127.0.0.1").unwrap().get_connection().unwrap() ;
        ArtifactDao::delete("Toronto1", &mut conn).expect("Failed to delete artifact Toronto1");
        let artifact = Artifact {
            id: "Toronto1".to_owned(),
            tags: HashMap::new(),
            total: 3,
            target: 2,
            build: Rollout {
                name: "build-toronto1".to_owned(),
                stats: ArtifactStatus::NotScheduled,
                last_sched: "2012-12-12T12:12:12Z".parse::<DateTime<Local>>().expect("Failed to parse datetime string to last_sched"),
                accounts: vec![AccountRef{name: "gcp".to_owned()}, AccountRef{name: "basic".to_owned()}],
                secrets: vec![SecretRef{name: "awsroute-53".to_owned()}, SecretRef{name: "pivnet".to_owned()}],
                art_refs: vec![ArtifactRef{name: "opsman".to_owned()}, ArtifactRef{name: "gcp-basic".to_owned()}],
                manifest: "{\"name\": \"manifest\"}".to_owned()
            },
            clean: Rollout {
                name: "clean-toronto1".to_owned(),
                stats: ArtifactStatus::NotScheduled,
                last_sched: "2012-12-12T12:12:12Z".parse::<DateTime<Local>>().expect("Failed to parse datetime string to last_sched"),
                accounts: vec![AccountRef{name: "gcp".to_owned()}, AccountRef{name: "basic".to_owned()}],
                secrets: vec![SecretRef{name: "awsroute-53".to_owned()}, SecretRef{name: "pivnet".to_owned()}],
                art_refs: vec![ArtifactRef{name: "opsman".to_owned()}, ArtifactRef{name: "gcp-basic".to_owned()}],
                manifest: "{\"name\": \"manifest\"}".to_owned()
            }
        };

        ArtifactDao::save(artifact.clone(), &mut conn).expect("failed to save artifact dao");

        let loaded_artifact = ArtifactDao::one("Toronto1", &mut conn).expect("Failed to load Artifact");
        assert_eq!(artifact, loaded_artifact);

    }
}
