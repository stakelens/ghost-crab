#[macro_export]
macro_rules! create_handler {
    ($name: ident, $handler: expr) => {
        #[derive(Clone)]
        pub struct $name;

        impl $name {
            pub fn new() -> Box<Self> {
                Box::new(Self)
            }
        }

        #[async_trait]
        impl Handleable for $name {
            async fn handle(&self, params: Context) {
                $handler(params).await;
            }
        }
    };
}
