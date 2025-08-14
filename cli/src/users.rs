use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use dialoguer::Password;
use hitster_core::{Permissions, User};
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

pub struct EditArgs {
    pub admin: bool,
    pub permissions: Option<u32>,
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
        Permissions::all()
    } else {
        Permissions::from_bits_truncate(args.permissions.unwrap())
    };

    if let Ok(pool) = SqlitePool::connect(url).await {
        let mut conn = pool.acquire().await.unwrap();
        if let Some(user) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut *conn)
            .await
            .unwrap()
        {
            let _ = sqlx::query("UPDATE users SET permissions = ? WHERE id = ?")
                .bind(permissions.bits())
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
                    permissions.contains(flag.1)
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

pub async fn create(url: &str, name: &str, args: EditArgs) -> bool {
    let permissions = if args.admin {
        Permissions::all()
    } else if let Some(p) = args.permissions {
        Permissions::from_bits_truncate(p)
    } else {
        Permissions::from_bits(0).unwrap()
    };

    if let Ok(pool) = SqlitePool::connect(url).await {
        let mut conn = pool.acquire().await.unwrap();

        if sqlx::query("SELECT * FROM users WHERE name = ?")
            .bind(name)
            .fetch_optional(&mut *conn)
            .await
            .unwrap()
            .is_some()
        {
            println!("a user with this name already exists");
            return false;
        }

        let password = Password::new()
            .with_prompt("Enter new password")
            .with_confirmation("Repeat password", "The passwords don't match")
            .interact();

        if password.is_err() {
            println!("aborted.");
            return false;
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let pw = argon2
            .hash_password(password.unwrap().as_bytes(), &salt)
            .unwrap()
            .to_string();
        let id = Uuid::new_v4();

        let _ = sqlx::query(
            "INSERT INTO users (id, name, password, tokens, permissions) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(name)
        .bind(pw)
        .bind("[]")
        .bind(permissions.bits())
        .execute(&mut *conn)
        .await;

        println!("created user {} ({}) with permissions:", name, id);

        for flag in Permissions::all().iter_names() {
            println!(
                "\t{} ({}): {}",
                flag.0.to_lowercase(),
                flag.1.bits(),
                permissions.contains(flag.1)
            );
        }
        true
    } else {
        println!("unable to open {url}");
        false
    }
}
