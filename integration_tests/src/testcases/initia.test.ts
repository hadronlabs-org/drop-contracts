import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import {
  DropCore,
  DropFactory,
  DropPump,
  DropPuppeteerInitia,
  DropStrategy,
  DropWithdrawalManager,
  DropWithdrawalVoucher,
  DropRewardsManager,
  DropSplitter,
  DropToken,
  DropLsmShareBondProvider,
  DropNativeBondProvider,
} from 'drop-ts-client';
import {
  QueryClient,
  StakingExtension,
  BankExtension,
  setupStakingExtension,
  setupBankExtension,
  SigningStargateClient,
} from '@cosmjs/stargate';
import { join } from 'path';
import { Tendermint34Client } from '@cosmjs/tendermint-rpc';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import { UnbondBatch } from 'drop-ts-client/lib/contractLib/dropCore';
import { sleep } from '../helpers/sleep';
import { waitForPuppeteerICQ } from '../helpers/waitForPuppeteerICQ';
import { instrumentCoreClass } from '../helpers/knot';
import { checkExchangeRate } from '../helpers/exchangeRate';
import { AccAddress } from '@initia/initia.js';

const DropTokenClass = DropToken.Client;
const DropFactoryClass = DropFactory.Client;
const DropCoreClass = DropCore.Client;
const DropPumpClass = DropPump.Client;
const DropPuppeteerClass = DropPuppeteerInitia.Client;
const DropStrategyClass = DropStrategy.Client;
const DropWithdrawalVoucherClass = DropWithdrawalVoucher.Client;
const DropWithdrawalManagerClass = DropWithdrawalManager.Client;
const DropRewardsManagerClass = DropRewardsManager.Client;
const DropRewardsPumpClass = DropPump.Client;
const DropSplitterClass = DropSplitter.Client;
const DropLsmShareBondProviderClass = DropLsmShareBondProvider.Client;
const DropNativeBondProviderClass = DropNativeBondProvider.Client;

const UNBONDING_TIME = 360;

