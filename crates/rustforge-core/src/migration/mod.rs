//! Moteur de Migration Universelle & DataMorph pour RustForge.
//! Désérialisation PHP sérialisée, format pivot TMIR et ingestion WXR streaming.

pub mod capsule;
pub mod exporter;
pub mod importer;
pub mod native_backup;
pub mod php_serializer;
pub mod tmir;
pub mod wxr_importer;

pub use capsule::{CapsuleManager, CapsuleManifest, CapsulePackReport, CapsuleUnpackReport};
pub use exporter::DataMorphExporter;
pub use importer::DataMorphImporter;
pub use php_serializer::{
    PhpDeserializer, PhpParseError, is_likely_php_serialized, maybe_deserialize_php,
};
pub use tmir::{
    RFIRDocument, RFIREntry, RFIREntryStatus, RFIRImportReport, RFIRMedia, RFIRTerm, TMIRDocument,
    TMIREntry, TMIREntryStatus, TMIRImportReport, TMIRMedia, TMIRTerm,
};
pub use wxr_importer::WxrImporter;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_wxr_xml_parsing_with_php_serialized_meta() {
        let mock_wxr = r#"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:wp="http://wordpress.org/export/1.2/">
  <channel>
    <title>Mon Blog WordPress</title>
    <link>https://mon-site-wp.fr</link>
    <wp:category>
      <wp:category_nicename>tutoriels</wp:category_nicename>
      <wp:cat_name>Tutoriels</wp:cat_name>
    </wp:category>
    <item>
      <title>Bienvenue sur WordPress</title>
      <link>https://mon-site-wp.fr/bienvenue/</link>
      <dc:creator>admin</dc:creator>
      <content:encoded><![CDATA[<!-- wp:paragraph --><p>Voici le premier article.</p><!-- /wp:paragraph -->]]></content:encoded>
      <wp:post_id>42</wp:post_id>
      <wp:post_date>2026-10-27 10:00:00</wp:post_date>
      <wp:post_name>bienvenue-sur-wordpress</wp:post_name>
      <wp:status>publish</wp:status>
      <wp:post_type>post</wp:post_type>
      <category domain="category" nicename="tutoriels"><![CDATA[Tutoriels]]></category>
      <wp:postmeta>
        <wp:meta_key>_reading_time</wp:meta_key>
        <wp:meta_value><![CDATA[3]]></wp:meta_value>
      </wp:postmeta>
      <wp:postmeta>
        <wp:meta_key>_complex_wp_settings</wp:meta_key>
        <wp:meta_value><![CDATA[a:2:{s:6:"layout";s:7:"sidebar";s:5:"stats";a:2:{i:0;i:10;i:1;i:20;}}]]></wp:meta_value>
      </wp:postmeta>
    </item>
  </channel>
</rss>"#;

        let cursor = Cursor::new(mock_wxr.as_bytes());
        let doc = WxrImporter::parse(cursor).expect("Le parsing WXR doit réussir");

        assert_eq!(
            doc.source_site_url,
            Some("https://mon-site-wp.fr".to_string())
        );
        assert_eq!(doc.terms.len(), 1);
        assert_eq!(doc.terms[0].slug, "tutoriels");
        assert_eq!(doc.terms[0].name, "Tutoriels");

        assert_eq!(doc.entries.len(), 1);
        let entry = &doc.entries[0];
        assert_eq!(entry.title, "Bienvenue sur WordPress");
        assert_eq!(entry.slug, "bienvenue-sur-wordpress");
        assert_eq!(entry.content_type, "post");
        assert_eq!(entry.status, TMIREntryStatus::Published);
        assert_eq!(entry.categories, vec!["Tutoriels"]);

        // Vérification du coffre-fort _legacy_vault et de la désérialisation PHP
        assert!(entry.legacy_vault.contains_key("_complex_wp_settings"));
        let deserialized_meta = entry.legacy_vault.get("_complex_wp_settings").unwrap();

        assert_eq!(
            deserialized_meta,
            &serde_json::json!({
                "layout": "sidebar",
                "stats": [10, 20]
            })
        );
    }

    #[test]
    fn test_sovereign_rf_xml_and_rfir_json_export() {
        let mut doc = RFIRDocument {
            source_generator: "RustForge Sovereign Exporter 1.0".into(),
            source_site_url: Some("https://rustforge.dev".into()),
            ..Default::default()
        };

        doc.terms.push(RFIRTerm {
            source_id: "term-1".to_string(),
            taxonomy: "rf_category".to_string(),
            slug: "rust".to_string(),
            name: "Rust & Sécurité".to_string(),
            description: Some("Tutoriels Rust".to_string()),
            parent_slug: None,
        });

        let mut vault = std::collections::BTreeMap::new();
        vault.insert("_rf_sovereign_score".to_string(), serde_json::json!(100));

        doc.entries.push(RFIREntry {
            source_id: "entry-1".to_string(),
            slug: "pourquoi-rustforge".to_string(),
            title: "Pourquoi choisir RustForge CMS".to_string(),
            content_type: "post".to_string(),
            status: RFIREntryStatus::Published,
            raw_content: "<!-- rf:heading --><h2>Pure Performance</h2><!-- /rf:heading -->"
                .to_string(),
            excerpt: Some("Découvrez le CMS souverain".to_string()),
            author_name: Some("Équipe RustForge".to_string()),
            author_email: Some("contact@rustforge.dev".to_string()),
            published_at: Some(chrono::Utc::now()),
            categories: vec!["Rust & Sécurité".to_string()],
            tags: vec!["Performance".to_string(), "Zéro CVE".to_string()],
            featured_image_url: Some("/uploads/2026/10/banner.webp".to_string()),
            legacy_vault: vault,
        });

        // 1. Test RFIR-JSON
        let json_str = DataMorphExporter::to_json(&doc).expect("Export JSON doit réussir");
        assert!(json_str.contains("\"source_generator\": \"RustForge Sovereign Exporter 1.0\""));
        assert!(json_str.contains("\"pourquoi-rustforge\""));

        // 2. Test RF-XML
        let xml_str = DataMorphExporter::to_rf_xml(&doc).expect("Export RF-XML doit réussir");
        assert!(xml_str.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml_str.contains(
            "<rustforge version=\"1.0\" xmlns:rf=\"https://rustforge.dev/export/1.0/\">"
        ));
        assert!(xml_str.contains("<rf:taxonomy>rf_category</rf:taxonomy>"));
        assert!(xml_str.contains("<rf:category><![CDATA[Rust & Sécurité]]></rf:category>"));
        assert!(xml_str.contains("<rf:tag><![CDATA[Performance]]></rf:tag>"));
        assert!(
            xml_str.contains("<!-- rf:heading --><h2>Pure Performance</h2><!-- /rf:heading -->")
        );
        assert!(xml_str.contains("<rf:vault><![CDATA[{\"_rf_sovereign_score\":100}]]></rf:vault>"));
        assert!(!xml_str.contains("wp:"));
        assert!(!xml_str.contains("wordpress"));
    }

    #[test]
    fn test_capsule_manifest_and_zip_structure() {
        let manifest = CapsuleManifest {
            version: "1.0".to_string(),
            generator: "RustForge Capsule Engine 1.0".to_string(),
            created_at: chrono::Utc::now(),
            source_site_url: Some("https://mysite.local".to_string()),
            entry_count: 5,
            term_count: 2,
            media_count: 10,
        };

        let temp_capsule =
            std::env::temp_dir().join(format!("test_capsule_{}.rfcapsule", uuid::Uuid::new_v4()));
        let file = std::fs::File::create(&temp_capsule).expect("create file");
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("capsule.json", options)
            .expect("start capsule.json");
        let manifest_bytes = serde_json::to_vec_pretty(&manifest).expect("serialize manifest");
        std::io::Write::write_all(&mut zip, &manifest_bytes).expect("write manifest");

        let doc = RFIRDocument::default();
        zip.start_file("database.rfir.json", options)
            .expect("start database.rfir.json");
        let doc_bytes = serde_json::to_vec_pretty(&doc).expect("serialize doc");
        std::io::Write::write_all(&mut zip, &doc_bytes).expect("write doc");

        zip.finish().expect("finish zip");

        // A legacy editorial capsule must never be accepted as a native full backup.
        assert!(CapsuleManager::inspect(std::fs::File::open(&temp_capsule).unwrap()).is_err());

        // Vérification de la lecture
        let read_file = std::fs::File::open(&temp_capsule).expect("open zip");
        let mut archive = zip::ZipArchive::new(read_file).expect("archive");
        assert!(archive.by_name("capsule.json").is_ok());
        assert!(archive.by_name("database.rfir.json").is_ok());

        let _ = std::fs::remove_file(&temp_capsule);
    }
}
