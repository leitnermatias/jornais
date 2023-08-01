pub mod model;
pub mod newspapers;

#[cfg(test)]
mod tests {

    use crate::newspapers;

    #[tokio::test]
    async fn newspaper_clarin() {
        let news = newspapers::get_clarin().await;

        assert!(!news.is_empty())
    }

    #[tokio::test]
    async fn newspaper_infobae() {
        let news = newspapers::get_infobae().await;

        assert!(!news.is_empty())
    }

    #[tokio::test]
    async fn newspaper_lanacion() {
        let news = newspapers::get_lanacion().await;
        assert!(!news.is_empty())
    }
}