use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;

use clickhouse::Client;

use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::models::tron::modules::TransactionRow;

pub struct TransactionBatcher {
    clickhouse: Arc<Client>,
    rows:
        Arc<
            Mutex<
                Vec<TransactionRow>
            >
        >,

    max_batch_size: usize,
    flush_interval: Duration,
}

impl TransactionBatcher {

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

                max_batch_size: 5000,

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
        row: TransactionRow,
    ) -> Result<()> {

        let mut rows =
            self.rows.lock().await;

        rows.push(row);

        if rows.len()
            >= self.max_batch_size
        {
            let batch =
                rows.drain(..)
                    .collect::<Vec<_>>();

            drop(rows);

            self.flush(batch).await?;
        }

        Ok(())
    }

    async fn flush(
        &self,
        batch: Vec<TransactionRow>,
    ) -> Result<()> {

        if batch.is_empty() {
            return Ok(());
        }

        let mut insert =
            self.clickhouse
                .insert::<TransactionRow>(
                    "transactions"
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
                        "[TX BATCHER ERROR] {:?}",
                        err
                    );
                }
            }
        });
    }
}