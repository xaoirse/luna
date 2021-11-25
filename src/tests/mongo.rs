// Test mongo models
// Run : cargo test -- --test-threads=1 Because of async functions
//
// TODO write all tests

#[cfg(test)]
mod test {
    use crate::database::*;
    use crate::model::*;

    #[tokio::test]
    async fn program() {
        // Drop test database if existed
        get_db().await.drop(None).await.unwrap();

        //  Minimum fields
        let mut min = Program {
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
        let mut full = Program {
            bounty: Some("Test_b".to_string()),
            handle: Some("Test_h".to_string()),
            icon: Some("Test_i".to_string()),
            name: "Test".to_string(),
            platform: Some(crate::model::program::ProgramPlatform::Anonymous),
            scopes: vec![
                "test.s1".to_string(),
                "test.s2".to_string(),
                "test.s3".to_string(),
            ],
            state: Some(crate::model::program::ProgramState::Open),
            ty: Some(crate::model::program::ProgramType::Public),
            update: None,
            url: Some("url.com".to_string()),
        };
        program_check(full.clone(), full.clone()).await;

        // Some fields
        min.url = Some("url2.com".to_string());
        full.url = Some("url2.com".to_string());
        program_check(min.clone(), full.clone()).await;

        // Appended
        let scopes = find_as_vec::<Scope>(None, None, None).await;
        assert_eq!(
            scopes.into_iter().map(|s| s.asset).collect::<Vec<String>>(),
            vec!["test.s1", "test.s2", "test.s3"]
        );
    }

    async fn program_check(doc: Program, expected_doc: Program) {
        let filter = Some(r#"{"name":"Test"}"#.to_string());
        let limit = None;
        let sort = None;

        update(doc).await;

        // Find and assert
        let mut docs = find_as_vec::<Program>(filter.clone(), limit.clone(), sort.clone()).await;
        docs[0].update = None;
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0], expected_doc);
    }

    #[tokio::test]
    async fn scope() {
        get_db().await.drop(None).await.unwrap();

        // Min
        let mut min = Scope {
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
        let mut full = Scope {
            asset: "Test".to_string(),
            subs: vec![
                "test.s1".to_string(),
                "test.s2".to_string(),
                "test.s3".to_string(),
            ],
            eligible_bounty: Some(true),
            severity: Some(crate::model::scope::ScopeSeverity::High),
            program: "Test Program".to_string(),
            ty: Some(crate::model::scope::ScopeType::WildcardDomain),
            update: None,
        };
        scope_check(full.clone(), full.clone()).await;

        // Some
        min.severity = Some(crate::model::scope::ScopeSeverity::Medium);
        full.severity = Some(crate::model::scope::ScopeSeverity::Medium);

        scope_check(min.clone(), full.clone()).await;

        // Appended
        let subs = find_as_vec::<Sub>(None, None, None).await;
        assert_eq!(
            subs.into_iter().map(|s| s.asset).collect::<Vec<String>>(),
            vec!["test.s1", "test.s2", "test.s3"]
        );
        // get_db().await.drop(None).await.unwrap();
    }

    async fn scope_check(doc: Scope, expected_doc: Scope) {
        let filter = Some(r#"{"asset":"Test"}"#.to_string());
        let limit = None;
        let sort = None;

        update(doc).await;

        // Find and assert
        let mut docs = find_as_vec::<Scope>(filter.clone(), limit.clone(), sort.clone()).await;
        docs[0].update = None;
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0], expected_doc);
    }

    #[tokio::test]
    async fn sub() {
        get_db().await.drop(None).await.unwrap();

        // Min
        let mut min = Sub {
            asset: "Test".to_string(),
            scope: "Test Scope".to_string(),
            urls: vec!["test.s1".to_string(), "test.s2".to_string()],
            host: None,
            ty: None,
            update: None,
        };
        sub_check(min.clone(), min.clone()).await;

        // Full
        let mut full = Sub {
            asset: "Test".to_string(),
            scope: "Test Scope".to_string(),
            urls: vec![
                "test.s1".to_string(),
                "test.s2".to_string(),
                "test.s3".to_string(),
            ],
            host: Some("host".to_string()),
            ty: Some(crate::model::sub::SubType::IP),
            update: None,
        };
        sub_check(full.clone(), full.clone()).await;

        // Some
        min.host = Some("host".to_string());
        full.host = Some("host".to_string());

        sub_check(min.clone(), full.clone()).await;

        // Appended
        let urls = find_as_vec::<URL>(None, None, None).await;
        assert_eq!(
            urls.into_iter().map(|s| s.url).collect::<Vec<String>>(),
            vec!["test.s1", "test.s2", "test.s3"]
        );
        // get_db().await.drop(None).await.unwrap();
    }

