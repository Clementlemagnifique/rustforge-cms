//! Creates a new installation configuration, without replacing an existing one.
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

pub fn configuration(database_url: &str) -> anyhow::Result<String> {
    use std::str::FromStr;
    sqlx::postgres::PgConnectOptions::from_str(database_url).map_err(|_| {
        anyhow::anyhow!(
            "Adresse PostgreSQL invalide. Utilisez postgres://utilisateur:mot-de-passe@hote/base."
        )
    })?;
    let mut config = toml::Table::new();
    config.insert(
        "database".into(),
        toml::Value::Table(toml::Table::from_iter([(
            "url".into(),
            toml::Value::String(database_url.into()),
        )])),
    );
    config.insert(
        "server".into(),
        toml::Value::Table(toml::Table::from_iter([
            ("host".into(), toml::Value::String("127.0.0.1".into())),
            ("port".into(), toml::Value::Integer(8080)),
            (
                "jwt_secret".into(),
                toml::Value::String(format!(
                    "{}{}",
                    uuid::Uuid::new_v4().simple(),
                    uuid::Uuid::new_v4().simple()
                )),
            ),
        ])),
    );
    config.insert(
        "storage".into(),
        toml::Value::Table(toml::Table::from_iter([
            ("uploads_dir".into(), toml::Value::String("uploads".into())),
            ("plugins_dir".into(), toml::Value::String("plugins".into())),
            ("themes_dir".into(), toml::Value::String("themes".into())),
            (
                "protected_downloads_dir".into(),
                toml::Value::String("protected_downloads".into()),
            ),
            (
                "admin_dist".into(),
                toml::Value::String("admin/dist".into()),
            ),
        ])),
    );
    Ok(toml::to_string_pretty(&config)?)
}

pub async fn initialize(root: &Path, database_url: &str) -> anyhow::Result<()> {
    let path = root.join("config.toml");
    if path.exists() {
        anyhow::bail!("Une configuration existe déjà. Elle est conservée. Lancez rustforge serve.");
    }
    if !root.join("admin/dist/index.html").is_file() {
        anyhow::bail!(
            "Le Studio compilé est absent. Utilisez le paquet complet ou compilez admin avec npm run build."
        );
    }
    let config = configuration(database_url)?;
    let pool=rustforge_core::connect(database_url).await.map_err(|_|anyhow::anyhow!("Connexion PostgreSQL impossible. Vérifiez la base et les identifiants ; aucune configuration n'a été créée."))?;
    rustforge_core::migrate(&pool).await.map_err(|_|anyhow::anyhow!("Initialisation des tables impossible. Vérifiez les droits PostgreSQL ; aucune configuration n'a été créée."))?;
    pool.close().await;
    for dir in ["uploads", "plugins", "protected_downloads"] {
        fs::create_dir_all(root.join(dir))?;
    }
    // create_new also protects against a concurrent initialization.
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&path)?;
    if let Err(error) = file
        .write_all(config.as_bytes())
        .and_then(|_| file.sync_all())
    {
        drop(file);
        let _ = fs::remove_file(&path);
        return Err(error.into());
    }
    println!(
        "RustForge CMS est prêt. Configuration personnelle créée ; les secrets ne sont pas affichés."
    );
    println!("Lancez rustforge serve puis ouvrez le lien d'installation indiqué.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_configuration_escapes_credentials_and_uses_unique_secrets() {
        let url = "postgres://alice:pass%22word@localhost/site";
        let a: toml::Value = toml::from_str(&configuration(url).unwrap()).unwrap();
        let b: toml::Value = toml::from_str(&configuration(url).unwrap()).unwrap();
        assert_eq!(a["database"]["url"].as_str(), Some(url));
        assert_eq!(a["server"]["jwt_secret"].as_str().unwrap().len(), 64);
        assert_ne!(a["server"]["jwt_secret"], b["server"]["jwt_secret"]);
        assert_eq!(a["server"]["host"].as_str(), Some("127.0.0.1"));
        assert!(configuration("not a database connection string").is_err());
    }
    #[tokio::test]
    async fn existing_configuration_is_never_overwritten() {
        let root = std::env::temp_dir().join(format!("trustpress-init-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("config.toml"), b"keep-me").unwrap();
        assert!(initialize(&root, "invalid").await.is_err());
        assert_eq!(fs::read(root.join("config.toml")).unwrap(), b"keep-me");
    }
}
