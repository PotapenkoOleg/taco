use tokio_postgres::{Error, NoTls};

pub struct CitusFactsCollector<'a> {
    connection_string: &'a str, // "host=192.168.4.112 dbname=stampede user=postgres password=postgres"
}

impl<'a> CitusFactsCollector<'a> {
    pub fn new(connection_string: &'a str) -> Self {
        CitusFactsCollector {
            connection_string,
        }
    }

    pub async fn get_active_worker_nodes(&self) -> Result<Vec<(String, i64)>, Error> {
        let (client, connection) = tokio_postgres::connect(&self.connection_string, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let rows = client
            .query("SELECT * FROM citus_get_active_worker_nodes();", &[])
            .await?;
        let mut result: Vec<(String, i64)> = Vec::new();
        for row in rows {
            let node_name: &str = row.get(0);
            let node_port: i64 = row.get(1);
            result.push((node_name.to_string(), node_port));
        }
        Ok(result)
    }
}
