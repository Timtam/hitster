use bitflags::Flags;
use hitster_core::{Permissions, User};
use sqlx::sqlite::SqlitePool;

pub struct EditArgs {
    pub admin: bool,
    pub permissions: u32,
}

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
                println!(
                    "\t\t{} ({}): {}",
                    flag.0.to_lowercase(),
                    flag.1.bits(),
                    user.permissions.contains(flag.1)
                );
            }
        }

        true
    } else {
        println!("unable to open {url}");
        false
    }
}

pub async fn edit(url: &str, id: &str, args: EditArgs) -> bool {
    let permissions = if args.admin {
        Some(Permissions::all())
    } else {
        let p = Permissions::from_bits_retain(args.permissions);
        if p.contains_unknown_bits() {
            println!(
                "unknown permission bits: {}",
                p.bits() & !Permissions::all().bits()
            );
            None
        } else {
            Some(p)
        }
    };

    if permissions.is_none() {
        return false;
    }

    if let Ok(pool) = SqlitePool::connect(url).await {
        let mut conn = pool.acquire().await.unwrap();
        if let Some(user) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut *conn)
            .await
            .unwrap()
        {
            let _ = sqlx::query("UPDATE users SET permissions = ? WHERE id = ?")
                .bind(permissions.as_ref().unwrap().bits())
                .bind(id)
                .execute(&mut *conn)
                .await;

            println!(
                "permissions for user {} ({}) were set to:",
                user.name, user.id
            );

            for flag in Permissions::all().iter_names() {
                println!(
                    "\t{} ({}): {}",
                    flag.0.to_lowercase(),
                    flag.1.bits(),
                    permissions.as_ref().unwrap().contains(flag.1)
                );
            }
            true
        } else {
            println!("no user with that id was found in the database");
            false
        }
    } else {
        println!("unable to open {url}");
        false
    }
}
