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

    struct LpConfigView has key, drop {
        extend_ref: address,
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

    // Emits stargate message which:
    // 1. Calls @me::drop_lp::store()
    // 2.1. Then calls @me::drop_lp::callback(..., ..., false) if store() fails
    // 2.2. Then calls @me::drop_lp::callback(..., ..., true) if store() succeeds
    public entry fun provide(
        lp_address: address, // Address of drop_lp instance object
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
            args: vector[json::marshal_to_string<String>(&address::to_string(lp_address))],
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
        lp_address: address,                    // Address of drop_lp instance object
        coin: Object<fungible_asset::Metadata>, // Address of coin object to withdraw
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
        // 1. Provide liquidity
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

        // 2. Sweep all received LP tokens to the configured recipient
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
        account: &signer, // This internal call should originate from module object
        _id: u64,         // Unused since we always use the same ID anyway
        success: bool,    // Did an attempt to read price from slinky succeed?
    ) acquires LpConfig {
        // Security model: if signer has LpConfig, that means this signer is legitimate
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
            // Warning: this algorithm assumes we are working with an X/USD pool
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
                // Slinky price is up to date
                lp_config.timestamp == block_timestamp
                    &&
                    // Pool price is not more than 1% off
                    bigdecimal::le(ratio, bigdecimal::from_ratio_u256(101, 100))
            ) {
                event.will_provide_liquidity = true;
            }
        } else {
            // Slinky doesn't know about INIT yet, skip any validation and go full YOLO
            event.will_provide_liquidity = true;
        };

        if (event.will_provide_liquidity) {
            provide_liquidity(&lp_signer, lp_config);
        };
        event::emit(event);
    }

    #[view]
    public fun lp_config(account: address): LpConfigView acquires LpConfig {
        let config = borrow_global<LpConfig>(account);
        let signer = object::generate_signer_for_extending(&config.extend_ref);
        let addr = signer::address_of(&signer);
        LpConfigView {
            extend_ref: addr,
            name: config.name,
            backup: config.backup,
            slinky_pair: config.slinky_pair,
            pair: config.pair,
            asset: config.asset,
            recipient: config.recipient,
            price: config.price,
            timestamp: config.timestamp,
            decimals: config.decimals,
        }
    }

    #[test_only]
    fun initialize_coin_for_testing(account: &signer, symbol: String): (coin::BurnCapability, coin::FreezeCapability, coin::MintCapability) {
        let (mint_cap, burn_cap, freeze_cap, _) =
            coin::initialize_and_generate_extend_ref(
                account,
                option::none(),
                string::utf8(b""),
                symbol,
                6,
                string::utf8(b""),
                string::utf8(b"")
            );

        return (burn_cap, freeze_cap, mint_cap)
    }

    #[test(chain = @me)] // dex::init_module_for_test uses 0x1 for that (why?)
    #[expected_failure(abort_code = 65538, location = Self)]
    fun test_execute_create_liquidity_provider_wrong_slilnky_pair(chain: &signer) {
        initia_std::primary_fungible_store::init_module_for_test();
        initia_std::dex::init_module_for_test();

        let chain_addr = signer::address_of(chain);
        let (_, _, init_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"INIT"));
        let (_, _, usdc_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"USDC"));
        let init_metadata = coin::metadata(chain_addr, string::utf8(b"INIT"));
        let _ = coin::metadata(chain_addr, string::utf8(b"USDC"));
        coin::mint_to(&init_mint_cap, chain_addr, 100000000);
        coin::mint_to(&usdc_mint_cap, chain_addr, 100000000);
        dex::create_pair_script(
            chain,
            std::string::utf8(b"name"),
            std::string::utf8(b"SYMBOL"),
            bigdecimal::from_ratio_u64(3, 1000),
            bigdecimal::from_ratio_u64(8, 10),
            bigdecimal::from_ratio_u64(2, 10),
            coin::metadata(chain_addr, string::utf8(b"INIT")),
            coin::metadata(chain_addr, string::utf8(b"USDC")),
            80000000,
            20000000
        );

        let pair_metadata_address = coin::metadata_address(signer::address_of(chain), string::utf8(b"SYMBOL")); 
        let config_object = object::address_to_object<dex::Config>(pair_metadata_address);
        create_liquidity_provider(
            chain,
            string::utf8(b"name"),
            option::none(),
            string::utf8(b""),
            config_object,
            init_metadata,
            chain_addr
        );
    }
    
    #[test(chain = @me)]
    #[expected_failure(abort_code = 65537, location = Self)]
    fun test_execute_create_liquidity_provider_empty_name(chain: &signer) {
        initia_std::primary_fungible_store::init_module_for_test();
        initia_std::dex::init_module_for_test();

        let chain_addr = signer::address_of(chain);
        let (_, _, init_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"INIT"));
        let (_, _, usdc_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"USDC"));
        let init_metadata = coin::metadata(chain_addr, string::utf8(b"INIT"));
        let _ = coin::metadata(chain_addr, string::utf8(b"USDC"));
        coin::mint_to(&init_mint_cap, chain_addr, 100000000);
        coin::mint_to(&usdc_mint_cap, chain_addr, 100000000);
        dex::create_pair_script(
            chain,
            std::string::utf8(b"name"),
            std::string::utf8(b"SYMBOL"),
            bigdecimal::from_ratio_u64(3, 1000),
            bigdecimal::from_ratio_u64(8, 10),
            bigdecimal::from_ratio_u64(2, 10),
            coin::metadata(chain_addr, string::utf8(b"INIT")),
            coin::metadata(chain_addr, string::utf8(b"USDC")),
            80000000,
            20000000
        );

        let pair_metadata_address = coin::metadata_address(signer::address_of(chain), string::utf8(b"SYMBOL")); 
        let config_object = object::address_to_object<dex::Config>(pair_metadata_address);
        create_liquidity_provider(
            chain,
            string::utf8(b""),
            option::none(),
            string::utf8(b"slinky_pair"),
            config_object,
            init_metadata,
            chain_addr
        );
    }
    
    #[test(chain = @me)]
    #[expected_failure(abort_code = 131075, location = Self)]
    fun test_execute_create_liquidity_provider_length_exceed(chain: &signer) {
        initia_std::primary_fungible_store::init_module_for_test();
        initia_std::dex::init_module_for_test();

        let chain_addr = signer::address_of(chain);
        let (_, _, init_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"INIT"));
        let (_, _, usdc_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"USDC"));
        let init_metadata = coin::metadata(chain_addr, string::utf8(b"INIT"));
        let _ = coin::metadata(chain_addr, string::utf8(b"USDC"));
        coin::mint_to(&init_mint_cap, chain_addr, 100000000);
        coin::mint_to(&usdc_mint_cap, chain_addr, 100000000);
        dex::create_pair_script(
            chain,
            std::string::utf8(b"name"),
            std::string::utf8(b"SYMBOL"),
            bigdecimal::from_ratio_u64(3, 1000),
            bigdecimal::from_ratio_u64(8, 10),
            bigdecimal::from_ratio_u64(2, 10),
            coin::metadata(chain_addr, string::utf8(b"INIT")),
            coin::metadata(chain_addr, string::utf8(b"USDC")),
            80000000,
            20000000
        );

        let pair_metadata_address = coin::metadata_address(signer::address_of(chain), string::utf8(b"SYMBOL")); 
        let config_object = object::address_to_object<dex::Config>(pair_metadata_address);
        create_liquidity_provider(
            chain,
            string::utf8(vector::map<u64, u8>(vector::range(0, 65), |e| e as u8)),
            option::none(),
            string::utf8(b"slinky_pair"),
            config_object,
            init_metadata,
            chain_addr
        );
    }
    
    #[test(chain = @me)]
    fun test_execute_create_liquidity_provider(chain: &signer) acquires LpConfig {
        initia_std::primary_fungible_store::init_module_for_test();
        initia_std::dex::init_module_for_test();

        let chain_addr = signer::address_of(chain);
        let (_, _, init_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"INIT"));
        let (_, _, usdc_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"USDC"));
        let init_metadata = coin::metadata(chain_addr, string::utf8(b"INIT"));
        let _ = coin::metadata(chain_addr, string::utf8(b"USDC"));
        coin::mint_to(&init_mint_cap, chain_addr, 100000000);
        coin::mint_to(&usdc_mint_cap, chain_addr, 100000000);
        dex::create_pair_script(
            chain,
            std::string::utf8(b"name"),
            std::string::utf8(b"SYMBOL"),
            bigdecimal::from_ratio_u64(3, 1000),
            bigdecimal::from_ratio_u64(8, 10),
            bigdecimal::from_ratio_u64(2, 10),
            coin::metadata(chain_addr, string::utf8(b"INIT")),
            coin::metadata(chain_addr, string::utf8(b"USDC")),
            80000000,
            20000000
        );

        let pair_metadata_address = coin::metadata_address(signer::address_of(chain), string::utf8(b"SYMBOL")); 
        let config_object = object::address_to_object<dex::Config>(pair_metadata_address);
        create_liquidity_provider(
            chain,
            string::utf8(b"name"),
            option::none(),
            string::utf8(b"slinky_pair"),
            config_object,
            init_metadata,
            chain_addr
        );

        let seed = b"drop_lp_name";
        let lp_object_address = object::create_object_address(&chain_addr, seed);
        let lp_config = borrow_global<LpConfig>(lp_object_address);
        // It's impossible to verify extend_ref here because we're allowed to create object
        // only once, so we can't create object with the same address in these tests,
        // let's just pray it's valid
        assert!(lp_config.name == string::utf8(b"name"));
        assert!(lp_config.backup == chain_addr);
        assert!(lp_config.slinky_pair == string::utf8(b"slinky_pair"));
        assert!(lp_config.pair == config_object);
        assert!(lp_config.asset == init_metadata);
        assert!(lp_config.recipient == chain_addr);
        assert!(lp_config.price == 0u256);
        assert!(lp_config.timestamp == 0u64);
        assert!(lp_config.decimals == 0u64);
    }

    #[test(chain = @me)]
    #[expected_failure]
    fun test_execute_provide_uninitialized(chain: &signer) acquires LpConfig {
        initia_std::primary_fungible_store::init_module_for_test();
        initia_std::dex::init_module_for_test();

        let chain_addr = signer::address_of(chain);
        let (_, _, init_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"INIT"));
        let (_, _, usdc_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"USDC"));
        let init_metadata = coin::metadata(chain_addr, string::utf8(b"INIT"));
        let _ = coin::metadata(chain_addr, string::utf8(b"USDC"));
        coin::mint_to(&init_mint_cap, chain_addr, 100000000);
        coin::mint_to(&usdc_mint_cap, chain_addr, 100000000);
        dex::create_pair_script(
            chain,
            std::string::utf8(b"name"),
            std::string::utf8(b"SYMBOL"),
            bigdecimal::from_ratio_u64(3, 1000),
            bigdecimal::from_ratio_u64(8, 10),
            bigdecimal::from_ratio_u64(2, 10),
            coin::metadata(chain_addr, string::utf8(b"INIT")),
            coin::metadata(chain_addr, string::utf8(b"USDC")),
            80000000,
            20000000
        );

        let pair_metadata_address = coin::metadata_address(signer::address_of(chain), string::utf8(b"SYMBOL")); 
        let config_object = object::address_to_object<dex::Config>(pair_metadata_address);
        create_liquidity_provider(
            chain,
            string::utf8(b"name"),
            option::none(),
            string::utf8(b"slinky_pair"),
            config_object,
            init_metadata,
            chain_addr
        );
        provide(chain_addr); // just a random address
    }

    #[test(chain = @me)]
    fun test_execute_provide(chain: &signer) acquires LpConfig {
        initia_std::primary_fungible_store::init_module_for_test();
        initia_std::dex::init_module_for_test();

        let chain_addr = signer::address_of(chain);
        let (_, _, init_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"INIT"));
        let (_, _, usdc_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"USDC"));
        let init_metadata = coin::metadata(chain_addr, string::utf8(b"INIT"));
        let _ = coin::metadata(chain_addr, string::utf8(b"USDC"));
        coin::mint_to(&init_mint_cap, chain_addr, 100000000);
        coin::mint_to(&usdc_mint_cap, chain_addr, 100000000);
        dex::create_pair_script(
            chain,
            std::string::utf8(b"name"),
            std::string::utf8(b"SYMBOL"),
            bigdecimal::from_ratio_u64(3, 1000),
            bigdecimal::from_ratio_u64(8, 10),
            bigdecimal::from_ratio_u64(2, 10),
            coin::metadata(chain_addr, string::utf8(b"INIT")),
            coin::metadata(chain_addr, string::utf8(b"USDC")),
            80000000,
            20000000
        );

        let pair_metadata_address = coin::metadata_address(signer::address_of(chain), string::utf8(b"SYMBOL")); 
        let config_object = object::address_to_object<dex::Config>(pair_metadata_address);
        create_liquidity_provider(
            chain,
            string::utf8(b"name"),
            option::none(),
            string::utf8(b"slinky_pair"),
            config_object,
            init_metadata,
            chain_addr
        );

        let seed = b"drop_lp_name";
        let lp_object_address = object::create_object_address(&chain_addr, seed);
        provide(lp_object_address);
    }

    #[test(chain = @me)]
    #[expected_failure(arithmetic_error, location = 0x1::dex)]
    fun test_execute_provide_liquidity_empty_balance(chain: &signer) acquires LpConfig {
        initia_std::primary_fungible_store::init_module_for_test();
        initia_std::dex::init_module_for_test();

        let chain_addr = signer::address_of(chain);
        let (_, _, init_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"INIT"));
        let (_, _, usdc_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"USDC"));
        let init_metadata = coin::metadata(chain_addr, string::utf8(b"INIT"));
        let _ = coin::metadata(chain_addr, string::utf8(b"USDC"));
        coin::mint_to(&init_mint_cap, chain_addr, 100000000);
        coin::mint_to(&usdc_mint_cap, chain_addr, 100000000);
        dex::create_pair_script(
            chain,
            std::string::utf8(b"name"),
            std::string::utf8(b"SYMBOL"),
            bigdecimal::from_ratio_u64(3, 1000),
            bigdecimal::from_ratio_u64(8, 10),
            bigdecimal::from_ratio_u64(2, 10),
            coin::metadata(chain_addr, string::utf8(b"INIT")),
            coin::metadata(chain_addr, string::utf8(b"USDC")),
            80000000,
            20000000
        );

        let pair_metadata_address = coin::metadata_address(signer::address_of(chain), string::utf8(b"SYMBOL")); 
        let config_object = object::address_to_object<dex::Config>(pair_metadata_address);
        create_liquidity_provider(
            chain,
            string::utf8(b"name"),
            option::none(),
            string::utf8(b"slinky_pair"),
            config_object,
            init_metadata,
            chain_addr
        );

        let seed = b"drop_lp_name";
        let lp_object_address = object::create_object_address(&chain_addr, seed);
        let lp_object_config = borrow_global<LpConfig>(lp_object_address);
        let lp_signer = object::generate_signer_for_extending(&lp_object_config.extend_ref);
        provide_liquidity(&lp_signer, lp_object_config);
    }
    
    #[test(chain = @me)]
    fun test_execute_provide_liquidity(chain: &signer) acquires LpConfig {
        initia_std::primary_fungible_store::init_module_for_test();
        initia_std::dex::init_module_for_test();

        let chain_addr = signer::address_of(chain);
        let (_, _, init_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"INIT"));
        let (_, _, usdc_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"USDC"));
        let init_metadata = coin::metadata(chain_addr, string::utf8(b"INIT"));
        let _ = coin::metadata(chain_addr, string::utf8(b"USDC"));
        coin::mint_to(&init_mint_cap, chain_addr, 100000000);
        coin::mint_to(&usdc_mint_cap, chain_addr, 100000000);
        dex::create_pair_script(
            chain,
            std::string::utf8(b"name"),
            std::string::utf8(b"SYMBOL"),
            bigdecimal::from_ratio_u64(3, 1000),
            bigdecimal::from_ratio_u64(8, 10),
            bigdecimal::from_ratio_u64(2, 10),
            coin::metadata(chain_addr, string::utf8(b"INIT")),
            coin::metadata(chain_addr, string::utf8(b"USDC")),
            80000000,
            20000000
        );

        let pair_metadata_address = coin::metadata_address(signer::address_of(chain), string::utf8(b"SYMBOL")); 
        let config_object = object::address_to_object<dex::Config>(pair_metadata_address);
        create_liquidity_provider(
            chain,
            string::utf8(b"name"),
            option::none(),
            string::utf8(b"slinky_pair"),
            config_object,
            init_metadata,
            chain_addr
        );

        let seed = b"drop_lp_name";
        let lp_object_address = object::create_object_address(&chain_addr, seed);
        let lp_object_config = borrow_global<LpConfig>(lp_object_address);
        let lp_signer = object::generate_signer_for_extending(&lp_object_config.extend_ref);
        coin::mint_to(&init_mint_cap, signer::address_of(&lp_signer), 100000);
        provide_liquidity(&lp_signer, lp_object_config);
    }

    #[test(chain = @me)]
    #[expected_failure]
    fun test_execute_backup_uninitialized(chain: &signer) acquires LpConfig {
        initia_std::primary_fungible_store::init_module_for_test();
        initia_std::dex::init_module_for_test();

        let chain_addr = signer::address_of(chain);
        let (_, _, init_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"INIT"));
        let (_, _, usdc_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"USDC"));
        let init_metadata = coin::metadata(chain_addr, string::utf8(b"INIT"));
        let _ = coin::metadata(chain_addr, string::utf8(b"USDC"));
        coin::mint_to(&init_mint_cap, chain_addr, 100000000);
        coin::mint_to(&usdc_mint_cap, chain_addr, 100000000);
        dex::create_pair_script(
            chain,
            std::string::utf8(b"name"),
            std::string::utf8(b"SYMBOL"),
            bigdecimal::from_ratio_u64(3, 1000),
            bigdecimal::from_ratio_u64(8, 10),
            bigdecimal::from_ratio_u64(2, 10),
            coin::metadata(chain_addr, string::utf8(b"INIT")),
            coin::metadata(chain_addr, string::utf8(b"USDC")),
            80000000,
            20000000
        );

        let pair_metadata_address = coin::metadata_address(signer::address_of(chain), string::utf8(b"SYMBOL")); 
        let config_object = object::address_to_object<dex::Config>(pair_metadata_address);
        create_liquidity_provider(
            chain,
            string::utf8(b"name"),
            option::none(),
            string::utf8(b"slinky_pair"),
            config_object,
            init_metadata,
            chain_addr
        );

        backup(chain, @me, init_metadata); // just a random address
    }

    #[test(chain = @me, random_sender = @0x123)]
    #[expected_failure(abort_code = 327684, location = Self)]
    fun test_execute_backup_unauthorized(chain: &signer, random_sender: &signer) acquires LpConfig {
        initia_std::primary_fungible_store::init_module_for_test();
        initia_std::dex::init_module_for_test();

        let chain_addr = signer::address_of(chain);
        let (_, _, init_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"INIT"));
        let (_, _, usdc_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"USDC"));
        let init_metadata = coin::metadata(chain_addr, string::utf8(b"INIT"));
        let _ = coin::metadata(chain_addr, string::utf8(b"USDC"));
        coin::mint_to(&init_mint_cap, chain_addr, 100000000);
        coin::mint_to(&usdc_mint_cap, chain_addr, 100000000);
        dex::create_pair_script(
            chain,
            std::string::utf8(b"name"),
            std::string::utf8(b"SYMBOL"),
            bigdecimal::from_ratio_u64(3, 1000),
            bigdecimal::from_ratio_u64(8, 10),
            bigdecimal::from_ratio_u64(2, 10),
            coin::metadata(chain_addr, string::utf8(b"INIT")),
            coin::metadata(chain_addr, string::utf8(b"USDC")),
            80000000,
            20000000
        );

        let pair_metadata_address = coin::metadata_address(signer::address_of(chain), string::utf8(b"SYMBOL")); 
        let config_object = object::address_to_object<dex::Config>(pair_metadata_address);
        create_liquidity_provider(
            chain,
            string::utf8(b"name"),
            option::none(),
            string::utf8(b"slinky_pair"),
            config_object,
            init_metadata,
            chain_addr
        );

        let seed = b"drop_lp_name";
        let lp_object_address = object::create_object_address(&chain_addr, seed);
        backup(random_sender, lp_object_address, init_metadata);
    }

    #[test(chain = @me)]
    fun test_execute_backup(chain: &signer) acquires LpConfig {
        initia_std::primary_fungible_store::init_module_for_test();
        initia_std::dex::init_module_for_test();

        let chain_addr = signer::address_of(chain);
        let (_, _, init_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"INIT"));
        let (_, _, usdc_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"USDC"));
        let init_metadata = coin::metadata(chain_addr, string::utf8(b"INIT"));
        let _ = coin::metadata(chain_addr, string::utf8(b"USDC"));
        coin::mint_to(&init_mint_cap, chain_addr, 100000000);
        coin::mint_to(&usdc_mint_cap, chain_addr, 100000000);
        dex::create_pair_script(
            chain,
            std::string::utf8(b"name"),
            std::string::utf8(b"SYMBOL"),
            bigdecimal::from_ratio_u64(3, 1000),
            bigdecimal::from_ratio_u64(8, 10),
            bigdecimal::from_ratio_u64(2, 10),
            coin::metadata(chain_addr, string::utf8(b"INIT")),
            coin::metadata(chain_addr, string::utf8(b"USDC")),
            80000000,
            20000000
        );

        let pair_metadata_address = coin::metadata_address(signer::address_of(chain), string::utf8(b"SYMBOL")); 
        let config_object = object::address_to_object<dex::Config>(pair_metadata_address);
        create_liquidity_provider(
            chain,
            string::utf8(b"name"),
            option::none(),
            string::utf8(b"slinky_pair"),
            config_object,
            init_metadata,
            chain_addr
        );

        let seed = b"drop_lp_name";
        let lp_object_address = object::create_object_address(&chain_addr, seed);
        backup(chain, lp_object_address, init_metadata);
    }

    #[test(chain = @me)]
    #[expected_failure]
    fun test_execute_store_uninitialized(chain: &signer) acquires LpConfig {
        initia_std::primary_fungible_store::init_module_for_test();
        initia_std::dex::init_module_for_test();

        let chain_addr = signer::address_of(chain);
        let (_, _, init_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"INIT"));
        let (_, _, usdc_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"USDC"));
        let init_metadata = coin::metadata(chain_addr, string::utf8(b"INIT"));
        let _ = coin::metadata(chain_addr, string::utf8(b"USDC"));
        coin::mint_to(&init_mint_cap, chain_addr, 100000000);
        coin::mint_to(&usdc_mint_cap, chain_addr, 100000000);
        dex::create_pair_script(
            chain,
            std::string::utf8(b"name"),
            std::string::utf8(b"SYMBOL"),
            bigdecimal::from_ratio_u64(3, 1000),
            bigdecimal::from_ratio_u64(8, 10),
            bigdecimal::from_ratio_u64(2, 10),
            coin::metadata(chain_addr, string::utf8(b"INIT")),
            coin::metadata(chain_addr, string::utf8(b"USDC")),
            80000000,
            20000000
        );

        let pair_metadata_address = coin::metadata_address(signer::address_of(chain), string::utf8(b"SYMBOL")); 
        let config_object = object::address_to_object<dex::Config>(pair_metadata_address);
        create_liquidity_provider(
            chain,
            string::utf8(b"name"),
            option::none(),
            string::utf8(b"slinky_pair"),
            config_object,
            init_metadata,
            chain_addr
        );

        store(@0x123); // just a random address
    }

    #[test(chain = @me)]
    fun test_execute_store_different_block(chain: &signer) acquires LpConfig {
        initia_std::primary_fungible_store::init_module_for_test();
        initia_std::dex::init_module_for_test();

        let chain_addr = signer::address_of(chain);
        let (_, _, init_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"INIT"));
        let (_, _, usdc_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"USDC"));
        let init_metadata = coin::metadata(chain_addr, string::utf8(b"INIT"));
        let _ = coin::metadata(chain_addr, string::utf8(b"USDC"));
        coin::mint_to(&init_mint_cap, chain_addr, 100000000);
        coin::mint_to(&usdc_mint_cap, chain_addr, 100000000);
        dex::create_pair_script(
            chain,
            std::string::utf8(b"name"),
            std::string::utf8(b"SYMBOL"),
            bigdecimal::from_ratio_u64(3, 1000),
            bigdecimal::from_ratio_u64(8, 10),
            bigdecimal::from_ratio_u64(2, 10),
            coin::metadata(chain_addr, string::utf8(b"INIT")),
            coin::metadata(chain_addr, string::utf8(b"USDC")),
            80000000,
            20000000
        );

        let pair_metadata_address = coin::metadata_address(signer::address_of(chain), string::utf8(b"SYMBOL")); 
        let config_object = object::address_to_object<dex::Config>(pair_metadata_address);
        create_liquidity_provider(
            chain,
            string::utf8(b"name"),
            option::none(),
            string::utf8(b"slinky_pair"),
            config_object,
            init_metadata,
            chain_addr
        );

        let seed = b"drop_lp_name";
        let lp_object_address = object::create_object_address(&chain_addr, seed);
        let config_before = borrow_global<LpConfig>(lp_object_address);
        block::set_block_info(123u64, 123u64);
        oracle::set_price(&string::utf8(b"slinky_pair"), 1u256, 2u64, 3u64);

        let config_before_extend_ref = signer::address_of(&object::generate_signer_for_extending(&config_before.extend_ref));
        let config_before_name = config_before.name;
        let config_before_backup = config_before.backup;
        let config_before_slinky_pair = config_before.slinky_pair;
        let config_before_pair = config_before.pair;
        let config_before_asset = config_before.asset;
        let config_before_recipient = config_before.recipient;
        let config_before_price = config_before.price;
        let config_before_timestamp = config_before.timestamp;
        let config_before_decimals = config_before.decimals;
        store(lp_object_address);

        let config_after = borrow_global<LpConfig>(lp_object_address);
        assert!(config_before_extend_ref == signer::address_of(&object::generate_signer_for_extending(&config_after.extend_ref)));
        assert!(config_before_name == config_after.name);
        assert!(config_before_backup == config_after.backup);
        assert!(config_before_slinky_pair == config_after.slinky_pair);
        assert!(config_before_pair == config_after.pair);
        assert!(config_before_asset == config_after.asset);
        assert!(config_before_recipient == config_after.recipient);

        assert!(config_before_price != config_after.price);
        assert!(config_before_timestamp != config_after.timestamp);
        assert!(config_before_decimals != config_after.decimals);

        assert!(config_after.price == 1u256);
        assert!(config_after.timestamp == 2u64);
        assert!(config_after.decimals == 3u64);
    }

    #[test(chain = @me)]
    fun test_execute_store_same_block(chain: &signer) acquires LpConfig {
        initia_std::primary_fungible_store::init_module_for_test();
        initia_std::dex::init_module_for_test();

        let chain_addr = signer::address_of(chain);
        let (_, _, init_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"INIT"));
        let (_, _, usdc_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"USDC"));
        let init_metadata = coin::metadata(chain_addr, string::utf8(b"INIT"));
        let _ = coin::metadata(chain_addr, string::utf8(b"USDC"));
        coin::mint_to(&init_mint_cap, chain_addr, 100000000);
        coin::mint_to(&usdc_mint_cap, chain_addr, 100000000);
        dex::create_pair_script(
            chain,
            std::string::utf8(b"name"),
            std::string::utf8(b"SYMBOL"),
            bigdecimal::from_ratio_u64(3, 1000),
            bigdecimal::from_ratio_u64(8, 10),
            bigdecimal::from_ratio_u64(2, 10),
            coin::metadata(chain_addr, string::utf8(b"INIT")),
            coin::metadata(chain_addr, string::utf8(b"USDC")),
            80000000,
            20000000
        );

        let pair_metadata_address = coin::metadata_address(signer::address_of(chain), string::utf8(b"SYMBOL")); 
        let config_object = object::address_to_object<dex::Config>(pair_metadata_address);
        create_liquidity_provider(
            chain,
            string::utf8(b"name"),
            option::none(),
            string::utf8(b"slinky_pair"),
            config_object,
            init_metadata,
            chain_addr
        );

        let seed = b"drop_lp_name";
        let lp_object_address = object::create_object_address(&chain_addr, seed);
        let config_before = borrow_global<LpConfig>(lp_object_address);
        let config_before_extend_ref = signer::address_of(&object::generate_signer_for_extending(&config_before.extend_ref));
        let config_before_name = config_before.name;
        let config_before_backup = config_before.backup;
        let config_before_slinky_pair = config_before.slinky_pair;
        let config_before_pair = config_before.pair;
        let config_before_asset = config_before.asset;
        let config_before_recipient = config_before.recipient;
        let config_before_price = config_before.price;
        let config_before_timestamp = config_before.timestamp;
        let config_before_decimals = config_before.decimals;
        store(lp_object_address);

        let config_after = borrow_global<LpConfig>(lp_object_address);
        assert!(config_before_extend_ref == signer::address_of(&object::generate_signer_for_extending(&config_after.extend_ref)));
        assert!(config_before_name == config_after.name);
        assert!(config_before_backup == config_after.backup);
        assert!(config_before_slinky_pair == config_after.slinky_pair);
        assert!(config_before_pair == config_after.pair);
        assert!(config_before_asset == config_after.asset);
        assert!(config_before_recipient == config_after.recipient);
        assert!(config_before_price == config_after.price);
        assert!(config_before_timestamp == config_after.timestamp);
        assert!(config_before_decimals == config_after.decimals);
    }

    #[test]
    fun test_execute_callback_uninitialized(){}
    #[test]
    fun test_execute_callback_success(){}
    #[test]
    fun test_execute_callback_failure(){}

    #[test(chain = @me)]
    fun test_query_lp_config(chain: &signer) acquires LpConfig {
        initia_std::primary_fungible_store::init_module_for_test();
        initia_std::dex::init_module_for_test();

        let chain_addr = signer::address_of(chain);
        let (_, _, init_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"INIT"));
        let (_, _, usdc_mint_cap) =
            initialize_coin_for_testing(chain, string::utf8(b"USDC"));
        let init_metadata = coin::metadata(chain_addr, string::utf8(b"INIT"));
        let _ = coin::metadata(chain_addr, string::utf8(b"USDC"));
        coin::mint_to(&init_mint_cap, chain_addr, 100000000);
        coin::mint_to(&usdc_mint_cap, chain_addr, 100000000);
        dex::create_pair_script(
            chain,
            std::string::utf8(b"name"),
            std::string::utf8(b"SYMBOL"),
            bigdecimal::from_ratio_u64(3, 1000),
            bigdecimal::from_ratio_u64(8, 10),
            bigdecimal::from_ratio_u64(2, 10),
            coin::metadata(chain_addr, string::utf8(b"INIT")),
            coin::metadata(chain_addr, string::utf8(b"USDC")),
            80000000,
            20000000
        );

        let pair_metadata_address = coin::metadata_address(signer::address_of(chain), string::utf8(b"SYMBOL")); 
        let config_object = object::address_to_object<dex::Config>(pair_metadata_address);
        create_liquidity_provider(
            chain,
            string::utf8(b"name"),
            option::none(),
            string::utf8(b"slinky_pair"),
            config_object,
            init_metadata,
            chain_addr
        );

        let seed = b"drop_lp_name";
        let lp_address = object::create_object_address(&chain_addr, seed);
        let lp_config = lp_config(lp_address);
        assert!(lp_config.name == string::utf8(b"name"));
        assert!(lp_config.backup == chain_addr);
        assert!(lp_config.slinky_pair == string::utf8(b"slinky_pair"));
        assert!(lp_config.pair == config_object);
        assert!(lp_config.asset == init_metadata);
        assert!(lp_config.recipient == chain_addr);
        assert!(lp_config.price == 0u256);
        assert!(lp_config.timestamp == 0u64);
        assert!(lp_config.decimals == 0u64);
    }
}
