use hitster_core::{Permissions, User};
use sqlx::sqlite::SqlitePool;

pub async fn list(url: &str) -> bool {
    if let Ok(pool) = SqlitePool::connect(url).await {
        let mut conn = pool.acquire().await.unwrap();
        let users = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&mut *conn)
            .await
            .unwrap();

        for user in users.into_iter() {
            println!("{}:", user.name);
            println!("\tid: {}", user.id);
            println!("\tpermissions:");
            for flag in Permissions::all().iter_names() {
                println!("\t\t{}: {}", flag.0.to_lowercase(), user.permissions.contains(flag.1));
            }
        }

        true
    } else {
        println!("unable to open {}", url);
        false
    }
}
