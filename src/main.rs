use futures::{stream, StreamExt};
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use uuid::Uuid;

const CONCURRENCY: usize = 20;
const NUM_ITERATIONS: usize = 1_000;

#[tokio::main]
async fn main() {
    let pool = MySqlPoolOptions::new()
        .max_connections(20)
        .connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    stream::iter(1..=NUM_ITERATIONS)
        .for_each_concurrent(CONCURRENCY, |_| async {
            let uuid = Uuid::new_v4().to_hyphenated().to_string();

            let mut transaction = pool.begin().await.unwrap();

            // Insert five records in one INSERT statement. Set batch_id to the same UUID
            // for all five records.
            sqlx::query!(
                "INSERT INTO test ( batch_id ) VALUES ( ? ), ( ? ), ( ? ), ( ? ), ( ? )",
                uuid,
                uuid,
                uuid,
                uuid,
                uuid,
            )
            .execute(&mut transaction)
            .await
            .unwrap();

            // Determine IDs of inserted records by using LAST_INSERT_ID and ROW_COUNT.
            let result =
                sqlx::query!("SELECT LAST_INSERT_ID() as last_insert_id, ROW_COUNT() as row_count")
                    .fetch_one(&mut transaction)
                    .await
                    .unwrap();

            let start = result.last_insert_id as i64;
            let end = result.last_insert_id as i64 + result.row_count;
            let ids_from_last_insert_id: Vec<i64> = (start..end).collect();

            // Determine IDs of inserted records by selecting where batch_id is equal to
            // the uuid for this iteration.
            let ids_from_batch_id: Vec<i64> =
                sqlx::query!("SELECT id FROM test WHERE batch_id = ?", uuid)
                    .fetch_all(&mut transaction)
                    .await
                    .unwrap()
                    .iter()
                    .map(|row| row.id as i64)
                    .collect();

            // Assert that the IDs from using both approaches match.
            assert_eq!(ids_from_last_insert_id, ids_from_batch_id);

            transaction.commit().await.unwrap();

            // The approach below is intentionally broken to verify that this script will catch
            // concurrency bugs. Uncomment the lines below and set CONCURRENCY to 1 and the
            // script will succeed. Set concurrency to a larger number (say 20), and the script
            // will fail.

            // let max_id = sqlx::query!("SELECT MAX(id) as max_id FROM test")
            //     .fetch_one(&pool)
            //     .await
            //     .unwrap()
            //     .max_id
            //     .unwrap() as i64;

            // let ids_from_select_max: Vec<i64> = ((max_id - 4)..=max_id).collect();

            // assert_eq!(ids_from_batch_id, ids_from_select_max);
        })
        .await;
}
