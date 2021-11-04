// Test mongo models
// Run : cargo test -- --test-threads=1 Because of async functions
//
// TODO write all tests

#[cfg(test)]
mod test {
    use crate::model::mongo;

    #[tokio::test]
    async fn program() {
        // Drop test database if existed
        mongo::get_db().await.drop(None).await.unwrap();

        //  Minimum fields
        let mut min = mongo::Program {
            bounty: None,
            handle: None,
            icon: None,
            name: "Test".to_string(),
            platform: None,
            scopes: vec!["test.s1".to_string(), "test.s2".to_string()],
            state: None,
            ty: None,
            update: None,
            url: None,
        };

        program_check(min.clone(), min.clone()).await;

        // Full fields
        let mut full = mongo::Program {
            bounty: Some("Test_b".to_string()),
            handle: Some("Test_h".to_string()),
            icon: Some("Test_i".to_string()),
            name: "Test".to_string(),
            platform: Some(mongo::ProgramPlatform::Anonymous),
            scopes: vec![
                "test.s1".to_string(),
                "test.s2".to_string(),
                "test.s3".to_string(),
            ],
            state: Some(mongo::ProgramState::Open),
            ty: Some(mongo::ProgramType::Public),
            update: None,
            url: Some("url.com".to_string()),
        };
        program_check(full.clone(), full.clone()).await;

        // Some fields
        min.url = Some("url2.com".to_string());
        full.url = Some("url2.com".to_string());
        program_check(min.clone(), full.clone()).await;

        // Appended
        let scopes = mongo::Scope::find(None, None, None).await;
        assert_eq!(
            scopes.into_iter().map(|s| s.asset).collect::<Vec<String>>(),
            vec!["test.s1", "test.s2", "test.s3"]
        );
    }

