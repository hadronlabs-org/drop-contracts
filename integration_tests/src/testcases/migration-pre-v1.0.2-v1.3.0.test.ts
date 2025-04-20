import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import {
  DropCore as DeprecatedDropCore,
  DropFactory as DeprecatedDropFactory,
  DropPump as DeprecatedDropPump,
  DropPuppeteer as DeprecatedDropPuppeteer,
  DropStrategy as DeprecatedDropStrategy,
  DropWithdrawalManager as DeprecatedDropWithdrawalManager,
  DropWithdrawalVoucher as DeprecatedDropWithdrawalVoucher,
  DropRewardsManager as DeprecatedDropRewardsManager,
  DropStaker as DeprecatedDropStaker,
  DropSplitter as DeprecatedDropSplitter,
  DropToken as DeprecatedDropToken,
} from 'drop-ts-client-v1-0-1';
import {
  DropCore,
  DropFactory,
  DropPump,
  DropPuppeteer,
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
import { MsgTransfer } from 'cosmjs-types/ibc/applications/transfer/v1/tx';
import { join } from 'path';
import { Tendermint34Client } from '@cosmjs/tendermint-rpc';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import {
  AccountData,
  coins,
  DirectSecp256k1HdWallet,
} from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { awaitBlocks, awaitTargetChannels, setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import {
  UnbondBatch as DeprecatedUnbondBatch,
  ResponseHookMsg,
} from 'drop-ts-client-v1-0-1/lib/contractLib/dropCore';
import { stringToPath } from '@cosmjs/crypto';
import { waitForTx } from '../helpers/waitForTx';
import { instrumentCoreClass } from '../helpers/knot';
import { waitForPuppeteerICQ } from '../helpers/waitForPuppeteerICQ';
import { checkExchangeRate } from '../helpers/exchangeRate';

const DeprecatedDropTokenClass = DeprecatedDropToken.Client;
const DeprecatedDropFactoryClass = DeprecatedDropFactory.Client;
const DeprecatedDropCoreClass = DeprecatedDropCore.Client;
const DeprecatedDropPumpClass = DeprecatedDropPump.Client;
const DeprecatedDropStakerClass = DeprecatedDropStaker.Client;
const DeprecatedDropPuppeteerClass = DeprecatedDropPuppeteer.Client;
const DeprecatedDropStrategyClass = DeprecatedDropStrategy.Client;
const DeprecatedDropWithdrawalVoucherClass =
  DeprecatedDropWithdrawalVoucher.Client;
const DeprecatedDropWithdrawalManagerClass =
  DeprecatedDropWithdrawalManager.Client;
const DeprecatedDropRewardsManagerClass = DeprecatedDropRewardsManager.Client;
const DeprecatedDropRewardsPumpClass = DeprecatedDropPump.Client;
const DeprecatedDropSplitterClass = DeprecatedDropSplitter.Client;

const DropTokenClass = DropToken.Client;
const DropFactoryClass = DropFactory.Client;
const DropCoreClass = DropCore.Client;
const DropPumpClass = DropPump.Client;
const DropPuppeteerClass = DropPuppeteer.Client;
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
    gaiaWallet?: DirectSecp256k1HdWallet;
    gaiaWallet2?: DirectSecp256k1HdWallet;
    oldClients: {
      factoryContractClient?: InstanceType<typeof DeprecatedDropFactoryClass>;
      coreContractClient?: InstanceType<typeof DeprecatedDropCoreClass>;
      stakerContractClient?: InstanceType<typeof DeprecatedDropStakerClass>;
      strategyContractClient?: InstanceType<typeof DeprecatedDropStrategyClass>;
      pumpContractClient?: InstanceType<typeof DeprecatedDropPumpClass>;
      puppeteerContractClient?: InstanceType<
        typeof DeprecatedDropPuppeteerClass
      >;
      splitterContractClient?: InstanceType<typeof DeprecatedDropSplitterClass>;
      tokenContractClient?: InstanceType<typeof DeprecatedDropTokenClass>;
      withdrawalVoucherContractClient?: InstanceType<
        typeof DeprecatedDropWithdrawalVoucherClass
      >;
      withdrawalManagerContractClient?: InstanceType<
        typeof DeprecatedDropWithdrawalManagerClass
      >;
      rewardsManagerContractClient?: InstanceType<
        typeof DeprecatedDropRewardsManagerClass
      >;
      rewardsPumpContractClient?: InstanceType<
        typeof DeprecatedDropRewardsPumpClass
      >;
    };
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
    puppeteerIcaAddress?: string;
    stakerIcaAddress?: string;
    rewardsPumpIcaAddress?: string;
    client?: SigningCosmWasmClient;
    gaiaClient?: SigningStargateClient;
    gaiaUserAddress?: string;
    gaiaUserAddress2?: string;
    gaiaQueryClient?: QueryClient & StakingExtension & BankExtension;
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
      staker?: number;
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
    neutronIBCDenom?: string;
    ldDenom?: string;
  } = { codeIds: {}, oldClients: {} };

  beforeAll(async (t) => {
    context.park = await setupPark(
      t,
      ['neutronv2', 'gaia'],
      {
        gaia: {
          genesis_opts: {
            'app_state.staking.params.unbonding_time': `${UNBONDING_TIME}s`,
          },
        },
        neutronv2: {
          genesis_opts: {
            'app_state.staking.params.bond_denom': `untrn`,
          },
        },
      },
      {
        neutron: true,
        hermes: {
          config: {
            'chains.1.trusting_period': '2m0s',
          },
        },
      },
    );
    await awaitTargetChannels(
      `http://127.0.0.1:${context.park.ports.gaia.rpc}`,
    );
    context.wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demowallet1.mnemonic,
      {
        prefix: 'neutron',
      },
    );
    context.gaiaWallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demowallet1.mnemonic,
      {
        prefix: 'cosmos',
      },
    );
    context.gaiaWallet2 = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demo1.mnemonic,
      {
        prefix: 'cosmos',
      },
    );
    context.account = (await context.wallet.getAccounts())[0];
    context.neutronClient = new NeutronClient({
      apiURL: `http://127.0.0.1:${context.park.ports.neutronv2.rest}`,
      rpcURL: `127.0.0.1:${context.park.ports.neutronv2.rpc}`,
      prefix: 'neutron',
    });
    context.neutronRPCEndpoint = `http://127.0.0.1:${context.park.ports.neutronv2.rpc}`;
    context.client = await SigningCosmWasmClient.connectWithSigner(
      context.neutronRPCEndpoint,
      context.wallet,
      {
        gasPrice: GasPrice.fromString('0.025untrn'),
      },
    );
    context.gaiaClient = await SigningStargateClient.connectWithSigner(
      `http://127.0.0.1:${context.park.ports.gaia.rpc}`,
      context.gaiaWallet,
      {
        gasPrice: GasPrice.fromString('0.025stake'),
      },
    );
    const tmClient = await Tendermint34Client.connect(
      `http://127.0.0.1:${context.park.ports.gaia.rpc}`,
    );
    context.gaiaQueryClient = QueryClient.withExtensions(
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
  });

  afterAll(async () => {
    await context.park.stop();
  });

  it('transfer tokens to neutron', async () => {
    context.gaiaUserAddress = (
      await context.gaiaWallet.getAccounts()
    )[0].address;
    context.gaiaUserAddress2 = (
      await context.gaiaWallet2.getAccounts()
    )[0].address;
    context.neutronUserAddress = (
      await context.wallet.getAccounts()
    )[0].address;
    {
      const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
        context.park.config.master_mnemonic,
        {
          prefix: 'cosmosvaloper',
          hdPaths: [stringToPath("m/44'/118'/1'/0/0") as any],
        },
      );
      context.validatorAddress = (await wallet.getAccounts())[0].address;
    }
    {
      const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
        context.park.config.master_mnemonic,
        {
          prefix: 'cosmosvaloper',
          hdPaths: [stringToPath("m/44'/118'/2'/0/0") as any],
        },
      );
      context.secondValidatorAddress = (await wallet.getAccounts())[0].address;
    }

    const { gaiaClient, gaiaUserAddress, neutronUserAddress, neutronClient } =
      context;
    const res = await gaiaClient.signAndBroadcast(
      gaiaUserAddress,
      [
        {
          typeUrl: '/ibc.applications.transfer.v1.MsgTransfer',
          value: MsgTransfer.fromPartial({
            sender: gaiaUserAddress,
            sourceChannel: 'channel-0',
            sourcePort: 'transfer',
            receiver: neutronUserAddress,
            token: { denom: 'stake', amount: '2000000' },
            timeoutTimestamp: BigInt((Date.now() + 10 * 60 * 1000) * 1e6),
            timeoutHeight: {
              revisionHeight: BigInt(0),
              revisionNumber: BigInt(0),
            },
          }),
        },
      ],
      2,
    );
    expect(res.transactionHash).toHaveLength(64);
    await waitFor(async () => {
      const balances =
        await neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
          neutronUserAddress,
        );
      context.neutronIBCDenom = balances.data.balances.find((b) =>
        b.denom.startsWith('ibc/'),
      )?.denom;
      return balances.data.balances.length > 1;
    }, 60_000);
    expect(context.neutronIBCDenom).toBeTruthy();
  });

  it('instantiate', async () => {
    const { client, account } = context;
    context.codeIds = {};

    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../migration_data/v1.1.0/v1.0.1/drop_core.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.core = res.codeId;
    }
    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../migration_data/v1.1.0/v1.0.1/drop_token.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.token = res.codeId;
    }
    {
      const buffer = fs.readFileSync(
        join(
          __dirname,
          '../../migration_data/v1.1.0/v1.0.1/drop_withdrawal_voucher.wasm',
        ),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.withdrawalVoucher = res.codeId;
    }
    {
      const buffer = fs.readFileSync(
        join(
          __dirname,
          '../../migration_data/v1.1.0/v1.0.1/drop_withdrawal_manager.wasm',
        ),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.withdrawalManager = res.codeId;
    }
    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../migration_data/v1.1.0/v1.0.1/drop_pump.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.pump = res.codeId;
    }
    {
      const buffer = fs.readFileSync(
        join(
          __dirname,
          '../../migration_data/v1.1.0/v1.0.1/drop_strategy.wasm',
        ),
      );
      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.strategy = res.codeId;
    }
    {
      const buffer = fs.readFileSync(
        join(
          __dirname,
          '../../migration_data/v1.1.0/v1.0.1/drop_distribution.wasm',
        ),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.distribution = res.codeId;
    }
    {
      const buffer = fs.readFileSync(
        join(
          __dirname,
          '../../migration_data/v1.1.0/v1.0.1/drop_validators_set.wasm',
        ),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.validatorsSet = res.codeId;
    }
    {
      const buffer = fs.readFileSync(
        join(
          __dirname,
          '../../migration_data/v1.1.0/v1.0.1/drop_puppeteer.wasm',
        ),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.puppeteer = res.codeId;
    }
    {
      const buffer = fs.readFileSync(
        join(
          __dirname,
          '../../migration_data/v1.1.0/v1.0.1/drop_rewards_manager.wasm',
        ),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.rewardsManager = res.codeId;
    }
    {
      const buffer = fs.readFileSync(
        join(
          __dirname,
          '../../migration_data/v1.1.0/v1.0.1/drop_splitter.wasm',
        ),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.splitter = res.codeId;
    }
    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../migration_data/v1.1.0/v1.0.1/drop_staker.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.staker = res.codeId;
    }

    const buffer = fs.readFileSync(
      join(__dirname, '../../migration_data/v1.1.0/v1.0.1/drop_factory.wasm'),
    );

    const res = await client.upload(
      account.address,
      new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await DeprecatedDropFactory.Client.instantiate(
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
          staker_code_id: context.codeIds.staker,
          distribution_code_id: context.codeIds.distribution,
          validators_set_code_id: context.codeIds.validatorsSet,
          puppeteer_code_id: context.codeIds.puppeteer,
          rewards_manager_code_id: context.codeIds.rewardsManager,
          splitter_code_id: context.codeIds.splitter,
          rewards_pump_code_id: context.codeIds.pump,
        },
        remote_opts: {
          connection_id: 'connection-0',
          transfer_channel_id: 'channel-0',
          reverse_transfer_channel_id: 'channel-0',
          port_id: 'transfer',
          denom: 'stake',
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
        base_denom: context.neutronIBCDenom,
        core_params: {
          idle_min_interval: 1,
          unbond_batch_switch_time: 60,
          unbonding_safe_period: 10,
          unbonding_period: 360,
          lsm_redeem_threshold: 2,
          lsm_min_bond_amount: '1000',
          lsm_redeem_max_interval: 60_000,
          bond_limit: '0',
          min_stake_amount: '2',
          icq_update_delay: 5,
        },
        staker_params: {
          min_stake_amount: '10000',
          min_ibc_transfer: '10000',
        },
      },
      'drop-staking-factory',
      'auto',
      [],
      account.address,
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.contractAddress = instantiateRes.contractAddress;
    context.oldClients.factoryContractClient = new DeprecatedDropFactory.Client(
      client,
      context.contractAddress,
    );
  });

  it('query factory state', async () => {
    const {
      oldClients: { factoryContractClient: contractClient },
      neutronClient,
    } = context;
    const res = await contractClient.queryState();
    expect(res).toBeTruthy();
    const tokenContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.token_contract,
      );
    expect(tokenContractInfo.data.contract_info.label).toBe(
      'drop-staking-token',
    );
    const stakerContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.staker_contract,
      );
    expect(stakerContractInfo.data.contract_info.label).toBe(
      'drop-staking-staker',
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
    context.oldClients.coreContractClient = new DeprecatedDropCore.Client(
      context.client,
      res.core_contract,
    );
    context.oldClients.withdrawalVoucherContractClient =
      new DeprecatedDropWithdrawalVoucher.Client(
        context.client,
        res.withdrawal_voucher_contract,
      );
    context.oldClients.withdrawalManagerContractClient =
      new DeprecatedDropWithdrawalManager.Client(
        context.client,
        res.withdrawal_manager_contract,
      );
    context.oldClients.rewardsManagerContractClient =
      new DeprecatedDropRewardsManager.Client(
        context.client,
        res.rewards_manager_contract,
      );
    context.oldClients.strategyContractClient =
      new DeprecatedDropStrategy.Client(context.client, res.strategy_contract);
    context.oldClients.stakerContractClient = new DeprecatedDropStaker.Client(
      context.client,
      res.staker_contract,
    );
    context.oldClients.rewardsPumpContractClient =
      new DeprecatedDropPump.Client(context.client, res.rewards_pump_contract);

    context.oldClients.tokenContractClient = new DeprecatedDropToken.Client(
      context.client,
      res.token_contract,
    );
    context.oldClients.puppeteerContractClient =
      new DeprecatedDropPuppeteer.Client(
        context.client,
        res.puppeteer_contract,
      );
    context.oldClients.splitterContractClient =
      new DeprecatedDropSplitter.Client(context.client, res.splitter_contract);
  });

  it('register staker ICA', async () => {
    const {
      oldClients: { stakerContractClient },
      neutronUserAddress,
    } = context;
    const res = await stakerContractClient.registerICA(
      neutronUserAddress,
      1.5,
      undefined,
      [{ amount: '1000000', denom: 'untrn' }],
    );
    expect(res.transactionHash).toHaveLength(64);
    let ica = '';
    await waitFor(async () => {
      const res = await stakerContractClient.queryIca();
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
    expect(ica).toHaveLength(65);
    expect(ica.startsWith('cosmos')).toBeTruthy();
    context.stakerIcaAddress = ica;
  });
  it('setup ICA for rewards pump', async () => {
    const {
      oldClients: { rewardsPumpContractClient },
      neutronUserAddress,
    } = context;
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
    expect(ica).toHaveLength(65);
    expect(ica.startsWith('cosmos')).toBeTruthy();
    context.rewardsPumpIcaAddress = ica;
  });

  it('register puppeteer ICA', async () => {
    const {
      oldClients: { puppeteerContractClient },
      neutronUserAddress,
    } = context;
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
    expect(ica).toHaveLength(65);
    expect(ica.startsWith('cosmos')).toBeTruthy();
    context.puppeteerIcaAddress = ica;
  });
  it('set puppeteer ICA to the staker', async () => {
    const res = await context.oldClients.factoryContractClient.adminExecute(
      context.neutronUserAddress,
      {
        msgs: [
          {
            wasm: {
              execute: {
                contract_addr:
                  context.oldClients.stakerContractClient.contractAddress,
                msg: Buffer.from(
                  JSON.stringify({
                    update_config: {
                      new_config: {
                        puppeteer_ica: context.puppeteerIcaAddress,
                      },
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
  it('grant staker to delegate funds from puppeteer ICA and set up rewards receiver', async () => {
    const { neutronUserAddress } = context;
    const res = await context.oldClients.factoryContractClient.adminExecute(
      neutronUserAddress,
      {
        msgs: [
          {
            wasm: {
              execute: {
                contract_addr:
                  context.oldClients.puppeteerContractClient.contractAddress,
                msg: Buffer.from(
                  JSON.stringify({
                    setup_protocol: {
                      delegate_grantee: context.stakerIcaAddress,
                      rewards_withdraw_address: context.rewardsPumpIcaAddress,
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
    const pupRes =
      await context.oldClients.puppeteerContractClient.queryTxState();
    expect(pupRes.status).toBe('waiting_for_ack');
  });
  it('wait puppeteer response', async () => {
    const {
      oldClients: { puppeteerContractClient },
    } = context;
    await waitFor(async () => {
      const res = await puppeteerContractClient.queryTxState();
      return res.status === 'idle';
    }, 100_000);
  });
  it('verify grant', async () => {
    const res = await context.park.executeInNetwork(
      'gaia',
      `${context.park.config.networks['gaia'].binary} query authz grants-by-grantee ${context.stakerIcaAddress} --output json`,
    );
    const out = JSON.parse(res.out);
    expect(out.grants).toHaveLength(1);
    const grant = out.grants[0];
    expect(grant.granter).toEqual(context.puppeteerIcaAddress);
    expect(grant.grantee).toEqual(context.stakerIcaAddress);
  });

  it('deploy pump', async () => {
    const { client, contractAddress, neutronUserAddress } = context;
    const res = await DeprecatedDropPump.Client.instantiate(
      client,
      neutronUserAddress,
      context.codeIds.pump,
      {
        connection_id: 'connection-0',
        local_denom: 'untrn',
        timeout: {
          local: 60,
          remote: 60,
        },
        dest_address:
          context.oldClients.withdrawalManagerContractClient.contractAddress,
        dest_port: 'transfer',
        dest_channel: 'channel-0',
        refundee: neutronUserAddress,
        owner: contractAddress,
      },
      'drop-staking-pump',
      1.5,
      [],
      contractAddress,
    );
    expect(res.contractAddress).toHaveLength(66);
    context.oldClients.pumpContractClient = new DeprecatedDropPump.Client(
      client,
      res.contractAddress,
    );
    await context.oldClients.pumpContractClient.registerICA(
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
      const res = await context.oldClients.pumpContractClient.queryIca();
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
    expect(ica).toHaveLength(65);
    expect(ica.startsWith('cosmos')).toBeTruthy();
    const resFactory =
      await context.oldClients.factoryContractClient.updateConfig(
        neutronUserAddress,
        {
          core: {
            pump_ica_address: ica,
          },
        },
      );
    expect(resFactory.transactionHash).toHaveLength(64);
  });

  it('query exchange rate', async () => {
    const {
      oldClients: { coreContractClient },
    } = context;
    context.exchangeRate = parseFloat(
      await coreContractClient.queryExchangeRate(),
    );
    expect(context.exchangeRate).toEqual(1);
  });

  it('bond w/o receiver', async () => {
    const {
      oldClients: { coreContractClient },
      neutronClient,
      neutronUserAddress,
      neutronIBCDenom,
    } = context;
    const res = await coreContractClient.bond(
      neutronUserAddress,
      {},
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
    await awaitBlocks(`http://127.0.0.1:${context.park.ports.gaia.rpc}`, 1);
    const balances =
      await neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
        neutronUserAddress,
      );

    const ldBalance = balances.data.balances.find((one) =>
      one.denom.startsWith('factory'),
    );

    expect(ldBalance).toEqual({
      denom: `factory/${context.oldClients.tokenContractClient.contractAddress}/drop`,
      amount: String(Math.floor(500_000 / context.exchangeRate)),
    });

    context.ldDenom = ldBalance?.denom;
  });
  it('verify bonded amount', async () => {
    const {
      oldClients: { coreContractClient },
    } = context;
    const bonded = await coreContractClient.queryTotalBonded();
    expect(bonded).toEqual('500000');
  });

  it('delegate tokens on gaia side', async () => {
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.master_mnemonic,
      {
        prefix: 'cosmosvaloper',
        hdPaths: [stringToPath("m/44'/118'/1'/0/0") as any],
      },
    );
    context.validatorAddress = (await wallet.getAccounts())[0].address;
    const res = await context.park.executeInNetwork(
      'gaia',
      `gaiad tx staking delegate ${context.validatorAddress} 1000000stake --from ${context.gaiaUserAddress} --yes --chain-id testgaia --home=/opt --keyring-backend=test --output json --fees 1000stake`,
    );
    expect(res.exitCode).toBe(0);
    const out = JSON.parse(res.out);
    expect(out.code).toBe(0);
    expect(out.txhash).toHaveLength(64);
    await waitForTx(context.gaiaClient, out.txhash);
  });
  it('tokenize share on gaia side', async () => {
    const res = await context.park.executeInNetwork(
      'gaia',
      `gaiad tx staking tokenize-share ${context.validatorAddress} 600000stake ${context.gaiaUserAddress} --from ${context.gaiaUserAddress} --yes --chain-id testgaia --home=/opt --keyring-backend=test --gas auto --gas-adjustment 2 --output json  --fees 1000stake`,
    );
    expect(res.exitCode).toBe(0);
    const out = JSON.parse(res.out);
    expect(out.code).toBe(0);
    expect(out.txhash).toHaveLength(64);
    await waitForTx(context.gaiaClient, out.txhash);
    const balances = await context.gaiaQueryClient.bank.allBalances(
      context.gaiaUserAddress,
    );
    expect(
      balances.find((a) => a.denom == `${context.validatorAddress}/1`),
    ).toEqual({
      denom: `${context.validatorAddress}/1`,
      amount: '600000',
    });
  });
  it('transfer tokenized share to neutron', async () => {
    const res = await context.park.executeInNetwork(
      'gaia',
      `gaiad tx ibc-transfer transfer transfer channel-0 ${context.neutronUserAddress} 600000${context.validatorAddress}/1 --from ${context.gaiaUserAddress}  --yes --chain-id testgaia --home=/opt --keyring-backend=test --gas auto --gas-adjustment 2 --output json  --fees 1000stake`,
    );
    expect(res.exitCode).toBe(0);
    const out = JSON.parse(res.out);
    expect(out.code).toBe(0);
    expect(out.txhash).toHaveLength(64);
    await waitForTx(context.gaiaClient, out.txhash);
  });
  it('wait for neutron to receive tokenized share', async () => {
    const { neutronClient, neutronUserAddress } = context;
    let balances;
    await waitFor(async () => {
      balances =
        await neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
          neutronUserAddress,
        );
      return balances.data.balances.some((b) => b.amount === '600000');
    });
    const shareOnNeutron = balances.data.balances.find(
      (b) => b.amount === '600000',
    );
    expect(shareOnNeutron).toBeDefined();
    expect(shareOnNeutron?.amount).toBe('600000');
    context.tokenizedDenomOnNeutron = shareOnNeutron?.denom;
  });
  it('add validators into validators set', async () => {
    const {
      neutronUserAddress,
      oldClients: { factoryContractClient },
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
              },
              {
                valoper_address: secondValidatorAddress,
                weight: 1,
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
  });

  it('unbond', async () => {
    const {
      oldClients: { coreContractClient },
      neutronUserAddress,
      ldDenom,
    } = context;
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
  });

  it('validate unbonding batch', async () => {
    const batch = await context.oldClients.coreContractClient.queryUnbondBatch({
      batch_id: '0',
    });
    expect(batch).toBeTruthy();
    expect(batch).toEqual<DeprecatedUnbondBatch>({
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

  it('check core contract settings', async () => {
    const {
      oldClients: { coreContractClient },
    } = context;
    const config = await coreContractClient.queryConfig();
    expect(config.lsm_redeem_threshold).toBe(2);
  });

  it('check staker contract settings', async () => {
    const {
      oldClients: { stakerContractClient },
    } = context;
    const config = await stakerContractClient.queryConfig();
    expect((config as any).min_ibc_transfer).toBe('10000');
    expect((config as any).min_staking_amount).toBe('10000');
  });

  it('check splitter contract settings', async () => {
    const {
      oldClients: { splitterContractClient, stakerContractClient },
      neutronIBCDenom,
    } = context;
    const config = await splitterContractClient.queryConfig();
    expect(config).toMatchObject({
      denom: neutronIBCDenom,
      receivers: [[stakerContractClient.contractAddress, '10000']],
    });
  });

  describe('state machine', () => {
    const ica: { balance?: number } = {};
    describe('prepare', () => {
      it('get ICA balance', async () => {
        const { gaiaClient } = context;
        const res = await gaiaClient.getBalance(
          context.puppeteerIcaAddress,
          context.park.config.networks.gaia.denom,
        );
        ica.balance = parseInt(res.amount);
        expect(ica.balance).toEqual(0);
      });
      it('get machine state', async () => {
        const state =
          await context.oldClients.coreContractClient.queryContractState();
        expect(state).toEqual('idle');
      });
    });
    describe('first cycle', () => {
      it('first tick did nothing and stays in idle', async () => {
        const {
          gaiaClient,
          neutronUserAddress,
          oldClients: { coreContractClient, puppeteerContractClient },
        } = context;

        const kvKeys = await puppeteerContractClient.queryKVQueryIds();
        console.log('kvKeys', kvKeys);

        await waitForPuppeteerICQ(
          gaiaClient,
          coreContractClient,
          puppeteerContractClient,
        );

        const res = await coreContractClient.tick(
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

        const state = await coreContractClient.queryContractState();
        expect(state).toEqual('idle');
      });
      it('tick', async () => {
        const {
          gaiaClient,
          neutronUserAddress,
          oldClients: { coreContractClient, puppeteerContractClient },
        } = context;

        await waitForPuppeteerICQ(
          gaiaClient,
          coreContractClient,
          puppeteerContractClient,
        );

        const res = await coreContractClient.tick(
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
        const state = await coreContractClient.queryContractState();
        expect(state).toEqual('idle');
      });
      // it('wait for the response from puppeteer', async () => {
      //   let response: ResponseHookMsg;
      //   await waitFor(async () => {
      //     try {
      //       response = (
      //         await context.oldClients.coreContractClient.queryLastPuppeteerResponse()
      //       ).response;
      //     } catch (e) {
      //       //
      //     }
      //     return !!response;
      //   }, 100_000);
      //   expect(response).toBeTruthy();
      //   expect<ResponseHookMsg>(response).toHaveProperty('success');
      // });
      it('next tick should go to idle', async () => {
        const {
          gaiaClient,
          neutronUserAddress,
          oldClients: { coreContractClient, puppeteerContractClient },
        } = context;

        await waitForPuppeteerICQ(
          gaiaClient,
          coreContractClient,
          puppeteerContractClient,
        );

        const res = await coreContractClient.tick(
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

        const state = await coreContractClient.queryContractState();
        expect(state).toEqual('idle');
      });
    });
  });

  it('upload intermediate code and run migration', async () => {
    const { client, account, contractAddress } = context;

    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../migration_data/v1.1.0/v1.0.2/drop_core.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.core = res.codeId;
    }

    const buffer = fs.readFileSync(
      join(__dirname, '../../migration_data/v1.1.0/v1.0.2/drop_factory.wasm'),
    );

    const res = await client.upload(
      account.address,
      new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);

    const fee = {
      amount: coins(5000, 'untrn'),
      gas: '2000000',
    };

    const result = await client.migrate(
      account.address,
      contractAddress,
      res.codeId,
      {
        core_code_id: context.codeIds.core,
      },
      fee,
    );

    expect(result.transactionHash).toHaveLength(64);

    await awaitBlocks(
      `http://127.0.0.1:${context.park.ports.neutronv2.rpc}`,
      3,
    );
  });

  it('check core contract settings', async () => {
    const {
      oldClients: { coreContractClient },
    } = context;
    const config = await coreContractClient.queryConfig();
    expect(config.lsm_redeem_threshold).toBe(1);
  });

  it('check staker contract settings', async () => {
    const {
      oldClients: { stakerContractClient },
    } = context;
    const config = await stakerContractClient.queryConfig();
    expect((config as any).min_ibc_transfer).toBe('1');
    expect((config as any).min_staking_amount).toBe('1');
  });

  it('check splitter contract settings', async () => {
    const {
      oldClients: { splitterContractClient },
    } = context;
    const config = await splitterContractClient.queryConfig();
    expect(config).toMatchObject({
      denom:
        'ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9',
      receivers: [],
    });
  });

  it('bond failed temporarily disabled', async () => {
    const {
      oldClients: { coreContractClient },
      neutronUserAddress,
      neutronIBCDenom,
    } = context;
    await expect(
      coreContractClient.bond(neutronUserAddress, {}, 1.6, undefined, [
        {
          amount: '500000',
          denom: neutronIBCDenom,
        },
      ]),
    ).rejects.toThrowError(
      /Bonding is temporarily disabled due to protocol upgrade/,
    );
  });

  it('unbond failed temporarily disabled', async () => {
    const {
      oldClients: { coreContractClient },
      neutronUserAddress,
    } = context;
    await expect(
      coreContractClient.unbond(neutronUserAddress, 1.6, undefined, [
        {
          amount: '1000',
          denom: 'untrn',
        },
      ]),
    ).rejects.toThrowError(
      /Unbonding is temporarily disabled due to protocol upgrade/,
    );
  });

  it('upload final code and run migrations', async () => {
    const { client, account, contractAddress } = context;

    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../../artifacts/drop_core.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.core = res.codeId;
    }

    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../../artifacts/drop_distribution.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.distribution = res.codeId;
    }

    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../../artifacts/drop_lsm_share_bond_provider.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.lsmShareBondProvider = res.codeId;
    }

    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../../artifacts/drop_native_bond_provider.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.nativeBondProvider = res.codeId;
    }

    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../../artifacts/drop_pump.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.pump = res.codeId;
    }

    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../../artifacts/drop_puppeteer.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.puppeteer = res.codeId;
    }

    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../../artifacts/drop_strategy.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.strategy = res.codeId;
    }

    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../../artifacts/drop_validators_set.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.validatorsSet = res.codeId;
    }

    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../../artifacts/drop_rewards_manager.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.rewardsManager = res.codeId;
    }

    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../../artifacts/drop_withdrawal_manager.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.withdrawalManager = res.codeId;
    }

    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../../artifacts/drop_token.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.token = res.codeId;
    }

    const buffer = fs.readFileSync(
      join(__dirname, '../../../artifacts/drop_factory.wasm'),
    );

    const res = await client.upload(
      account.address,
      new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);

    const fee = {
      amount: coins(15000, 'untrn'),
      gas: '6000000',
    };

    const result = await client.migrate(
      account.address,
      contractAddress,
      res.codeId,
      {
        core_code_id: context.codeIds.core,
        lsm_share_bond_provider_code_id: context.codeIds.lsmShareBondProvider,
        native_bond_provider_code_id: context.codeIds.nativeBondProvider,
        distribution_code_id: context.codeIds.distribution,
        pump_code_id: context.codeIds.pump,
        unbonding_pump_contract:
          context.oldClients.pumpContractClient.contractAddress,
        puppeteer_code_id: context.codeIds.puppeteer,
        validators_set_code_id: context.codeIds.validatorsSet,
        strategy_code_id: context.codeIds.strategy,
        rewards_manager_code_id: context.codeIds.rewardsManager,
        withdrawal_manager_code_id: context.codeIds.withdrawalManager,
        token_code_id: context.codeIds.token,
        salt: 'salt',
        port_id: 'port',
        timeout: 100,
        min_ibc_transfer: '100',
      },
      fee,
    );

    expect(result.transactionHash).toHaveLength(64);

    await awaitBlocks(
      `http://127.0.0.1:${context.park.ports.neutronv2.rpc}`,
      3,
    );
  });

  it('register new clients', () => {
    const { client } = context;
    context.factoryContractClient = new DropFactory.Client(
      client,
      context.contractAddress,
    );
  });

  it('query updated factory state', async () => {
    const { factoryContractClient: contractClient, neutronClient } = context;
    const res = await contractClient.queryState();
    expect(res).toBeTruthy();

    expect(res).not.toHaveProperty('staker_contract');

    const lsmShareBondProviderContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.lsm_share_bond_provider_contract,
      );
    expect(lsmShareBondProviderContractInfo.data.contract_info.label).toBe(
      'drop-staking-lsm_share_bond_provider',
    );
    const nativeBondProviderContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.native_bond_provider_contract,
      );
    expect(nativeBondProviderContractInfo.data.contract_info.label).toBe(
      'drop-staking-native_bond_provider',
    );

    context.coreContractClient = instrumentCoreClass(
      new DropCore.Client(context.client, res.core_contract),
    );
    context.puppeteerContractClient = new DropPuppeteer.Client(
      context.client,
      res.puppeteer_contract,
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
    context.pumpContractClient = new DropPump.Client(
      context.client,
      res.unbonding_pump_contract,
    );

    context.rewardsPumpContractClient = new DropPump.Client(
      context.client,
      res.rewards_pump_contract,
    );
  });

  it('check core bond providers settings', async () => {
    const { coreContractClient } = context;
    const bondProviders = await coreContractClient.queryBondProviders();

    expect(bondProviders.length).toEqual(2);

    expect(bondProviders.sort()).toEqual(
      [
        context.lsmShareBondProviderContractClient.contractAddress,
        context.nativeBondProviderContractClient.contractAddress,
      ].sort(),
    );
  });

  it('check splitter contract settings', async () => {
    const {
      oldClients: { splitterContractClient },
    } = context;
    const config = await splitterContractClient.queryConfig();
    expect(config).toMatchObject({
      denom:
        'ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9',
      receivers: [
        [context.nativeBondProviderContractClient.contractAddress, '9000'],
        [
          'neutron1xm4xgfv4xz4ccv0tjvlfac5gqwjnv9zzx4l47t7ve7j2sn4k7gwqkg947d',
          '1000',
        ],
      ],
    });
  });

  it('check puppeteer contract allowed senders', async () => {
    const {
      puppeteerContractClient,
      lsmShareBondProviderContractClient,
      nativeBondProviderContractClient,
      coreContractClient,
      factoryContractClient,
    } = context;

    const config = await puppeteerContractClient.queryConfig();

    expect(config['allowed_senders'].sort()).toEqual(
      [
        lsmShareBondProviderContractClient.contractAddress,
        nativeBondProviderContractClient.contractAddress,
        coreContractClient.contractAddress,
        factoryContractClient.contractAddress,
      ].sort(),
    );
  });

  it('check unbonding pump contract config', async () => {
    const {
      pumpContractClient,
      neutronUserAddress,
      oldClients: { withdrawalManagerContractClient },
    } = context;

    console.log('Unbonding pump contract config');

    const config = await pumpContractClient.queryConfig();

    console.log(config);

    expect(config).toMatchObject({
      dest_address: withdrawalManagerContractClient.contractAddress,
      dest_channel: 'channel-0',
      dest_port: 'transfer',
      connection_id: 'connection-0',
      refundee: neutronUserAddress,
      timeout: { local: 60, remote: 60 },
      local_denom: 'untrn',
    });
  });

  it('check rewards pump contract config', async () => {
    const {
      rewardsPumpContractClient,
      oldClients: { splitterContractClient },
    } = context;

    console.log('Rewards pump contract config');

    const config = await rewardsPumpContractClient.queryConfig();

    console.log(config);

    expect(config).toMatchObject({
      dest_address: splitterContractClient.contractAddress,
      dest_channel: 'channel-0',
      dest_port: 'transfer',
      connection_id: 'connection-0',
      refundee: null,
      timeout: { local: 60, remote: 60 },
      local_denom: 'untrn',
    });
  });
});