describe('Core', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    initiaWallet?: DirectSecp256k1HdWallet;
    initiaWallet2?: DirectSecp256k1HdWallet;
    factoryContractClient?: InstanceType<typeof DropFactoryClass>;
    coreContractClient?: InstanceType<typeof DropCoreClass>;
    strategyContractClient?: InstanceType<typeof DropStrategyClass>;
    pumpContractClient?: InstanceType<typeof DropPumpClass>;
    puppeteerContractClient?: InstanceType<typeof DropPuppeteerClass>;
    splitterContractClient?: InstanceType<typeof DropSplitterClass>;
    tokenContractClient?: InstanceType<typeof DropTokenClass>;
    withdrawalVoucherContractClient?: InstanceType<
      typeof DropWithdrawalVoucherClass
    >;
    withdrawalManagerContractClient?: InstanceType<
      typeof DropWithdrawalManagerClass
    >;
    rewardsManagerContractClient?: InstanceType<typeof DropRewardsManagerClass>;
    rewardsPumpContractClient?: InstanceType<typeof DropRewardsPumpClass>;
    lsmShareBondProviderContractClient?: InstanceType<
      typeof DropLsmShareBondProviderClass
    >;
    nativeBondProviderContractClient?: InstanceType<
      typeof DropNativeBondProviderClass
    >;
    account?: AccountData;
    icaAddress?: string;
    rewardsPumpIcaAddress?: string;
    client?: SigningCosmWasmClient;
    initiaClient?: SigningStargateClient;
    initiaUserAddress?: string;
    initiaUserAddress2?: string;
    initiaQueryClient?: QueryClient & StakingExtension & BankExtension;
    neutronRPCEndpoint?: string;
    neutronClient?: InstanceType<typeof NeutronClient>;
    neutronUserAddress?: string;
    neutronSecondUserAddress?: string;
    validatorAddress?: string;
    secondValidatorAddress?: string;
    tokenizedDenomOnNeutron?: string;
    codeIds: {
      core?: number;
      token?: number;
      withdrawalVoucher?: number;
      withdrawalManager?: number;
      strategy?: number;
      puppeteer?: number;
      validatorsSet?: number;
      distribution?: number;
      rewardsManager?: number;
      splitter?: number;
      pump?: number;
      lsmShareBondProvider?: number;
      nativeBondProvider?: number;
    };
    exchangeRate?: number;
    moveIBCDenom?: string;
    ldDenom?: string;
    moveToken?: {
      metadataAddr: string;
    };
    moveDenom?: string;
    demo1Address?: string;
    initiaNTRN?: string;
    initiaAddress3?: string;
    liquidityProviderModuleAddress?: string;
    liquidityProviderModuleAddressHex?: string;
  } = { codeIds: {} };

  beforeAll(async (t) => {
    context.park = await setupPark(
      t,
      ['neutron', 'initia'],
      {
        initia: {
          genesis_opts: {
            'app_state.mstaking.params.unbonding_time': `${UNBONDING_TIME}s`,
            'app_state.gov.params.min_deposit': [
              { denom: 'uinit', amount: '1000' },
            ],
            'app_state.gov.params.quorum': '0.01',
            'app_state.gov.params.threshold': '0.01',
            'app_state.gov.params.voting_period': '20s',
            'app_state.gov.params.expedited_voting_period': '20s',
          },
        },
      },
      {
        neutron: true,
        hermes: {
          config: {
            'chains.1.trusting_period': '2m0s',
            'chains.1.gas_price.price': 0.2,
          },
        },
      },
    );
    context.wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demowallet1.mnemonic,
      {
        prefix: 'neutron',
      },
    );
    context.initiaWallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demowallet1.mnemonic,
      {
        prefix: 'init',
      },
    );
    context.initiaWallet2 = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demo1.mnemonic,
      {
        prefix: 'init',
      },
    );
    context.initiaAddress3 = (
      await (
        await DirectSecp256k1HdWallet.fromMnemonic(
          context.park.config.wallets.demo3.mnemonic,
          {
            prefix: 'init',
          },
        )
      ).getAccounts()
    )[0].address;
    context.account = (await context.wallet.getAccounts())[0];
    context.neutronClient = new NeutronClient({
      apiURL: `http://127.0.0.1:${context.park.ports.neutron.rest}`,
      rpcURL: `127.0.0.1:${context.park.ports.neutron.rpc}`,
      prefix: 'neutron',
    });
    context.neutronRPCEndpoint = `http://127.0.0.1:${context.park.ports.neutron.rpc}`;
    context.client = await SigningCosmWasmClient.connectWithSigner(
      context.neutronRPCEndpoint,
      context.wallet,
      {
        gasPrice: GasPrice.fromString('0.025untrn'),
      },
    );
    context.initiaClient = await SigningStargateClient.connectWithSigner(
      `http://127.0.0.1:${context.park.ports.initia.rpc}`,
      context.initiaWallet,
      {
        gasPrice: GasPrice.fromString('0.25uinit'),
      },
    );
    const tmClient = await Tendermint34Client.connect(
      `http://127.0.0.1:${context.park.ports.initia.rpc}`,
    );
    context.initiaQueryClient = QueryClient.withExtensions(
      tmClient,
      setupStakingExtension,
      setupBankExtension,
    );
    const secondWallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demo2.mnemonic,
      {
        prefix: 'neutron',
      },
    );
    context.neutronSecondUserAddress = (
      await secondWallet.getAccounts()
    )[0].address;
    context.initiaUserAddress = (
      await context.initiaWallet.getAccounts()
    )[0].address;
    context.initiaUserAddress2 = (
      await context.initiaWallet2.getAccounts()
    )[0].address;
    context.neutronUserAddress = (
      await context.wallet.getAccounts()
    )[0].address;
    {
      const res = await context.park.executeInNetwork(
        'initia',
        `${context.park.config.networks['initia'].binary} query mstaking validators --output json`,
      );
      const validators = JSON.parse(res.out).validators;
      context.validatorAddress = validators[0].operator_address;
      context.secondValidatorAddress = validators[1].operator_address;
    }
  });

  afterAll(async () => {
    await context.park.stop();
  });

  it('create move tokens', async () => {
    const demo1Address = (
      await context.park.executeInNetwork(
        'initia',
        `${context.park.config.networks['initia'].binary} keys show demo1 --home=/opt --keyring-backend=test -a`,
      )
    ).out.trim();
    context.demo1Address = demo1Address;

    await context.park.executeInNetwork(
      'neutron',
      `neutrond tx ibc-transfer transfer transfer channel-0 ${demo1Address} 10000000untrn --from=demo1 --home=/opt --chain-id=ntrntest --keyring-backend=test --fees=10000untrn -y`,
    );

    await waitFor(
      async () => {
        const res = await context.initiaClient.getAllBalances(demo1Address);
        context.initiaNTRN = res.find((r) => r.denom.startsWith('ibc'))?.denom;
        return !!context.initiaNTRN;
      },
      50_000,
      2_000,
    );

    const initiaNTRNmetadata = JSON.parse(
      JSON.parse(
        (
          await context.park.executeInNetwork(
            'initia',
            `initiad query move view 0x1 coin metadata --args '["address:0x1", "string:${context.initiaNTRN}"]' --output=json`,
          )
        ).out,
      ).data,
    );

    const initiaUINITmetadata = JSON.parse(
      JSON.parse(
        (
          await context.park.executeInNetwork(
            'initia',
            `initiad query move view 0x1 coin metadata --args '["address:0x1", "string:uinit"]' --output=json`,
          )
        ).out,
      ).data,
    );
    const createPairTx = JSON.parse(
      (
        await context.park.executeInNetwork(
          'initia',
          `initiad tx move execute 0x1 dex create_pair_script --args '["string:name", "string:INIT_NTRN", "bigdecimal:0.001", "bigdecimal:0.5", "bigdecimal:0.5", "object:${initiaUINITmetadata}", "object:${initiaNTRNmetadata}", "u64:9000000", "u64:9000000"]' --from=demo1  --keyring-backend=test --home=/opt --chain-id=${context.park.config.networks['initia'].chain_id} --fees 3000000uinit --gas=2000000 --output=json -y`,
        )
      ).out,
    );
    await sleep(3_000);

    const createPairRes = JSON.parse(
      (
        await context.park.executeInNetwork(
          'initia',
          `initiad q tx ${createPairTx.txhash} --output=json`,
        )
      ).out,
    );
    const moveEvent = createPairRes.events.find(
      (e) =>
        e.type === 'move' &&
        e.attributes.some(
          (a) =>
            a.key === 'type_tag' && a.value === '0x1::dex::CreatePairEvent',
        ),
    );
    const moveDataAttr = JSON.parse(
      moveEvent.attributes.find((a) => a.key === 'data').value,
    );
    context.moveToken = {
      metadataAddr: moveDataAttr.liquidity_token,
    };
    const balanceQuery = await context.park.executeInNetwork(
      'initia',
      `${context.park.config.networks['initia'].binary} query bank balances ${demo1Address} --output=json`,
    );
    context.moveDenom = JSON.parse(balanceQuery.out).balances.find((o) =>
      o.denom.startsWith('move'),
    ).denom;
    expect(context.moveDenom).toBeTruthy();
  });

  it('submit proposal to add bond denom', async () => {
    await context.park.executeInNetwork(
      'initia',
      `initiad tx mstaking delegate ${context.validatorAddress} 900000000uinit --from=demowallet1  --keyring-backend=test --home=/opt --chain-id=${context.park.config.networks['initia'].chain_id} --fees 300000uinit --gas 2000000 --output=json -y`,
    );
    await context.park.executeInNetwork(
      'initia',
      `initiad tx mstaking delegate ${context.validatorAddress} 900000000uinit --from=demo2  --keyring-backend=test --home=/opt --chain-id=${context.park.config.networks['initia'].chain_id} --fees 300000uinit --gas 2000000 --output=json -y`,
    );
    await context.park.executeInNetwork(
      'initia',
      `initiad tx mstaking delegate ${context.validatorAddress} 900000000uinit --from=demo3  --keyring-backend=test --home=/opt --chain-id=${context.park.config.networks['initia'].chain_id} --fees 300000uinit --gas 2000000 --output=json -y`,
    );
    await sleep(4_000);
    const modules = JSON.parse(
      (
        await context.park.executeInNetwork(
          'initia',
          `${context.park.config.networks['initia'].binary} query auth module-accounts --output=json`,
        )
      ).out,
    );
    const govAddress = modules.accounts.find((a) => a.value.name === 'gov')
      .value.address;

    await context.park.executeInNetwork(
      'initia',
      `
      echo '${JSON.stringify({
        messages: [
          {
            '@type': '/initia.move.v1.MsgWhitelist',
            authority: govAddress,
            metadata_lp: AccAddress.fromHex(
              context.moveDenom.replace(/^move./, ''),
            ),
            reward_weight: '1.000000000000000000',
          },
        ],
        metadata: 'ipfs://CID',
        deposit: '1000000uinit',
        title: 'title',
        summary: 'sum',
        expedited: false,
      })}' > proposal.json 
    `,
    );
    await context.park.executeInNetwork(
      'initia',
      `initiad tx gov submit-proposal ./proposal.json --from=demo1  --keyring-backend=test --home=/opt --chain-id=${context.park.config.networks['initia'].chain_id} --gas 20000000 --fees 3000000uinit --output=json -y`,
    );
    await sleep(4_000);
    await context.park.executeInNetwork(
      'initia',
      `${context.park.config.networks['initia'].binary}  tx gov deposit 1 1000000uinit --from=demo1  --keyring-backend=test --home=/opt --chain-id=${context.park.config.networks['initia'].chain_id} --fees 30000uinit --output=json -y`,
    );
    await sleep(4_000);
    await context.park.executeInNetwork(
      'initia',
      `initiad  tx gov vote 1 yes --from=demowallet1  --keyring-backend=test --home=/opt --chain-id=${context.park.config.networks['initia'].chain_id} --fees 30000uinit --output=json -y`,
    );
    await context.park.executeInNetwork(
      'initia',
      `initiad  tx gov vote 1 yes --from=demo2  --keyring-backend=test --home=/opt --chain-id=${context.park.config.networks['initia'].chain_id} --fees 30000uinit --output=json -y`,
    );
    await context.park.executeInNetwork(
      'initia',
      `initiad  tx gov vote 1 yes --from=demo3  --keyring-backend=test --home=/opt --chain-id=${context.park.config.networks['initia'].chain_id} --fees 30000uinit --output=json -y`,
    );
    await sleep(4_000);
    await waitFor(
      async () => {
        const res = JSON.parse(
          (
            await context.park.executeInNetwork(
              'initia',
              `${context.park.config.networks['initia'].binary} query mstaking params --output=json`,
            )
          ).out,
        );
        return res.bond_denoms.length > 1;
      },
      60_000,
      1_000,
    );
  });

  it('transfer tokens to neutron', async () => {
    const { neutronUserAddress, neutronClient } = context;

    await context.park.executeInNetwork(
      'initia',
      `initiad tx ibc-transfer transfer transfer channel-0 ${neutronUserAddress} 8900000${context.moveDenom} --from=demo1 --home=/opt --chain-id=${context.park.config.networks['initia'].chain_id} --keyring-backend=test --fees=50000uinit --gas 300000 -y`,
    );

    await waitFor(async () => {
      const balances =
        await neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
          neutronUserAddress,
        );
      context.moveIBCDenom = balances.data.balances.find((b) =>
        b.denom.startsWith('ibc/'),
      )?.denom;
      return balances.data.balances.length > 1;
    }, 60_000);
    expect(context.moveIBCDenom).toBeTruthy();
  });

  it('instantiate', async () => {
    const { client, account } = context;
    context.codeIds = {};

    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(join(__dirname, '../../../artifacts/drop_core.wasm')),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.core = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_token.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.token = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_withdrawal_voucher.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.withdrawalVoucher = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_withdrawal_manager.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.withdrawalManager = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(join(__dirname, '../../../artifacts/drop_pump.wasm')),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.pump = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_strategy.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.strategy = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_distribution.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.distribution = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_validators_set.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.validatorsSet = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_puppeteer_initia.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.puppeteer = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_rewards_manager.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.rewardsManager = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_splitter.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.splitter = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(
              __dirname,
              '../../../artifacts/drop_lsm_share_bond_provider.wasm',
            ),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.lsmShareBondProvider = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(
              __dirname,
              '../../../artifacts/drop_native_bond_provider.wasm',
            ),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.nativeBondProvider = res.codeId;
    }

    const res = await client.upload(
      account.address,
      Uint8Array.from(
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_factory.wasm'),
        ),
      ),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await DropFactory.Client.instantiate(
      client,
      account.address,
      res.codeId,
      {
        sdk_version: process.env.SDK_VERSION || '0.46.0',
        local_denom: 'untrn',
        code_ids: {
          core_code_id: context.codeIds.core,
          token_code_id: context.codeIds.token,
          withdrawal_voucher_code_id: context.codeIds.withdrawalVoucher,
          withdrawal_manager_code_id: context.codeIds.withdrawalManager,
          strategy_code_id: context.codeIds.strategy,
          distribution_code_id: context.codeIds.distribution,
          validators_set_code_id: context.codeIds.validatorsSet,
          puppeteer_code_id: context.codeIds.puppeteer,
          rewards_manager_code_id: context.codeIds.rewardsManager,
          splitter_code_id: context.codeIds.splitter,
          rewards_pump_code_id: context.codeIds.pump,
          lsm_share_bond_provider_code_id: context.codeIds.lsmShareBondProvider,
          native_bond_provider_code_id: context.codeIds.nativeBondProvider,
        },
        remote_opts: {
          connection_id: 'connection-0',
          transfer_channel_id: 'channel-0',
          reverse_transfer_channel_id: 'channel-0',
          port_id: 'transfer',
          denom: context.moveDenom,
          update_period: 2,
          timeout: {
            local: 60,
            remote: 60,
          },
        },
        salt: 'salt',
        subdenom: 'drop',
        token_metadata: {
          description: 'Drop token',
          display: 'drop',
          exponent: 6,
          name: 'Drop liquid staking token',
          symbol: 'DROP',
          uri: null,
          uri_hash: null,
        },
        base_denom: context.moveIBCDenom,
        core_params: {
          idle_min_interval: 120,
          unbond_batch_switch_time: 60,
          unbonding_safe_period: 10,
          unbonding_period: 360,
          icq_update_delay: 5,
        },
        native_bond_params: {
          min_stake_amount: '10000',
          min_ibc_transfer: '10000',
        },
        lsm_share_bond_params: {
          lsm_redeem_threshold: 2,
          lsm_min_bond_amount: '1000',
          lsm_redeem_max_interval: 60_000,
        },
      },
      'drop-staking-factory',
      'auto',
      [],
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.contractAddress = instantiateRes.contractAddress;
    context.factoryContractClient = new DropFactory.Client(
      client,
      context.contractAddress,
    );
  });

  it('query factory state', async () => {
    const { factoryContractClient: contractClient, neutronClient } = context;
    const res = await contractClient.queryState();
    expect(res).toBeTruthy();
    const tokenContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.token_contract,
      );
    expect(tokenContractInfo.data.contract_info.label).toBe(
      'drop-staking-token',
    );
    const coreContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.core_contract,
      );
    expect(coreContractInfo.data.contract_info.label).toBe('drop-staking-core');
    const withdrawalVoucherContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.withdrawal_voucher_contract,
      );
    expect(withdrawalVoucherContractInfo.data.contract_info.label).toBe(
      'drop-staking-withdrawal-voucher',
    );
    const withdrawalManagerContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.withdrawal_manager_contract,
      );
    expect(withdrawalManagerContractInfo.data.contract_info.label).toBe(
      'drop-staking-withdrawal-manager',
    );
    const puppeteerContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.puppeteer_contract,
      );
    expect(puppeteerContractInfo.data.contract_info.label).toBe(
      'drop-staking-puppeteer',
    );
    context.coreContractClient = instrumentCoreClass(
      new DropCore.Client(context.client, res.core_contract),
    );
    context.withdrawalVoucherContractClient = new DropWithdrawalVoucher.Client(
      context.client,
      res.withdrawal_voucher_contract,
    );
    context.withdrawalManagerContractClient = new DropWithdrawalManager.Client(
      context.client,
      res.withdrawal_manager_contract,
    );
    context.rewardsManagerContractClient = new DropRewardsManager.Client(
      context.client,
      res.rewards_manager_contract,
    );
    context.strategyContractClient = new DropStrategy.Client(
      context.client,
      res.strategy_contract,
    );
    context.rewardsPumpContractClient = new DropPump.Client(
      context.client,
      res.rewards_pump_contract,
    );
    context.tokenContractClient = new DropToken.Client(
      context.client,
      res.token_contract,
    );
    context.puppeteerContractClient = new DropPuppeteerInitia.Client(
      context.client,
      res.puppeteer_contract,
    );
    context.splitterContractClient = new DropSplitter.Client(
      context.client,
      res.splitter_contract,
    );
    context.lsmShareBondProviderContractClient =
      new DropLsmShareBondProvider.Client(
        context.client,
        res.lsm_share_bond_provider_contract,
      );
    context.nativeBondProviderContractClient =
      new DropNativeBondProvider.Client(
        context.client,
        res.native_bond_provider_contract,
      );
  });

  it('setup ICA for rewards pump', async () => {
    const { rewardsPumpContractClient, neutronUserAddress } = context;
    const res = await rewardsPumpContractClient.registerICA(
      neutronUserAddress,
      1.5,
      undefined,
      [{ amount: '1000000', denom: 'untrn' }],
    );
    expect(res.transactionHash).toHaveLength(64);
    let ica = '';
    await waitFor(async () => {
      const res = await rewardsPumpContractClient.queryIca();
      switch (res) {
        case 'none':
        case 'in_progress':
        case 'timeout':
          return false;
        default:
          ica = res.registered.ica_address;
          return true;
      }
    }, 100_000);
    expect(ica).toHaveLength(63);
    expect(ica.startsWith('init')).toBeTruthy();
    context.rewardsPumpIcaAddress = ica;
  });

  it('register puppeteer ICA', async () => {
    const { puppeteerContractClient, neutronUserAddress } = context;
    const res = await puppeteerContractClient.registerICA(
      neutronUserAddress,
      1.5,
      undefined,
      [{ amount: '1000000', denom: 'untrn' }],
    );
    expect(res.transactionHash).toHaveLength(64);
    let ica = '';
    await waitFor(async () => {
      const res = await puppeteerContractClient.queryIca();
      switch (res) {
        case 'none':
        case 'in_progress':
        case 'timeout':
          return false;
        default:
          ica = res.registered.ica_address;
          return true;
      }
    }, 100_000);
    expect(ica).toHaveLength(63);
    expect(ica.startsWith('init')).toBeTruthy();
    context.icaAddress = ica;
  });

  it('transfer some tokens to the puppeteer ICA', async () => {
    await context.park.executeInNetwork(
      'initia',
      `initiad tx bank send ${context.initiaUserAddress2} ${context.icaAddress} 1000${context.moveDenom} --from=demo1 --home=/opt --chain-id=${context.park.config.networks['initia'].chain_id} --keyring-backend=test --fees=50000uinit --gas 300000 -y`,
    );
    await sleep(10_000);

    const balances = await context.initiaClient.getAllBalances(
      context.icaAddress,
    );
    expect(balances).toEqual([
      {
        amount: '1000',
        denom: context.moveDenom,
      },
    ]);
  });

  it('build movevm lp module', async () => {
    const ownerAddress =
      '0x' +
      (
        await context.park.executeInNetwork(
          'initia',
          `initiad keys parse ${context.initiaAddress3} | grep bytes | awk '{print $2}' | tr -d '\n'`,
        )
      ).out;

    await context.park.executeInNetwork(
      'initia',
      `initiad move build --path /movevm/liquidity-provider --force --named-addresses 'me=${ownerAddress}'`,
    );
  });

  it('upload movevm lp module', async () => {
    await context.park.executeInNetwork(
      'initia',
      `initiad move deploy --path /movevm/liquidity-provider --upgrade-policy COMPATIBLE --from demo3 --home /opt --keyring-backend test --gas auto --gas-adjustment 1.5 --gas-prices 0.025uinit --chain-id ${context.park.config.networks['initia'].chain_id} -o json -y`,
    );
    await sleep(10_000);
  });

  it('instantiate movevm lp module', async () => {
    const ownerAddress =
      '0x' +
      (
        await context.park.executeInNetwork(
          'initia',
          `initiad keys parse ${context.initiaAddress3} | grep bytes | awk '{print $2}' | tr -d '\n'`,
        )
      ).out;

    const initiaUINITmetadata = JSON.parse(
      JSON.parse(
        (
          await context.park.executeInNetwork(
            'initia',
            `initiad query move view 0x1 coin metadata --args '["address:0x1", "string:uinit"]' --output=json`,
          )
        ).out,
      ).data,
    );

    const rewardsPumpIcaAddress =
      '0x' +
      (
        await context.park.executeInNetwork(
          'initia',
          `initiad keys parse ${context.rewardsPumpIcaAddress} | grep bytes | awk '{print $2}' | tr -d '\n'`,
        )
      ).out;

    let res = JSON.parse(
      (
        await context.park.executeInNetwork(
          'initia',
          `initiad tx move execute ${ownerAddress} drop_lp create_liquidity_provider --args '["string:test_uinit", "address:${ownerAddress}", "string:INIT/USD", "bool:true", "object:${context.moveToken.metadataAddr}", "object:${initiaUINITmetadata}", "address:${rewardsPumpIcaAddress}"]' --from demo3 --home /opt --gas auto --gas-adjustment 1.5 --gas-prices 0.025uinit --chain-id ${context.park.config.networks['initia'].chain_id} --keyring-backend test -y -o json`,
        )
      ).out,
    );
    await sleep(10_000);
    res = JSON.parse(
      (
        await context.park.executeInNetwork(
          'initia',
          `initiad query tx --type=hash ${res.txhash} -o json`,
        )
      ).out,
    );
    const moveEvents = res.events
      .filter((event) => event.type == 'move')
      .map((event) => event.attributes)
      .flat(1);
    const createObjectEvent =
      moveEvents[
        moveEvents.findIndex(
          (attribute) =>
            attribute.key == 'type_tag' &&
            attribute.value == '0x1::object::CreateEvent',
        ) + 1
      ];
    context.liquidityProviderModuleAddressHex = JSON.parse(
      createObjectEvent.value,
    ).object;
    context.liquidityProviderModuleAddress = (
      await context.park.executeInNetwork(
        'initia',
        `initiad keys parse ${context.liquidityProviderModuleAddressHex.substring(2)} | grep init1 | awk '{print $2}' | tr -d '\n'`,
      )
    ).out;
  });

  it('set up rewards receiver', async () => {
    const { neutronUserAddress, liquidityProviderModuleAddress } = context;
    const res = await context.factoryContractClient.adminExecute(
      neutronUserAddress,
      {
        msgs: [
          {
            wasm: {
              execute: {
                contract_addr: context.puppeteerContractClient.contractAddress,
                msg: Buffer.from(
                  JSON.stringify({
                    setup_protocol: {
                      rewards_withdraw_address: liquidityProviderModuleAddress,
                    },
                  }),
                ).toString('base64'),
                funds: [
                  {
                    amount: '20000',
                    denom: 'untrn',
                  },
                ],
              },
            },
          },
        ],
      },
      1.5,
      undefined,
      [
        {
          amount: '20000',
          denom: 'untrn',
        },
      ],
    );
    expect(res.transactionHash).toHaveLength(64);
    const pupRes = await context.puppeteerContractClient.queryTxState();
    expect(pupRes.status).toBe('waiting_for_ack');
  });

  it('wait puppeteer response', async () => {
    const { puppeteerContractClient } = context;
    await waitFor(async () => {
      const res = await puppeteerContractClient.queryTxState();
      return res.status === 'idle';
    }, 100_000);
  });

  it('query exchange rate', async () => {
    const { coreContractClient } = context;
    context.exchangeRate = parseFloat(
      await coreContractClient.queryExchangeRate(),
    );
    expect(context.exchangeRate).toEqual(1);
    await checkExchangeRate(context);
  });

  it('register native bond provider in the core', async () => {
    const res = await context.factoryContractClient.adminExecute(
      context.neutronUserAddress,
      {
        msgs: [
          {
            wasm: {
              execute: {
                contract_addr: context.coreContractClient.contractAddress,
                msg: Buffer.from(
                  JSON.stringify({
                    add_bond_provider: {
                      bond_provider_address:
                        context.nativeBondProviderContractClient
                          .contractAddress,
                    },
                  }),
                ).toString('base64'),
                funds: [],
              },
            },
          },
        ],
      },
      1.5,
      undefined,
      [],
    );
    expect(res.transactionHash).toHaveLength(64);
  });

  it('bond w/o receiver', async () => {
    const {
      coreContractClient,
      neutronClient,
      neutronUserAddress,
      moveIBCDenom,
    } = context;
    const res = await coreContractClient.bond(
      neutronUserAddress,
      {},
      1.6,
      undefined,
      [
        {
          amount: '500000',
          denom: moveIBCDenom,
        },
      ],
    );
    expect(res.transactionHash).toHaveLength(64);
    const balances =
      await neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
        neutronUserAddress,
      );
    expect(
      balances.data.balances.find((one) => one.denom.startsWith('factory')),
    ).toEqual({
      denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
      amount: String(Math.floor(500_000 / context.exchangeRate)),
    });
    await checkExchangeRate(context);
  });

  it('bond with receiver', async () => {
    const {
      coreContractClient,
      neutronClient,
      neutronUserAddress,
      moveIBCDenom: neutronIBCDenom,
      neutronSecondUserAddress,
    } = context;
    const res = await coreContractClient.bond(
      neutronUserAddress,
      { receiver: neutronSecondUserAddress },
      1.6,
      undefined,
      [
        {
          amount: '500000',
          denom: neutronIBCDenom,
        },
      ],
    );
    expect(res.transactionHash).toHaveLength(64);
    const balances =
      await neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
        neutronSecondUserAddress,
      );
    const ldBalance = balances.data.balances.find((one) =>
      one.denom.startsWith('factory'),
    );
    expect(ldBalance).toEqual({
      denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
      amount: String(Math.floor(500_000 / context.exchangeRate)),
    });
    context.ldDenom = ldBalance?.denom;
    await checkExchangeRate(context);
  });

  it('add validators into validators set', async () => {
    const {
      neutronUserAddress,
      factoryContractClient,
      validatorAddress,
      secondValidatorAddress,
    } = context;
    const res = await factoryContractClient.proxy(
      neutronUserAddress,
      {
        validator_set: {
          update_validators: {
            validators: [
              {
                valoper_address: validatorAddress,
                weight: 1,
                on_top: '0',
              },
              {
                valoper_address: secondValidatorAddress,
                weight: 1,
                on_top: '0',
              },
            ],
          },
        },
      },
      1.5,
      undefined,
      [
        {
          amount: '1000000',
          denom: 'untrn',
        },
      ],
    );
    expect(res.transactionHash).toHaveLength(64);
    await sleep(10_000);
    await context.park.restartRelayer('neutron', 1);
  });

  it('unbond', async () => {
    const { coreContractClient, neutronUserAddress, ldDenom } = context;
    let res = await coreContractClient.unbond(
      neutronUserAddress,
      1.6,
      undefined,
      [
        {
          amount: Math.floor(200_000 / context.exchangeRate).toString(),
          denom: ldDenom,
        },
      ],
    );
    expect(res.transactionHash).toHaveLength(64);
    res = await coreContractClient.unbond(neutronUserAddress, 1.6, undefined, [
      {
        amount: Math.floor(300_000 / context.exchangeRate).toString(),
        denom: ldDenom,
      },
    ]);
    expect(res.transactionHash).toHaveLength(64);
    await checkExchangeRate(context);
  });

  it('validate unbonding batch', async () => {
    const batch = await context.coreContractClient.queryUnbondBatch({
      batch_id: '0',
    });
    expect(batch).toBeTruthy();
    expect(batch).toEqual<UnbondBatch>({
      slashing_effect: null,
      status_timestamps: expect.any(Object),
      expected_release_time: 0,
      status: 'new',
      total_dasset_amount_to_withdraw: '500000',
      expected_native_asset_amount: '0',
      total_unbond_items: 2,
      unbonded_amount: null,
      withdrawn_amount: null,
    });
  });

  describe('state machine', () => {
    const ica: { balance?: number } = {};
    describe('prepare', () => {
      it('get ICA balance', async () => {
        const { initiaClient } = context;
        const res = await initiaClient.getBalance(
          context.icaAddress,
          context.moveDenom,
        );
        ica.balance = parseInt(res.amount);
        expect(ica.balance).toEqual(1000);
      });
      it('deploy pump', async () => {
        const { client, account, neutronUserAddress } = context;
        const resUpload = await client.upload(
          account.address,
          Uint8Array.from(
            fs.readFileSync(
              join(__dirname, '../../../artifacts/drop_pump.wasm'),
            ),
          ),
          1.5,
        );
        expect(resUpload.codeId).toBeGreaterThan(0);
        const { codeId } = resUpload;
        const res = await DropPump.Client.instantiate(
          client,
          neutronUserAddress,
          codeId,
          {
            connection_id: 'connection-0',
            local_denom: 'untrn',
            timeout: {
              local: 60,
              remote: 60,
            },
            dest_address:
              context.withdrawalManagerContractClient.contractAddress,
            dest_port: 'transfer',
            dest_channel: 'channel-0',
            refundee: neutronUserAddress,
            owner: account.address,
          },
          'drop-staking-pump',
          1.5,
          [],
        );
        expect(res.contractAddress).toHaveLength(66);
        context.pumpContractClient = new DropPump.Client(
          client,
          res.contractAddress,
        );
        await context.pumpContractClient.registerICA(
          neutronUserAddress,
          1.5,
          undefined,
          [
            {
              amount: '1000000',
              denom: 'untrn',
            },
          ],
        );
        let ica = '';
        await waitFor(async () => {
          const res = await context.pumpContractClient.queryIca();
          switch (res) {
            case 'none':
            case 'in_progress':
            case 'timeout':
              return false;
            default:
              ica = res.registered.ica_address;
              return true;
          }
        }, 50_000);
        expect(ica).toHaveLength(63);
        expect(ica.startsWith('init')).toBeTruthy();
        const resFactory = await context.factoryContractClient.updateConfig(
          neutronUserAddress,
          {
            core: {
              pump_ica_address: ica,
            },
          },
        );
        expect(resFactory.transactionHash).toHaveLength(64);
      });
      it('get machine state', async () => {
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('idle');
      });
    });
    describe('paused tick', () => {
      it('pause protocol', async () => {
        const {
          account,
          factoryContractClient: contractClient,
          neutronUserAddress,
        } = context;

        await contractClient.pause(account.address);

        await expect(
          context.coreContractClient.tick(neutronUserAddress, 1.5, undefined, [
            {
              amount: '1000000',
              denom: 'untrn',
            },
          ]),
        ).rejects.toThrowError(/Contract execution is paused/);

        await contractClient.unpause(account.address);
      });
    });
    describe('first cycle', () => {
      it('first tick did nothing and stays in idle', async () => {
        const {
          initiaClient,
          neutronUserAddress,
          coreContractClient,
          puppeteerContractClient,
        } = context;

        await waitForPuppeteerICQ(
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
        );

        const res = await context.coreContractClient.tick(
          neutronUserAddress,
          1.5,
          undefined,
          [],
        );
        expect(res.transactionHash).toHaveLength(64);

        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('idle');
      });
      it('tick 1 (peripheral) transfer coins from neutron to target chain', async () => {
        const {
          client,
          neutronUserAddress,
          moveIBCDenom: neutronIBCDenom,
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
          nativeBondProviderContractClient,
        } = context;

        const balancesBefore = await context.initiaClient.getAllBalances(
          context.icaAddress,
        );

        expect(balancesBefore).toEqual([
          {
            amount: '1000',
            denom: context.moveDenom,
          },
        ]);

        await waitForPuppeteerICQ(
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
        );

        await context.coreContractClient.tick(
          neutronUserAddress,
          1.5,
          undefined,
          [
            {
              amount: '100000',
              denom: 'untrn',
            },
          ],
        );
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('peripheral');

        await waitFor(async () => {
          const res =
            await context.nativeBondProviderContractClient.queryTxState();
          return res.status === 'idle';
        }, 100_000);

        const balances = await context.initiaClient.getAllBalances(
          context.icaAddress,
        );
        expect(balances).toEqual([
          {
            amount: '1001000',
            denom: context.moveDenom,
          },
        ]);

        const res = await client.getBalance(
          nativeBondProviderContractClient.contractAddress,
          neutronIBCDenom,
        );
        const balance = parseInt(res.amount);
        expect(balance).toEqual(0);
      });
      it('tick 2 (idle)', async () => {
        const {
          initiaClient,
          neutronUserAddress,
          coreContractClient,
          puppeteerContractClient,
        } = context;

        await waitForPuppeteerICQ(
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
        );

        const res = await context.coreContractClient.tick(
          neutronUserAddress,
          1.5,
          undefined,
          [],
        );
        expect(res.transactionHash).toHaveLength(64);

        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('idle');
      });
      it('tick 3 (peripheral) stake collected coins on remote chain', async () => {
        const {
          neutronUserAddress,
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
        } = context;

        await waitForPuppeteerICQ(
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
        );

        await context.coreContractClient.tick(
          neutronUserAddress,
          1.5,
          undefined,
          [
            {
              amount: '1000000',
              denom: 'untrn',
            },
          ],
        );
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('peripheral');
        let res;
        await waitFor(async () => {
          try {
            res = await context.puppeteerContractClient.queryExtension({
              msg: {
                delegations: {},
              },
            });
          } catch (e) {
            //
          }
          return res && res.delegations.delegations.length !== 0;
        }, 100_000);

        await checkExchangeRate(context);
      });
      it('tick 4 (idle)', async () => {
        const {
          initiaClient,
          neutronUserAddress,
          coreContractClient,
          puppeteerContractClient,
        } = context;

        await waitForPuppeteerICQ(
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
        );

        const res = await context.coreContractClient.tick(
          neutronUserAddress,
          1.5,
          undefined,
          [],
        );
        expect(res.transactionHash).toHaveLength(64);

        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('idle');
      });
      it('decrease idle interval', async () => {
        const { factoryContractClient, neutronUserAddress } = context;
        const res = await factoryContractClient.updateConfig(
          neutronUserAddress,
          {
            core: {
              idle_min_interval: 30,
            },
          },
        );
        expect(res.transactionHash).toHaveLength(64);
      });
      it('wait delegations', async () => {
        let delegations = [];
        await waitFor(async () => {
          const res: any = await context.puppeteerContractClient.queryExtension(
            {
              msg: {
                delegations: {},
              },
            },
          );
          delegations = res.delegations.delegations;
          return res && res.delegations.delegations.length > 0;
        }, 100_000);
        expect(delegations.length).toEqual(2);
        const [delegation] = delegations;
        expect(delegation.validator).toEqual(context.validatorAddress);
        expect(delegation.amount).toMatchObject({
          amount: '500000',
          denom: context.moveDenom,
        });
      });
      it('tick goes to claiming', async () => {
        const {
          neutronUserAddress,
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
        } = context;

        await waitForPuppeteerICQ(
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
        );

        const res = await context.coreContractClient.tick(
          neutronUserAddress,
          1.5,
          undefined,
          [
            {
              amount: '1000000',
              denom: 'untrn',
            },
          ],
        );
        expect(res.transactionHash).toHaveLength(64);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('claiming');
        await checkExchangeRate(context);
      });
      it('tick goes to unbonding', async () => {
        const {
          neutronUserAddress,
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
        } = context;

        await waitForPuppeteerICQ(
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
        );

        await sleep(10_000); // XXX: why does tick() sometimes fail with "rpc error: code = Unknown desc = failed to execute message; message index: 0: Puppeteer balance is outdated: ICA height 197, control height 196: execute wasm contract failed"?
        const res = await context.coreContractClient.tick(
          neutronUserAddress,
          1.5,
          undefined,
          [
            {
              amount: '1000000',
              denom: 'untrn',
            },
          ],
        );
        expect(res.transactionHash).toHaveLength(64);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('unbonding');
        await checkExchangeRate(context);
      });
      it('tick is failed bc no response from puppeteer yet', async () => {
        const { neutronUserAddress } = context;
        await expect(
          context.coreContractClient.tick(
            neutronUserAddress,
            1.5,
            undefined,
            [],
          ),
        ).rejects.toThrowError(/Puppeteer response is not received/);
      });
      it('query one unbonding batch', async () => {
        const batch = await context.coreContractClient.queryUnbondBatch({
          batch_id: '0',
        });
        expect(batch).toBeTruthy();
        expect(batch).toEqual<UnbondBatch>({
          slashing_effect: null,
          status: 'unbond_requested',
          status_timestamps: expect.any(Object),
          expected_release_time: 0,
          total_dasset_amount_to_withdraw: '500000',
          expected_native_asset_amount: '500000',
          total_unbond_items: 2,
          unbonded_amount: null,
          withdrawn_amount: null,
        });
      });
      it('query all unbonding batches at once', async () => {
        const { unbond_batches: unbondBatches, next_page_key: nextPageKey } =
          await context.coreContractClient.queryUnbondBatches({});

        expect(unbondBatches.length).toEqual(2);
        expect(nextPageKey).toBeNull();

        const [firstBatch, secondBatch] = unbondBatches;
        expect(firstBatch).toBeTruthy();
        expect(secondBatch).toBeTruthy();
        expect(firstBatch).toEqual<UnbondBatch>({
          slashing_effect: null,
          status: 'unbond_requested',
          status_timestamps: expect.any(Object),
          expected_release_time: 0,
          total_dasset_amount_to_withdraw: '500000',
          expected_native_asset_amount: '500000',
          total_unbond_items: 2,
          unbonded_amount: null,
          withdrawn_amount: null,
        });
        expect(secondBatch).toEqual<UnbondBatch>({
          slashing_effect: null,
          status: 'new',
          status_timestamps: expect.any(Object),
          expected_release_time: 0,
          total_dasset_amount_to_withdraw: '0',
          expected_native_asset_amount: '0',
          total_unbond_items: 0,
          unbonded_amount: null,
          withdrawn_amount: null,
        });
      });
      it('query all unbonding batches with limit and page key', async () => {
        const {
          unbond_batches: firstUnbondBatches,
          next_page_key: firstNextPageKey,
        } = await context.coreContractClient.queryUnbondBatches({
          limit: '1',
        });

        expect(firstUnbondBatches.length).toEqual(1);
        expect(firstNextPageKey).toBeTruthy();

        const [firstBatch] = firstUnbondBatches;
        expect(firstBatch).toBeTruthy();
        expect(firstBatch).toEqual<UnbondBatch>({
          slashing_effect: null,
          status: 'unbond_requested',
          status_timestamps: expect.any(Object),
          expected_release_time: 0,
          total_dasset_amount_to_withdraw: '500000',
          expected_native_asset_amount: '500000',
          total_unbond_items: 2,
          unbonded_amount: null,
          withdrawn_amount: null,
        });

        const {
          unbond_batches: secondUnbondBatches,
          next_page_key: secondNextPageKey,
        } = await context.coreContractClient.queryUnbondBatches({
          limit: '1',
          page_key: firstNextPageKey,
        });

        expect(secondUnbondBatches.length).toEqual(1);
        expect(secondNextPageKey).toBeNull();

        const [secondBatch] = secondUnbondBatches;
        expect(firstBatch).toBeTruthy();
        expect(secondBatch).toEqual<UnbondBatch>({
          slashing_effect: null,
          status: 'new',
          status_timestamps: expect.any(Object),
          expected_release_time: 0,
          total_dasset_amount_to_withdraw: '0',
          expected_native_asset_amount: '0',
          total_unbond_items: 0,
          unbonded_amount: null,
          withdrawn_amount: null,
        });
      });
      it('wait for response from puppeteer', async () => {
        let response;
        await waitFor(async () => {
          try {
            response = (
              await context.coreContractClient.queryLastPuppeteerResponse()
            ).response;
          } catch (e) {
            //
          }
          return !!response;
        }, 100_000);
      });
      it('next tick goes to idle', async () => {
        const {
          neutronUserAddress,
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
        } = context;

        await waitForPuppeteerICQ(
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
        );

        const res = await context.coreContractClient.tick(
          neutronUserAddress,
          1.5,
          undefined,
          [],
        );
        expect(res.transactionHash).toHaveLength(64);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('idle');
        await checkExchangeRate(context);
      });
      it('verify that unbonding batch is in unbonding state', async () => {
        const batch = await context.coreContractClient.queryUnbondBatch({
          batch_id: '0',
        });
        expect(batch).toBeTruthy();
        expect(batch).toEqual<UnbondBatch>({
          slashing_effect: null,
          status: 'unbonding',
          status_timestamps: expect.any(Object),
          expected_release_time: expect.any(Number),
          total_dasset_amount_to_withdraw: '500000',
          expected_native_asset_amount: '500000',
          total_unbond_items: 2,
          unbonded_amount: null,
          withdrawn_amount: null,
        });
      });
    });
    describe('second cycle', () => {
      it('idle tick', async () => {
        const {
          neutronUserAddress,
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
        } = context;

        await waitForPuppeteerICQ(
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
        );

        const res = await context.coreContractClient.tick(
          neutronUserAddress,
          1.5,
          undefined,
          [],
        );
        expect(res.transactionHash).toHaveLength(64);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('claiming');
        await checkExchangeRate(context);
      });
      it('wait for response from puppeteer', async () => {
        let response;
        await waitFor(async () => {
          try {
            response = (
              await context.coreContractClient.queryLastPuppeteerResponse()
            ).response;
          } catch (e) {
            //
          }
          return !!response;
        }, 100_000);
      });
      it('trigger liquidity provider', async () => {
        const ownerAddress =
          '0x' +
          (
            await context.park.executeInNetwork(
              'initia',
              `initiad keys parse ${context.initiaAddress3} | grep bytes | awk '{print $2}' | tr -d '\n'`,
            )
          ).out;

        await context.park.executeInNetwork(
          'initia',
          `initiad tx move execute ${ownerAddress} drop_lp provide --args '["address:${context.liquidityProviderModuleAddressHex}"]' --from demo3 --home /opt --gas auto --gas-adjustment 1.5 --gas-prices 0.025uinit --chain-id ${context.park.config.networks['initia'].chain_id} --keyring-backend test -y`,
        );
        await sleep(10_000);
      });

      it('get rewards pump ICA balance', async () => {
        const { initiaClient } = context;
        const res = await initiaClient.getBalance(
          context.rewardsPumpIcaAddress,
          context.moveDenom,
        );
        const newBalance = parseInt(res.amount);
        expect(newBalance).toBeGreaterThan(0);
      });
      it('wait for balance to update', async () => {
        const { remote_height: currentHeight } =
          (await context.puppeteerContractClient.queryExtension({
            msg: {
              balances: {},
            },
          })) as any;
        await waitFor(async () => {
          const { remote_height: nowHeight } =
            (await context.puppeteerContractClient.queryExtension({
              msg: {
                balances: {},
              },
            })) as any;
          return nowHeight !== currentHeight;
        }, 30_000);
      });
      it('next tick goes to idle', async () => {
        const {
          neutronUserAddress,
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
        } = context;

        await waitForPuppeteerICQ(
          initiaClient,
          coreContractClient,
          puppeteerContractClient,
        );

        const res = await context.coreContractClient.tick(
          neutronUserAddress,
          1.5,
          undefined,
          [],
        );
        expect(res.transactionHash).toHaveLength(64);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('idle');
        await checkExchangeRate(context);
      });
    });

    describe('sixth stake rewards', () => {
      let rewardsPumpIcaBalance = 0;
      it('pump rewards', async () => {
        const { rewardsPumpContractClient, neutronUserAddress, initiaClient } =
          context;
        rewardsPumpIcaBalance = parseInt(
          (
            await initiaClient.getBalance(
              context.rewardsPumpIcaAddress,
              context.moveDenom,
            )
          ).amount,
          10,
        );

        await rewardsPumpContractClient.push(
          neutronUserAddress,
          {
            coins: [
              {
                amount: rewardsPumpIcaBalance.toString(),
                denom: context.moveDenom,
              },
            ],
          },
          1.5,
          undefined,
          [{ amount: '20000', denom: 'untrn' }],
        );
        await waitFor(async () => {
          const balances =
            await context.neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
              context.splitterContractClient.contractAddress,
            );
          return balances.data.balances.length > 0;
        }, 60_000);
      });
      it('split it', async () => {
        const res = await context.splitterContractClient.distribute(
          context.neutronUserAddress,
          1.5,
          undefined,
        );
        expect(res.transactionHash).toHaveLength(64);

        const nativeBondProviderBalance = (
          await context.neutronClient.CosmosBankV1Beta1.query.queryBalance(
            context.nativeBondProviderContractClient.contractAddress,
            { denom: context.moveIBCDenom },
          )
        ).data.balance.amount;
        expect(parseInt(nativeBondProviderBalance, 10)).toEqual(150000);
      });
    });
  });

  it.skip('update validators set and check kv queries id', async () => {
    const {
      neutronUserAddress,
      factoryContractClient,
      validatorAddress,
      secondValidatorAddress,
    } = context;

    const queryIdsOriginal =
      await context.puppeteerContractClient.queryKVQueryIds();

    expect(queryIdsOriginal).toEqual([[1, 'delegations_and_balance']]);

    const res = await factoryContractClient.proxy(
      neutronUserAddress,
      {
        validator_set: {
          update_validators: {
            validators: [
              {
                valoper_address: validatorAddress,
                weight: 1,
                on_top: null,
              },
              {
                valoper_address: secondValidatorAddress,
                weight: 1,
                on_top: null,
              },
            ],
          },
        },
      },
      1.5,
      undefined,
      [
        {
          amount: '1000000',
          denom: 'untrn',
        },
      ],
    );
    expect(res.transactionHash).toHaveLength(64);

    const queryIdsNew = await context.puppeteerContractClient.queryKVQueryIds();
    expect(queryIdsNew).toEqual([[2, 'delegations_and_balance']]);
  });
});
