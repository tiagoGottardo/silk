use std::env::consts::OS;
use std::io;
use std::process::Command;

pub const VIDEO_DOWNLOAD_PATH: &str = "~/Videos/";
pub const AUDIO_DOWNLOAD_PATH: &str = "~/Music/";

pub async fn play_video_command(stream_url: String) -> io::Result<()> {
    let result = match OS {
        "linux" => {
            Command::new("sh")
                .arg("-c")
                .arg(format!("mpv '{}' > /dev/null", stream_url))
                .status()?;
        }
        "windows" => {
            Command::new("powershell")
                .arg("-Command")
                .arg(format!("mpv '{}' > $null", stream_url))
                .status()?;
        }
        _ => {
            panic!("Operating System not supported!")
        }
    };

    Ok(result)
}

pub mod env {
    use std::path::PathBuf;

    fn get_dotenv_path() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".env")
            .to_str()
            .expect(".env is not found in the correct directory!")
            .to_string()
    }

    pub struct Env {
        pub database_url: String,
    }

    impl Env {
        pub fn init() {
            dotenvy::from_path(get_dotenv_path()).expect(".env file not found");
        }

        pub fn get() -> Self {
            Self {
                database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL is void"),
            }
        }
    }
}

pub mod db {
    use super::env::Env;
    use sqlx::SqlitePool;
    use tokio::sync::OnceCell;

    static DB: OnceCell<SqlitePool> = OnceCell::const_new();

    pub async fn init() {
        let pool = SqlitePool::connect(&Env::get().database_url)
            .await
            .expect("Some error occur on database connection");

        DB.set(pool).unwrap();
    }

    pub fn get() -> SqlitePool {
        DB.get().expect("Database has not been initialized").clone()
    }
}
