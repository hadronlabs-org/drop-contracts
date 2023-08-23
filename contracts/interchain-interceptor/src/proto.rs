pub mod cosmos {
    pub mod base {
        pub mod query {
            pub mod v1beta1 {
                include!("./proto/cosmos.base.query.v1beta1.rs");
            }
        }
        pub mod v1beta1 {
            include!("./proto/cosmos.base.v1beta1.rs");
        }
    }
}

pub mod liquidstaking {
    pub mod distribution {
        pub mod v1beta1 {
            include!("./proto/liquidstaking.distribution.v1beta1.rs");
        }
    }
    pub mod staking {
        pub mod v1beta1 {
            include!("./proto/liquidstaking.staking.v1beta1.rs");
        }
    }
}
