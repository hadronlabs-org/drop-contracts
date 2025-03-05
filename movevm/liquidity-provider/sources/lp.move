module me::drop_lp {
    use initia_std::object::{Self, Object, ExtendRef};
    use initia_std::cosmos;
    use initia_std::json;
    use initia_std::oracle;
    use initia_std::bigdecimal::{Self, BigDecimal};
    use initia_std::math128;
    use initia_std::dex;
    use initia_std::fungible_asset;
    use initia_std::block;
    use initia_std::coin;

    use std::string::{Self, String};
    use std::option::{Self, Option};
    use std::error;
    use std::vector;
    use std::signer;
    use std::event;
    use std::address;

    const ENO_NAME: u64 = 1;
    const ENO_SLINKY_PAIR: u64 = 2;
    const ENAME_TOO_LONG: u64 = 3;
    const EUNAUTHORIZED: u64 = 4;

    const NAME_MAX_LENGTH: u64 = 64;
    const PROVIDE_CB_ID: u64 = 1;

    struct LpConfig has key {
        extend_ref: ExtendRef,
        name: String,
        backup: address,
        slinky_pair: String,
        pair: Object<dex::Config>,
        asset: Object<fungible_asset::Metadata>,
        recipient: address,
        price: u256,
        ts: u64,
        decimals: u64,
    }

    #[event]
    struct CreateLpEvent has drop {
        lp_address: address,
        name: String,
        backup: address,
        slinky_pair: String,
        pair: Object<dex::Config>,
        asset: Object<fungible_asset::Metadata>,
        recipient: address,
    }

    #[event]
    struct DecisionEvent has copy, drop {
        callback_success: bool,
        slinky_price: Option<BigDecimal>,
        pool_price: Option<BigDecimal>,
        ratio: Option<BigDecimal>,
        will_provide_liquidity: bool,
    }

    #[event]
    struct StorePriceEvent has drop {
        block_ts: u64,
        price: u256,
        ts: u64,
        decimals: u64,
    }

    #[event]
    struct ProvideLiquidityEvent has drop {
        amount_in: u64,
        amount_out: u64,
        recipient: address,
    }

    #[event]
    struct BackupEvent has drop {
        coin: Object<fungible_asset::Metadata>,
        amount_out: u64,
        recipient: address,
    }

    struct MsgExecuteJSON has drop {
        _type_: String,
        sender: String,
        module_address: String,
        module_name: String,
        function_name: String,
        type_args: vector<String>,
        args: vector<String>,
    }

    public entry fun create_liquidity_provider(
        account: &signer,
        name: String, // just a name for a visual reference, e.g. "testnet_uinit"
        backup: Option<address>, // priviliged account with a right to withdraw any coins from module object,
        //                          will be set to `account` if omitted
        slinky_pair: String, // name of a slinky pair, e.g. "INIT/USD"
        pair: Object<dex::Config>, // address of liquidity pool, e.g. 0xdbf06c48af3984ec6d9ae8a9aa7dbb0bb1e784aa9b8c4a5681af660cf8558d7d
        //                            for uinit-usdc on initiation-2 testnet
        asset: Object<fungible_asset::Metadata>, // address of asset, e.g. 0x8e4733bdabcf7d4afc3d14f0dd46c9bf52fb0fce9e4b996c939e195b8bc891d9
        //                                          for uinit on initiation-2 testnet
        recipient: address, // address which will be receiving all LP tokens
    ) {
        assert!(
            string::length(&slinky_pair) > 0,
            error::invalid_argument(ENO_SLINKY_PAIR),
        );

        assert!(
            string::length(&name) > 0,
            error::invalid_argument(ENO_NAME),
        );
        assert!(
            string::length(&name) <= NAME_MAX_LENGTH,
            error::out_of_range(ENAME_TOO_LONG),
        );

        let seed = b"drop_lp_";
        vector::append(&mut seed, *string::bytes(&name));

        let constructor_ref = object::create_named_object(account, seed);
        let extend_ref = object::generate_extend_ref(&constructor_ref);
        let lp_signer = object::generate_signer_for_extending(&extend_ref);
        let lp_address = signer::address_of(&lp_signer);

        let backup = option::destroy_with_default(
            backup,
            signer::address_of(account),
        );

        move_to(
            &lp_signer,
            LpConfig {
                extend_ref,
                name,
                backup,
                slinky_pair,
                pair,
                asset,
                recipient,
                price: 0,
                ts: 0,
                decimals: 0,
            },
        );

        event::emit(
            CreateLpEvent {
                lp_address,
                name,
                backup,
                slinky_pair,
                pair,
                asset,
                recipient,
            }
        );
    }

    // emits stargate message which:
    // 1. calls @me::drop_lp::store()
    // 2.1. then calls @me::drop_lp::callback(..., ..., false) if store() fails
    // 2.2. then calls @me::drop_lp::callback(..., ..., true) if store() succeeds
    public entry fun provide(
        lp_address: address, // address of drop_lp instance object
    ) acquires LpConfig {
        let lp_config = borrow_global<LpConfig>(lp_address);
        let lp_signer = object::generate_signer_for_extending(&lp_config.extend_ref);
        let lp_address = signer::address_of(&lp_signer);

        let msg = MsgExecuteJSON {
            _type_: string::utf8(b"/initia.move.v1.MsgExecuteJSON"),
            sender: address::to_sdk(lp_address),
            module_address: address::to_sdk(@me),
            module_name: string::utf8(b"drop_lp"),
            function_name: string::utf8(b"store"),
            type_args: vector[],
            args: vector[address::to_string(lp_address)],
        };

        let cb = address::to_string(@me);
        string::append(&mut cb, string::utf8(b"::drop_lp::callback"));

        cosmos::stargate_with_options(
            &lp_signer,
            json::marshal(&msg),
            cosmos::allow_failure_with_callback(PROVIDE_CB_ID, cb),
        );
    }

    // Last resort function only available for a specially designated backup address to withdraw all funds
    // of `coin` denomination from module's object address in case if there is an unrecoverable bug somewhere
    public entry fun backup(
        account: &signer,
        lp_address: address, // address of drop_lp instance object
        coin: Object<fungible_asset::Metadata>, // address of coin object to withdraw
    ) acquires LpConfig {
        let lp_config = borrow_global<LpConfig>(lp_address);

        assert!(
            signer::address_of(account) == lp_config.backup,
            error::permission_denied(EUNAUTHORIZED),
        );

        let lp_signer = object::generate_signer_for_extending(&lp_config.extend_ref);
        let amount_out = coin::balance(signer::address_of(&lp_signer), coin);
        coin::transfer(&lp_signer, lp_config.backup, coin, amount_out);
        event::emit(BackupEvent {
            coin,
            amount_out,
            recipient: lp_config.backup,
        });
    }

    fun provide_liquidity(
        lp_signer: &signer,
        lp_config: &LpConfig,
    ) {
        // 1. provide liquidity
        let amount_in = coin::balance(
            signer::address_of(lp_signer),
            lp_config.asset,
        );
        dex::single_asset_provide_liquidity_script(
            lp_signer,
            lp_config.pair,
            lp_config.asset,
            amount_in,
            option::none(),
        );

        // 2. sweep all received LP tokens to the configured recipient
        let metadata_out = object::address_to_object(object::object_address(&lp_config.pair));
        let amount_out = coin::balance(
            signer::address_of(lp_signer),
            metadata_out,
        );
        coin::transfer(lp_signer, lp_config.recipient, metadata_out, amount_out);
        event::emit(ProvideLiquidityEvent {
            amount_in,
            amount_out,
            recipient: lp_config.recipient,
        });
    }

    // Read price from slinky and store it in module storage
    entry fun store(
        lp_address: address,
    ) acquires LpConfig {
        let lp_config = borrow_global_mut<LpConfig>(lp_address);
        let block_ts = block::get_current_block_timestamp();
        if (block_ts > lp_config.ts) {
            let (price, ts, decimals) = oracle::get_price(lp_config.slinky_pair);
            lp_config.price = price;
            lp_config.ts = ts;
            lp_config.decimals = decimals;
            event::emit(StorePriceEvent {
                block_ts,
                price,
                ts,
                decimals,
            });
        } else {
            event::emit(StorePriceEvent {
                block_ts,
                price: 0,
                ts: 0,
                decimals: 0,
            });
        }
    }

    // MEV protection, only provide liquidity if pool price is up to date with off-chain price feed
    entry fun callback(
        account: &signer, // this internal call should originate from module object
        _id: u64, // unused since we always use the same ID anyway
        success: bool, // did an attempt to read price from slinky succeed?
    ) acquires LpConfig {
        // security model: if signer has LpConfig, that means this signer is legitimate
        let lp_config = borrow_global<LpConfig>(signer::address_of(account));
        let lp_signer = object::generate_signer_for_extending(&lp_config.extend_ref);

        let event = DecisionEvent {
            callback_success: success,
            slinky_price: option::none(),
            pool_price: option::none(),
            ratio: option::none(),
            will_provide_liquidity: false,
        };

        if (success) {
            // warning: this algorithm assumes we are working with an X/USD pool
            //          and it WILL fail horribly with any other pool
            let slinky_price = bigdecimal::from_ratio_u256(
                lp_config.price,
                (math128::pow(10, (lp_config.decimals as u128)) as u256),
            );
            let pool_price = dex::get_spot_price(
                lp_config.pair,
                lp_config.asset,
            );
            let ratio = if (bigdecimal::gt(slinky_price, pool_price)) {
                bigdecimal::div(slinky_price, pool_price)
            } else {
                bigdecimal::div(pool_price, slinky_price)
            };
            let block_ts = block::get_current_block_timestamp();
            event.slinky_price = option::some(slinky_price);
            event.pool_price = option::some(pool_price);
            event.ratio = option::some(ratio);
            if (
                // slinky price is up to date
                lp_config.ts == block_ts
                    &&
                    // pool price is not more than 1% off
                    bigdecimal::le(ratio, bigdecimal::from_ratio_u256(101, 100))
            ) {
                event.will_provide_liquidity = true;
            }
        } else {
            // slinky doesn't know about INIT yet, skip any validation and go full YOLO
            event.will_provide_liquidity = true;
        };

        if (event.will_provide_liquidity) {
            provide_liquidity(&lp_signer, lp_config);
        };
        event::emit(event);
    }
}
