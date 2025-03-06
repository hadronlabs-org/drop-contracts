module me::drop_lp {
    use initia_std::object::{Self, Object, ExtendRef};
    use initia_std::cosmos;
    use initia_std::json;
    use initia_std::oracle;
    use initia_std::bigdecimal::{Self};
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
        timestamp: u64,
        decimals: u64,
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

    #[event]
    struct CreateLiquidityProviderEvent has drop {
        ty: String,
        lp_address: address,
        name: String,
        backup: address,
        slinky_pair: String,
        pair: Object<dex::Config>,
        asset: Object<fungible_asset::Metadata>,
        recipient: address,
    }

    #[event]
    struct ProvideEvent has drop {
        ty: String,
        lp_address: address,
        callback_function: String,
        fid: u64,
        msg: MsgExecuteJSON,
    }

    #[event]
    struct BackupEvent has drop {
        ty: String,
        account: address,
        coin: Object<fungible_asset::Metadata>,
        recipient: address,
        amount: u64,
    }

    #[event]
    struct CallbackEvent has drop {
        ty: String,
        slinky_price: Option<bigdecimal::BigDecimal>,
        pool_price: Option<bigdecimal::BigDecimal>,
        ratio: Option<bigdecimal::BigDecimal>,
        block_timestamp: u64,
        store_timestamp: u64,
        callback_success: bool,
        will_provide_liquidity: bool,
    }

    #[event]
    struct StoreEvent has drop {
        ty: String,
        price_before: u256,
        price_after: u256,
        decimals_before: u64,
        decimals_after: u64,
        timestamp_before: u64,
        timestamp_after: u64,
    }

    #[event]
    struct ProvideLiquidityEvent has drop {
        ty: String,
        recipient: address,
        signer_address: address,
        coin: Object<fungible_asset::Metadata>,
        amount: u64,
        lp_metadata: Object<fungible_asset::Metadata>,
        lp_amount: u64,
    }

    public entry fun create_liquidity_provider(
        account: &signer,
        name: String,                            // Just a name for a visual reference, e.g. "testnet_uinit"
        backup: Option<address>,                 // Priviliged account with a right to withdraw any coins from module object,
                                                 // will be set to `account` if omitted

        slinky_pair: String,                     // Name of a slinky pair, e.g. "INIT/USD"
        pair: Object<dex::Config>,               // Address of liquidity pool, e.g. 0xdbf06c48af3984ec6d9ae8a9aa7dbb0bb1e784aa9b8c4a5681af660cf8558d7d
                                                 // for uinit-usdc on initiation-2 testnet

        asset: Object<fungible_asset::Metadata>, // Address of asset, e.g. 0x8e4733bdabcf7d4afc3d14f0dd46c9bf52fb0fce9e4b996c939e195b8bc891d9
                                                 // for uinit on initiation-2 testnet

        recipient: address,                      // Address which will be receiving all LP tokens
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
                timestamp: 0,
                decimals: 0,
            },
        );
        event::emit(
            CreateLiquidityProviderEvent {
                ty: string::utf8(b"execute_create_liquidity_provider"),
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
        let callback_function = address::to_string(@me);
        string::append(&mut callback_function, string::utf8(b"::drop_lp::callback"));
        cosmos::stargate_with_options(
            &lp_signer,
            json::marshal(&msg),
            cosmos::allow_failure_with_callback(PROVIDE_CB_ID, callback_function),
        );
        event::emit(ProvideEvent {
            ty: string::utf8(b"execute_provide"),
            lp_address: lp_address,
            callback_function: callback_function,
            fid: PROVIDE_CB_ID,
            msg: msg,
        });
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
            ty: string::utf8(b"execute_backup"),
            account: signer::address_of(account),
            coin: coin,
            amount: amount_out,
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
            ty: string::utf8(b"execute_provide_liquidity"),
            signer_address: signer::address_of(lp_signer),
            coin: lp_config.asset,
            amount: amount_in,
            lp_amount: amount_out,
            lp_metadata: metadata_out,
            recipient: lp_config.recipient,
        });
    }

    // Read price from slinky and store it in module storage
    entry fun store(
        lp_address: address,
    ) acquires LpConfig {
        let lp_config = borrow_global_mut<LpConfig>(lp_address);
        let block_timestamp = block::get_current_block_timestamp();
        if (block_timestamp > lp_config.timestamp) {
            let (price, timestamp, decimals) = oracle::get_price(lp_config.slinky_pair);
            event::emit(StoreEvent {
                ty: string::utf8(b"execute_store"),
                price_before: lp_config.price,
                price_after: price,
                timestamp_before: lp_config.timestamp,
                timestamp_after: timestamp,
                decimals_before: lp_config.decimals,
                decimals_after: decimals,
            });
            lp_config.price = price;
            lp_config.timestamp = timestamp;
            lp_config.decimals = decimals;
        } else {
            event::emit(StoreEvent {
                ty: string::utf8(b"execute_store"),
                price_before: lp_config.price,
                price_after: lp_config.price,
                timestamp_before: lp_config.timestamp,
                timestamp_after: lp_config.timestamp,
                decimals_before: lp_config.decimals,
                decimals_after: lp_config.decimals,
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
        let event = CallbackEvent {
            ty: string::utf8(b"execute_callback"),
            callback_success: success,
            block_timestamp: block::get_current_block_timestamp(),
            store_timestamp: lp_config.timestamp,
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
            let block_timestamp = block::get_current_block_timestamp();
            event.slinky_price = option::some(slinky_price);
            event.pool_price = option::some(pool_price);
            event.ratio = option::some(ratio);
            if (
                // slinky price is up to date
                lp_config.timestamp == block_timestamp
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

    #[test]
    fun test_create_liquidity_provider_wrong_slilnky_pair(){}
    #[test]
    fun test_create_liquidity_provider_empty_name(){}
    #[test]
    fun test_create_liquidity_provider_length_exceed(){}
    #[test]
    fun test_create_liquidity_provider(){}

    #[test]
    fun test_provide_uninitialized(){}
    #[test]
    fun test_provide(){}

    #[test]
    fun test_backup_uninitialized(){}
    #[test]
    fun test_backup_unauthorized(){}
    #[test]
    fun test_backup_empty_transfer(){}
    #[test]
    fun test_backup(){}

    #[test]
    fun test_store_uninitialized(){}
    #[test]
    fun test_store_different_block(){}
    #[test]
    fun test_store_same_block(){}

    #[test]
    fun test_provide_liquidity_empty_balance(){}
    #[test]
    fun test_provide_liquidity(){}

    #[test]
    fun test_callback_uninitialized(){}
    #[test]
    fun test_callback_success(){}
    #[test]
    fun test_callback_failure(){}
}
