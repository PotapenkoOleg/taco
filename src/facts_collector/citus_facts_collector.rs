use crate::shared::citus_facts_collector_result::CitusFactsCollectorResult;
use anyhow::Result;
use tokio_postgres::NoTls;

pub struct CitusFactsCollector<'a> {
    connection_string: &'a str,
}

impl<'a> CitusFactsCollector<'a> {
    pub fn new(connection_string: &'a str) -> Self {
        CitusFactsCollector { connection_string }
    }

    pub async fn get_active_worker_nodes(&self) -> Result<Vec<CitusFactsCollectorResult>> {
        let (client, connection) = tokio_postgres::connect(&self.connection_string, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        // SELECT nodename, nodeport, noderole, groupid
        // FROM pg_dist_node;
        let rows = client
            .query("SELECT * FROM citus_get_active_worker_nodes();", &[])
            .await?;
        let mut result: Vec<CitusFactsCollectorResult> = Vec::new();
        for row in rows {
            result.push(CitusFactsCollectorResult {
                node_name: row.get(0),
                node_port: row.get(1),
            });
        }
        Ok(result)
    }
}