    async fn sub_check(doc: Sub, expected_doc: Sub) {
        let filter = Some(r#"{"asset":"Test"}"#.to_string());
        let limit = None;
        let sort = None;

        update(doc).await;

        // Find and assert
        let mut docs = find_as_vec::<Sub>(filter.clone(), limit.clone(), sort.clone()).await;
        docs[0].update = None;
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0], expected_doc);
    }

    #[tokio::test]
    async fn host() {
        get_db().await.drop(None).await.unwrap();

        // Min
        let min = Host {
            ip: "Test".to_string(),
            sub: "Test Sub".to_string(),
            services: vec!["test.s1".to_string(), "test.s2".to_string()],
            update: None,
        };
        host_check(min.clone(), min.clone()).await;

        // Full
        let full = Host {
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
        let services = find_as_vec::<Service>(None, None, None).await;
        assert_eq!(
            services
                .into_iter()
                .map(|s| s.port)
                .collect::<Vec<String>>(),
            vec!["test.s1", "test.s2", "test.s3"]
        );
        // get_db().await.drop(None).await.unwrap();
    }

    async fn host_check(doc: Host, expected_doc: Host) {
        let filter = Some(r#"{"ip":"Test"}"#.to_string());
        let limit = None;
        let sort = None;

        update(doc).await;

        // Find and assert
        let mut docs = find_as_vec::<Host>(filter.clone(), limit.clone(), sort.clone()).await;
        docs[0].update = None;
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0], expected_doc);
    }

    #[tokio::test]
    async fn url() {
        get_db().await.drop(None).await.unwrap();

        // Min
        let mut min = URL {
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

        let mut full = URL {
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
        let techs = find_as_vec::<Tech>(None, None, None).await;
        assert_eq!(
            techs.into_iter().map(|s| s.name).collect::<Vec<String>>(),
            vec!["test.s1", "test.s2", "test.s3"]
        );
        // get_db().await.drop(None).await.unwrap();
    }

    async fn url_check(doc: URL, expected_doc: URL) {
        let filter = Some(r#"{"url":"Test"}"#.to_string());
        let limit = None;
        let sort = None;

        update(doc).await;

        // Find and assert
        let mut docs = find_as_vec::<URL>(filter.clone(), limit.clone(), sort.clone()).await;
        docs[0].update = None;
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0], expected_doc);
    }

    #[tokio::test]
    async fn service() {
        get_db().await.drop(None).await.unwrap();

        // Min
        let mut min = Service {
            port: "Test".to_string(),
            protocol: None,
            banner: None,
        };
        service_check(min.clone(), min.clone()).await;

        // Full

        let mut full = Service {
            port: "Test".to_string(),
            protocol: Some("Test".to_string()),
            banner: Some("Test".to_string()),
        };

        service_check(full.clone(), full.clone()).await;

        // Some
        min.protocol = Some("Title".to_string());
        full.protocol = Some("Title".to_string());

        service_check(min.clone(), full.clone()).await;

        // get_db().await.drop(None).await.unwrap();
    }

    async fn service_check(doc: Service, expected_doc: Service) {
        let filter = Some(r#"{"port":"Test"}"#.to_string());
        let limit = None;
        let sort = None;

        update(doc).await;

        // Find and assert
        let docs = find_as_vec::<Service>(filter.clone(), limit.clone(), sort.clone()).await;
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0], expected_doc);
    }

    #[tokio::test]
    async fn tech() {
        get_db().await.drop(None).await.unwrap();

        // Min
        let min = Tech {
            name: "Test".to_string(),
            version: "".to_string(),
        };
        tech_check(min.clone(), min.clone()).await;

        // get_db().await.drop(None).await.unwrap();
    }

    async fn tech_check(doc: Tech, expected_doc: Tech) {
        let filter = Some(r#"{"name":"Test"}"#.to_string());
        let limit = None;
        let sort = None;

        update(doc).await;

        // Find and assert
        let docs = find_as_vec::<Tech>(filter.clone(), limit.clone(), sort.clone()).await;
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0], expected_doc);
    }

    // TODO: Job
}
