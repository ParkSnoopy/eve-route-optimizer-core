use scraper::{ Html, Selector };

use std::sync::LazyLock;

use crate::{
    manager::RouteOptimizeManager,

    config,
    trace,
    route::RouteOption,
    system::SystemPair,
};



static SEL_0: LazyLock<Selector> = LazyLock::new(|| Selector::parse(r#"div[id="navtools"]"#).unwrap());
static SEL_1: LazyLock<Selector> = LazyLock::new(|| Selector::parse(r#"table[class="tablelist table-tooltip"]"#).unwrap());
static SEL_2: LazyLock<Selector> = LazyLock::new(|| Selector::parse(r#"tr"#).unwrap());
static SEL_3: LazyLock<Selector> = LazyLock::new(|| Selector::parse(r#"td"#).unwrap());

impl RouteOptimizeManager {
    pub fn make_url(&self, system_pair: &SystemPair) -> String {
        format!("{}{}{}:{}{}",
            config::ROUTE_SEARCH_URL_PREFIX,
            match self.args().route_option {
                RouteOption::Fastest => "",
                RouteOption::Highsec => "2:",
                RouteOption::LowNull => "3:",
            },
            system_pair.left().read().unwrap().name(),
            system_pair.right().read().unwrap().name(),
            config::ROUTE_SEARCH_URL_POSTFIX,
        )
    }

    pub fn parse_text_into_length(&self, text: &String) -> u64 {
        //trace::debug(format!("Parsing text from request: {}", &text[..20]));
        let distance: u64 = Html::parse_document(text)
            .select(&SEL_0)
            .next()
            .expect(&trace::string::error("Unexpected response format"))
            .select(&SEL_1)
            .next()
            .expect(&trace::string::error("System Name Invalid"))
            .select(&SEL_2)
            .last()
            .unwrap()
            .select(&SEL_3)
            .next()
            .unwrap()
            .inner_html()
            .replace('.', "")
            .trim()
            .parse()
            .expect(&trace::string::error("Failed to parse route length"));

        distance - 1 // route start from self
    }
}
