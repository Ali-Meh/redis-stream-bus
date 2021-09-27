use std::process;

pub struct RedisServer {
    pub process: process::Child,
    pub addr: String,
}

impl RedisServer {
    pub fn new(bind: String, port: String) -> RedisServer {
        RedisServer::new_with_addr(bind, port, |cmd| {
            cmd.spawn()
                .unwrap_or_else(|err| panic!("Failed to run {:?}: {}", cmd, err))
        })
    }

    pub fn new_with_addr<F: FnOnce(&mut process::Command) -> process::Child>(
        bind: String,
        port: String,
        spawner: F,
    ) -> RedisServer {
        let mut redis_cmd = process::Command::new("redis-server");
        redis_cmd
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null());

        redis_cmd
            .arg("--port")
            .arg(port.clone())
            .arg("--bind")
            .arg(bind.clone());

        RedisServer {
            process: spawner(&mut redis_cmd),
            addr: format!("redis://{}", bind),
        }
    }

    pub fn get_client_addr(&self) -> &str {
        &self.addr
    }

    pub fn stop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

impl Drop for RedisServer {
    fn drop(&mut self) {
        self.stop()
    }
}
