use crate::error;
use redis::ConnectionLike;

pub struct Queue {
    name: String
}

impl Queue {
    pub fn enqueue(&self, art_id: &str, conn: &mut dyn ConnectionLike) -> error::Result<()> {
        redis::Cmd::rpush(&self.name, art_id).execute(conn);
        Ok(())
    }

    pub fn dequeue(&self, conn: &mut dyn ConnectionLike) -> error::Result<Option<String>> {
        let response: Option<String> = redis::Cmd::lpop(&self.name, None).query(conn)?;
        Ok(response)
    }

    pub fn block_dequeue(&self, timeout_sec: usize, conn: &mut dyn ConnectionLike) -> error::Result<String> {
        let response: Option<(String,String)> = redis::Cmd::blpop(&self.name, timeout_sec).query(conn)?;
        match response {
            Some((_, art_id)) => Ok(art_id),
            None => Err(error::error("Timeout"))
        }
    }

    pub fn reset(&self, conn: &mut dyn ConnectionLike) -> error::Result<()> {
        redis::Cmd::del(&self.name).execute(conn);
        Ok(())
    }

    pub fn new(name: String) -> Self {
        Self { name }
    } 
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enqueue_and_dequeue() {
        let queue = Queue { name: "unit-test-01".to_owned() };
        let mut conn = redis::Client::open("redis://127.0.0.1").unwrap().get_connection().unwrap();
        queue.reset(&mut conn).unwrap();
        queue.enqueue("art-001", &mut conn).unwrap();
        queue.enqueue("art-002", &mut conn).unwrap();
        let art_id = queue.block_dequeue(5, &mut conn).unwrap();
        assert_eq!(art_id, "art-001");
        let art_id = queue.dequeue(&mut conn).unwrap().unwrap();
        assert_eq!(art_id, "art-002");
        let art_id = queue.dequeue(&mut conn).unwrap();
        assert!(art_id.is_none());
        let art_id = queue.block_dequeue(1, &mut conn);
        assert!(art_id.is_err());
    }
}
