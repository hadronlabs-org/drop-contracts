module me::liquidity_provider {
    use initia_std::dex;
    use initia_std::object::{Self, Object, ExtendRef};
    use initia_std::coin;
    use initia_std::json;
    use initia_std::cosmos;
    use initia_std::oracle;
    use initia_std::bigdecimal;
    use initia_std::math128;
    use initia_std::block;
    use initia_std::fungible_asset::Metadata;

    use std::option;
    use std::signer;
    use std::address;
    use std::string::{Self, String};
    use std::error;

    const ENOT_OWNER: u64 = 1;

    struct ModuleStore has key {
        extend_ref: ExtendRef,
        price: u256,
        ts: u64,
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

    fun init_module(creator: &signer) {
        let constructor_ref = object::create_object(@me, false);
        let extend_ref = object::generate_extend_ref(&constructor_ref);
        move_to(creator, ModuleStore { extend_ref, price: 0, ts: 0, decimals: 0, });
    }

    // emits stargate message which:
    // 1. calls @me::liquidity_provider::store()
    // 2.1. then calls @me::liquidity_provider(1, false) if store() fails
    // 2.2. then calls @me::liquidity_provider(1, true) if store() succeeds
    public entry fun provide() acquires ModuleStore {
        let store = borrow_global<ModuleStore>(@me);
        let signer = object::generate_signer_for_extending(&store.extend_ref);
        let addr = signer::address_of(&signer);

        let msg = MsgExecuteJSON {
            _type_: string::utf8(b"/initia.move.v1.MsgExecuteJSON"),
            sender: address::to_sdk(addr),
            module_address: address::to_sdk(@me),
            module_name: string::utf8(b"liquidity_provider"),
            function_name: string::utf8(b"store"),
            type_args: vector[],
            args: vector[],
        };

        let fid = address::to_string(@me);
        string::append(&mut fid, string::utf8(b"::liquidity_provider::callback"));

        cosmos::stargate_with_options(
            &signer,
            json::marshal(&msg),
            cosmos::allow_failure_with_callback(1, fid),
        );
    }

    // Last resort function only available for module admin to withdraw all funds of `coin` denomination
    // from module's object address in case if there is an unrecoverable bug somewhere
    public entry fun backup(account: &signer, coin: Object<Metadata>) acquires ModuleStore {
        assert!(
            signer::address_of(account) == @me,
            error::permission_denied(ENOT_OWNER),
        );

        let store = borrow_global<ModuleStore>(@me);
        let ref_signer = object::generate_signer_for_extending(&store.extend_ref);
        let balance = coin::balance(signer::address_of(&ref_signer), coin);
        coin::transfer(&ref_signer, signer::address_of(account), coin, balance);
    }

    // 1. provide liquidity
    // 2. sweep all received LP tokens to @recipient
    fun provide_liquidity(account: &signer) {
        let addr = signer::address_of(account);

        let metadata_in = object::address_to_object(@asset);
        let amount_in = coin::balance(addr, metadata_in);
        dex::single_asset_provide_liquidity_script(
            account,
            object::address_to_object(@pair),
            metadata_in,
            amount_in,
            option::none(),
        );

        let metadata_out = object::address_to_object(@pair);
        let amount_out = coin::balance(addr, metadata_out);
        coin::transfer(account, @recipient, metadata_out, amount_out);
    }

    // Read INIT price from slinky
    entry fun store() acquires ModuleStore {
        let store = borrow_global_mut<ModuleStore>(@me);
        let (price, ts, decimals) = oracle::get_price(string::utf8(b"INIT/USD"));
        if (ts != store.ts) {
            store.price = price;
            store.ts = ts;
            store.decimals = decimals;
        }
    }

    // MEV protection, only provide liquidity if pool price is up to date with off-chain price feed
    entry fun callback(_id: u64, success: bool) acquires ModuleStore {
        let store = borrow_global<ModuleStore>(@me);
        let signer = object::generate_signer_for_extending(&store.extend_ref);

        if (success) {
            let slinky_price = bigdecimal::from_ratio_u256(
                store.price,
                (math128::pow(10, (store.decimals as u128)) as u256),
            );
            let pool_price = dex::get_spot_price(
                object::address_to_object(@pair),
                object::address_to_object(@asset),
            );
            let ratio = if (bigdecimal::gt(slinky_price, pool_price)) {
                bigdecimal::div(slinky_price, pool_price)
            } else {
                bigdecimal::div(pool_price, slinky_price)
            };
            let block_ts = block::get_current_block_timestamp();
            if (
                // slinky price is up to date
                store.ts == block_ts
                    &&
                    // pool price is not more than 1% off
                    bigdecimal::le(ratio, bigdecimal::from_ratio_u256(101, 1))
            ) {
                provide_liquidity(&signer);
            }
        } else {
            // slinky doesn't know about INIT yet, skip any validation and go full YOLO
            provide_liquidity(&signer);
        }
    }

    #[test(me = @me)]
    fun test_init_module(me: &signer) acquires ModuleStore {
        assert!(exists<ModuleStore>(@me) == false);
        init_module(me);

        assert!(exists<ModuleStore>(@me) == true);

        let store = borrow_global<ModuleStore>(@me);
        assert!(store.price == 0);
        assert!(store.ts == 0);
        assert!(store.decimals == 0);
    }

    #[test]
    #[expected_failure(abort_code = 327681, location = me::liquidity_provider)]
    fun test_backup_unauthorized() acquires ModuleStore {
        let account = @0x0;
        let addr = signer::address_of(&account);
        let coin_name = string::utf8(b"coin_name");
        let coin_symbol = string::utf8(b"coin_symbol");
        coin::initialize(
            &account,
            std::option::some(123),
            coin_name,
            coin_symbol,
            6u8,
            string::utf8(b"icon_uri"),
            string::utf8(b"project_uri")
        );

        backup(&account, coin::metadata(addr, coin_symbol));
    }

    #[test(me = @me)]
    fun test_backup(me: &signer) acquires ModuleStore {
        init_module(me);

        let addr_me = signer::address_of(me);
        let coin_name = string::utf8(b"coin_name");
        let coin_symbol = string::utf8(b"coin_symbol");
        let (mint_capability, _, _) = coin::initialize(
            me,
            std::option::some(123),
            coin_name,
            coin_symbol,
            6u8,
            string::utf8(b"decimals"),
            string::utf8(b"icon_uri")
        );
        let metadata = coin::metadata(addr_me, coin_symbol);
        let store = borrow_global<ModuleStore>(@me);
        let module_addr = object::address_from_extend_ref(&store.extend_ref);
        coin::mint_to(&mint_capability, module_addr, 123u64);

        assert!(coin::balance(module_addr, metadata) == 123);
        assert!(coin::balance(@me, metadata) == 0);
        backup(me, metadata);
        assert!(coin::balance(@me, metadata) == 123);
        assert!(coin::balance(module_addr, metadata) == 0);
    }
}
