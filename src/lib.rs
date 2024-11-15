mod config;

mod cli;
mod request;

mod route;
mod system;

#[allow(unused)]
mod trace;
mod progress;

pub mod manager;



#[cfg(test)]
mod tests {

    use crate::*;
    use std::sync::{ Arc, RwLock };

    #[tokio::test]
    async fn test_manager() {
        let manager = manager::RouteOptimizeManager::new("-- --start Nisuwa --end Nisuwa --route Jita,Amarr,Hek,Rens,Dodixie --route-option low-null");

        let current_shortest: Arc<RwLock<system::CurrentShortest>> = manager.main().await.unwrap();
        let (length, routes) = current_shortest.read().unwrap().inner();

        let mut routes: Vec<Vec<String>> = routes
            .into_iter()
            .map(|sync_route| sync_route
                .into_iter()
                .map(|sync_system| sync_system.read().unwrap().name().clone())
                .collect())
            .collect();

        let mut routes_result = vec![
            vec!["DODIXIE", "AMARR", "RENS", "HEK", "JITA"],
            vec!["JITA", "HEK", "RENS", "AMARR", "DODIXIE"],
        ];

        routes.sort();
        routes_result.sort();

        assert_eq!(length, 109);
        assert_eq!(routes, routes_result);
    }
}