    async fn program_check(doc: mongo::Program, expected_doc: mongo::Program) {
        let filter = Some(r#"{"name":"Test"}"#.to_string());
        let limit = None;
        let sort = None;

        doc.clone().update().await;

        // Find and assert
        let mut docs = mongo::Program::find(filter.clone(), limit.clone(), sort.clone()).await;
        docs[0].update = None;
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0], expected_doc);
    }

    #[tokio::test]
    async fn scope() {
        mongo::get_db().await.drop(None).await.unwrap();

        // Min
        let mut min = mongo::Scope {
            asset: "Test".to_string(),
            subs: vec!["test.s1".to_string(), "test.s2".to_string()],
            eligible_bounty: None,
            severity: None,
            program: "Test Program".to_string(),
            ty: None,
            update: None,
        };
        scope_check(min.clone(), min.clone()).await;

        // Full
        let mut full = mongo::Scope {
            asset: "Test".to_string(),
            subs: vec![
                "test.s1".to_string(),
                "test.s2".to_string(),
                "test.s3".to_string(),
            ],
            eligible_bounty: Some(true),
            severity: Some(mongo::ScopeSeverity::High),
            program: "Test Program".to_string(),
            ty: Some(mongo::ScopeType::WildcardDomain),
            update: None,
        };
        scope_check(full.clone(), full.clone()).await;

        // Some
        min.severity = Some(mongo::ScopeSeverity::Medium);
        full.severity = Some(mongo::ScopeSeverity::Medium);

        scope_check(min.clone(), full.clone()).await;

        // Appended
        let subs = mongo::Sub::find(None, None, None).await;
        assert_eq!(
            subs.into_iter().map(|s| s.asset).collect::<Vec<String>>(),
            vec!["test.s1", "test.s2", "test.s3"]
        );
        // mongo::get_db().await.drop(None).await.unwrap();
    }

    async fn scope_check(doc: mongo::Scope, expected_doc: mongo::Scope) {
        let filter = Some(r#"{"asset":"Test"}"#.to_string());
        let limit = None;
        let sort = None;

        doc.clone().update().await;

        // Find and assert
        let mut docs = mongo::Scope::find(filter.clone(), limit.clone(), sort.clone()).await;
        docs[0].update = None;
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0], expected_doc);
    }

    #[tokio::test]
    async fn sub() {
        mongo::get_db().await.drop(None).await.unwrap();

        // Min
        let mut min = mongo::Sub {
            asset: "Test".to_string(),
            scope: "Test Scope".to_string(),
            urls: vec!["test.s1".to_string(), "test.s2".to_string()],
            host: None,
            ty: None,
            update: None,
        };
        sub_check(min.clone(), min.clone()).await;

        // Full
        let mut full = mongo::Sub {
            asset: "Test".to_string(),
            scope: "Test Scope".to_string(),
            urls: vec![
                "test.s1".to_string(),
                "test.s2".to_string(),
                "test.s3".to_string(),
            ],
            host: Some("host".to_string()),
            ty: Some(mongo::SubType::IP),
            update: None,
        };
        sub_check(full.clone(), full.clone()).await;

        // Some
        min.host = Some("host".to_string());
        full.host = Some("host".to_string());

        sub_check(min.clone(), full.clone()).await;

        // Appended
        let urls = mongo::URL::find(None, None, None).await;
        assert_eq!(
            urls.into_iter().map(|s| s.url).collect::<Vec<String>>(),
            vec!["test.s1", "test.s2", "test.s3"]
        );
        // mongo::get_db().await.drop(None).await.unwrap();
    }

    async fn sub_check(doc: mongo::Sub, expected_doc: mongo::Sub) {
        let filter = Some(r#"{"asset":"Test"}"#.to_string());
        let limit = None;
        let sort = None;

        doc.clone().update().await;

        // Find and assert
        let mut docs = mongo::Sub::find(filter.clone(), limit.clone(), sort.clone()).await;
        docs[0].update = None;
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0], expected_doc);
    }

    #[tokio::test]
    async fn host() {
        mongo::get_db().await.drop(None).await.unwrap();

        // Min
        let min = mongo::Host {
            ip: "Test".to_string(),
            sub: "Test Sub".to_string(),
            services: vec!["test.s1".to_string(), "test.s2".to_string()],
            update: None,
        };
        host_check(min.clone(), min.clone()).await;

        // Full
        let full = mongo::Host {
            ip: "Test".to_string(),
            sub: "Test Sub".to_string(),
            services: vec![
                "test.s1".to_string(),
                "test.s2".to_string(),
                "test.s3".to_string(),
            ],
            update: None,
        };
        host_check(full.clone(), full.clone()).await;

        // Appended
        let services = mongo::Service::find(None, None, None).await;
        assert_eq!(
            services
                .into_iter()
                .map(|s| s.port)
                .collect::<Vec<String>>(),
            vec!["test.s1", "test.s2", "test.s3"]
        );
        // mongo::get_db().await.drop(None).await.unwrap();
    }

    async fn host_check(doc: mongo::Host, expected_doc: mongo::Host) {
        let filter = Some(r#"{"ip":"Test"}"#.to_string());
        let limit = None;
        let sort = None;

        doc.clone().update().await;

        // Find and assert
        let mut docs = mongo::Host::find(filter.clone(), limit.clone(), sort.clone()).await;
        docs[0].update = None;
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0], expected_doc);
    }

    #[tokio::test]
    async fn url() {
        mongo::get_db().await.drop(None).await.unwrap();

        // Min
        let mut min = mongo::URL {
            url: "Test".to_string(),
            sub: "Test Scope".to_string(),
            techs: vec!["test.s1".to_string(), "test.s2".to_string()],
            title: None,
            status_code: None,
            content_type: None,
            update: None,
        };
        url_check(min.clone(), min.clone()).await;

        // Full

        let mut full = mongo::URL {
            url: "Test".to_string(),
            sub: "Test Scope".to_string(),
            techs: vec![
                "test.s1".to_string(),
                "test.s2".to_string(),
                "test.s3".to_string(),
            ],
            title: Some("Test".to_string()),
            status_code: Some("Test".to_string()),
            content_type: Some("Test".to_string()),
            update: None,
        };

        url_check(full.clone(), full.clone()).await;

        // Some
        min.title = Some("Title".to_string());
        full.title = Some("Title".to_string());

        url_check(min.clone(), full.clone()).await;

        // Appended
        let techs = mongo::Tech::find(None, None, None).await;
        assert_eq!(
            techs.into_iter().map(|s| s.name).collect::<Vec<String>>(),
            vec!["test.s1", "test.s2", "test.s3"]
        );
        // mongo::get_db().await.drop(None).await.unwrap();
    }

    async fn url_check(doc: mongo::URL, expected_doc: mongo::URL) {
        let filter = Some(r#"{"url":"Test"}"#.to_string());
        let limit = None;
        let sort = None;

        doc.clone().update().await;

        // Find and assert
        let mut docs = mongo::URL::find(filter.clone(), limit.clone(), sort.clone()).await;
        docs[0].update = None;
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0], expected_doc);
    }

    #[tokio::test]
    async fn service() {
        mongo::get_db().await.drop(None).await.unwrap();

        // Min
        let mut min = mongo::Service {
            port: "Test".to_string(),
            protocol: None,
            banner: None,
        };
        service_check(min.clone(), min.clone()).await;

        // Full

        let mut full = mongo::Service {
            port: "Test".to_string(),
            protocol: Some("Test".to_string()),
            banner: Some("Test".to_string()),
        };

        service_check(full.clone(), full.clone()).await;

        // Some
        min.protocol = Some("Title".to_string());
        full.protocol = Some("Title".to_string());

        service_check(min.clone(), full.clone()).await;

        // mongo::get_db().await.drop(None).await.unwrap();
    }

    async fn service_check(doc: mongo::Service, expected_doc: mongo::Service) {
        let filter = Some(r#"{"port":"Test"}"#.to_string());
        let limit = None;
        let sort = None;

        doc.clone().update().await;

        // Find and assert
        let docs = mongo::Service::find(filter.clone(), limit.clone(), sort.clone()).await;
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0], expected_doc);
    }

    #[tokio::test]
    async fn tech() {
        mongo::get_db().await.drop(None).await.unwrap();

        // Min
        let min = mongo::Tech {
            name: "Test".to_string(),
            version: None,
        };
        tech_check(min.clone(), min.clone()).await;

        // Full

        let full = mongo::Tech {
            name: "Test".to_string(),
            version: Some("Version".to_string()),
        };

        tech_check(full.clone(), full.clone()).await;

        // mongo::get_db().await.drop(None).await.unwrap();
    }

    async fn tech_check(doc: mongo::Tech, expected_doc: mongo::Tech) {
        let filter = Some(r#"{"name":"Test"}"#.to_string());
        let limit = None;
        let sort = None;

        doc.clone().update().await;

        // Find and assert
        let docs = mongo::Tech::find(filter.clone(), limit.clone(), sort.clone()).await;
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0], expected_doc);
    }

    // TODO: Job
}
