pub mod cosmos {
    pub mod base {
        pub mod query {
            pub mod v1beta1 {
                include!("./cosmos.base.query.v1beta1.rs");
            }
        }
        pub mod v1beta1 {
            include!("./cosmos.base.v1beta1.rs");
        }
    }
    pub mod staking {
        pub mod v1beta1 {
            include!("./cosmos.staking.v1beta1.rs");
        }
    }
}

pub mod gaia {
    pub mod liquid {
        pub mod module {
            include!("./gaia.liquid.module.v1.rs");
        }
        pub mod v1beta1 {
            include!("./gaia.liquid.v1beta1.rs");
        }
    }
}

pub mod liquidstaking {
    pub mod distribution {
        pub mod v1beta1 {
            include!("./liquidstaking.distribution.v1beta1.rs");
        }
    }
    pub mod staking {
        pub mod v1beta1 {
            include!("./liquidstaking.staking.v1beta1.rs");
        }
    }
}

pub mod initia {
    pub mod mstaking {
        pub mod v1 {
            include!("./initia.mstaking.v1.rs");
        }
    }
}
