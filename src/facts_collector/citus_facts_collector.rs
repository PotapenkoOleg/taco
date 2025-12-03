use crate::shared::active_worker_nodes_result::ActiveWorkerNodesResult;
use crate::shared::pg_dist_node_info_result::PgDistNodeInfoResult;
use anyhow::Result;
use tokio_postgres::NoTls;

pub struct CitusFactsCollector<'a> {
    connection_string: &'a str,
}

impl<'a> CitusFactsCollector<'a> {
    pub fn new(connection_string: &'a str) -> Self {
        CitusFactsCollector { connection_string }
    }

    pub async fn get_active_worker_nodes(&self) -> Result<Vec<ActiveWorkerNodesResult>> {
        let (client, connection) = tokio_postgres::connect(&self.connection_string, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let rows = client
            .query("SELECT * FROM citus_get_active_worker_nodes();", &[])
            .await?;
        let mut result: Vec<ActiveWorkerNodesResult> = Vec::new();
        for row in rows {
            result.push(ActiveWorkerNodesResult {
                node_name: row.get(0),
                node_port: row.get(1),
            });
        }
        Ok(result)
    }

    // https://docs.citusdata.com/en/v13.0/develop/api_metadata.html#worker-node-table
    pub async fn get_pg_dist_node_info(&self) -> Result<Vec<PgDistNodeInfoResult>> {
        let (client, connection) = tokio_postgres::connect(&self.connection_string, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let rows = client
            .query(
                "SELECT *, noderole::varchar FROM pg_dist_node order by groupid, nodename;",
                &[],
            )
            .await?;
        let mut result: Vec<PgDistNodeInfoResult> = Vec::new();
        for row in rows {
            result.push(PgDistNodeInfoResult {
                nodeid: row.get(0),
                groupid: row.get(1),
                nodename: row.get(2),
                nodeport: row.get(3),
                noderack: row.get(4),
                hasmetadata: row.get(5),
                isactive: row.get(6),
                //noderole: None, // skipping custom data type
                nodecluster: row.get(8),
                metadatasynced: row.get(9),
                shouldhaveshards: row.get(10),
                noderole: row.get(11),
            });
        }
        Ok(result)
    }
}
