use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;

use clickhouse::Client;

use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::models::tron::relationship::AddressRelationshipRow;

pub struct RelationshipBatcher {

    clickhouse: Arc<Client>,

    rows:
        Arc<
            Mutex<
                Vec<AddressRelationshipRow>
            >
        >,

    max_batch_size: usize,

    flush_interval: Duration,
}

impl RelationshipBatcher {

    pub fn new(
        clickhouse: Arc<Client>,
    ) -> Arc<Self> {

        let batcher =
            Arc::new(Self {

                clickhouse,

                rows:
                Arc::new(
                    Mutex::new(
                        Vec::new()
                    )
                ),

                max_batch_size: 20000,

                flush_interval:
                Duration::from_secs(1),
            });

        Self::start_flush_task(
            batcher.clone()
        );

        batcher
    }

    pub async fn push(
        &self,
        row: AddressRelationshipRow,
    ) -> Result<()> {

        let mut rows =
            self.rows.lock().await;

        rows.push(row);

        if rows.len()
            >= self.max_batch_size
        {
            let batch =
                rows
                    .drain(..)
                    .collect::<Vec<_>>();

            drop(rows);

            self.flush(batch).await?;
        }

        Ok(())
    }

    async fn flush(
        &self,
        batch: Vec<AddressRelationshipRow>,
    ) -> Result<()> {

        if batch.is_empty() {
            return Ok(());
        }

        let mut insert =
            self.clickhouse
                .insert::<AddressRelationshipRow>(
                    "address_relationships"
                )
                .await?;

        for row in batch.iter() {

            insert
                .write(row)
                .await?;
        }

        insert.end().await?;

        Ok(())
    }

    fn start_flush_task(
        batcher: Arc<Self>,
    ) {

        tokio::spawn(async move {

            loop {

                sleep(
                    batcher.flush_interval
                ).await;

                let batch = {

                    let mut rows =
                        batcher
                            .rows
                            .lock()
                            .await;

                    if rows.is_empty() {
                        continue;
                    }

                    rows
                        .drain(..)
                        .collect::<Vec<_>>()
                };

                if let Err(err) =
                    batcher
                        .flush(batch)
                        .await
                {
                    eprintln!(
                        "[RELATIONSHIP BATCHER ERROR] {:?}",
                        err
                    );
                }
            }
        });
    }
}