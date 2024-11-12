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
  ResponseHookMsg as DeprecatedResponseHookMsg,
  UnbondBatch as DeprecatedUnbondBatch,
} from 'drop-ts-client-v1-0-1/lib/contractLib/dropCore';
import { stringToPath } from '@cosmjs/crypto';
import { sleep } from '../helpers/sleep';
import { waitForTx } from '../helpers/waitForTx';
import { waitForPuppeteerICQ } from '../helpers/waitForPuppeteerICQ';
import { instrumentCoreClass } from '../helpers/knot';

const DropTokenClass = DeprecatedDropToken.Client;
const DropFactoryClass = DeprecatedDropFactory.Client;
const DropCoreClass = DeprecatedDropCore.Client;
const DropPumpClass = DeprecatedDropPump.Client;
const DropStakerClass = DeprecatedDropStaker.Client;
const DropPuppeteerClass = DeprecatedDropPuppeteer.Client;
const DropStrategyClass = DeprecatedDropStrategy.Client;
const DropWithdrawalVoucherClass = DeprecatedDropWithdrawalVoucher.Client;
const DropWithdrawalManagerClass = DeprecatedDropWithdrawalManager.Client;
const DropRewardsManagerClass = DeprecatedDropRewardsManager.Client;
const DropRewardsPumpClass = DeprecatedDropPump.Client;
const DropSplitterClass = DeprecatedDropSplitter.Client;

const UNBONDING_TIME = 360;

describe('Core', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    gaiaWallet2?: DirectSecp256k1HdWallet;
    factoryContractClient?: InstanceType<typeof DropFactoryClass>;
    coreContractClient?: InstanceType<typeof DropCoreClass>;
    stakerContractClient?: InstanceType<typeof DropStakerClass>;
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
    account?: AccountData;
    icaAddress?: string;
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
  } = { codeIds: {} };

  beforeAll(async (t) => {
    context.park = await setupPark(
      t,
      ['neutron', 'gaia'],
      {
        gaia: {
          genesis_opts: {
            'app_state.staking.params.unbonding_time': `${UNBONDING_TIME}s`,
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
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../migration_data/v1.1.0/v1.0.1/drop_core.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.core = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../migration_data/v1.1.0/v1.0.1/drop_token.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.token = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(
            __dirname,
            '../../migration_data/v1.1.0/v1.0.1/drop_withdrawal_voucher.wasm',
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
        fs.readFileSync(
          join(
            __dirname,
            '../../migration_data/v1.1.0/v1.0.1/drop_withdrawal_manager.wasm',
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
        fs.readFileSync(
          join(__dirname, '../../migration_data/v1.1.0/v1.0.1/drop_pump.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.pump = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(
            __dirname,
            '../../migration_data/v1.1.0/v1.0.1/drop_strategy.wasm',
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
        fs.readFileSync(
          join(
            __dirname,
            '../../migration_data/v1.1.0/v1.0.1/drop_distribution.wasm',
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
        fs.readFileSync(
          join(
            __dirname,
            '../../migration_data/v1.1.0/v1.0.1/drop_validators_set.wasm',
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
        fs.readFileSync(
          join(
            __dirname,
            '../../migration_data/v1.1.0/v1.0.1/drop_puppeteer.wasm',
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
        fs.readFileSync(
          join(
            __dirname,
            '../../migration_data/v1.1.0/v1.0.1/drop_rewards_manager.wasm',
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
        fs.readFileSync(
          join(
            __dirname,
            '../../migration_data/v1.1.0/v1.0.1/drop_splitter.wasm',
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
        fs.readFileSync(
          join(
            __dirname,
            '../../migration_data/v1.1.0/v1.0.1/drop_staker.wasm',
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.staker = res.codeId;
    }

    const res = await client.upload(
      account.address,
      fs.readFileSync(
        join(__dirname, '../../migration_data/v1.1.0/v1.0.1/drop_factory.wasm'),
      ),
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
          idle_min_interval: 40,
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
    context.factoryContractClient = new DeprecatedDropFactory.Client(
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
    context.coreContractClient = instrumentCoreClass(
      new DeprecatedDropCore.Client(context.client, res.core_contract),
    );
    context.withdrawalVoucherContractClient =
      new DeprecatedDropWithdrawalVoucher.Client(
        context.client,
        res.withdrawal_voucher_contract,
      );
    context.withdrawalManagerContractClient =
      new DeprecatedDropWithdrawalManager.Client(
        context.client,
        res.withdrawal_manager_contract,
      );
    context.rewardsManagerContractClient =
      new DeprecatedDropRewardsManager.Client(
        context.client,
        res.rewards_manager_contract,
      );
    context.strategyContractClient = new DeprecatedDropStrategy.Client(
      context.client,
      res.strategy_contract,
    );
    context.stakerContractClient = new DeprecatedDropStaker.Client(
      context.client,
      res.staker_contract,
    );
    context.rewardsPumpContractClient = new DeprecatedDropPump.Client(
      context.client,
      res.rewards_pump_contract,
    );
    context.tokenContractClient = new DeprecatedDropToken.Client(
      context.client,
      res.token_contract,
    );
    context.puppeteerContractClient = new DeprecatedDropPuppeteer.Client(
      context.client,
      res.puppeteer_contract,
    );
    context.splitterContractClient = new DeprecatedDropSplitter.Client(
      context.client,
      res.splitter_contract,
    );
  });

  it('register staker ICA', async () => {
    const { stakerContractClient, neutronUserAddress } = context;
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
    expect(ica).toHaveLength(65);
    expect(ica.startsWith('cosmos')).toBeTruthy();
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
    expect(ica).toHaveLength(65);
    expect(ica.startsWith('cosmos')).toBeTruthy();
    context.icaAddress = ica;
  });
  it('set puppeteer ICA to the staker', async () => {
    const res = await context.factoryContractClient.adminExecute(
      context.neutronUserAddress,
      {
        msgs: [
          {
            wasm: {
              execute: {
                contract_addr: context.stakerContractClient.contractAddress,
                msg: Buffer.from(
                  JSON.stringify({
                    update_config: {
                      new_config: {
                        puppeteer_ica: context.icaAddress,
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
  it('verify grant', async () => {
    const res = await context.park.executeInNetwork(
      'gaia',
      `${context.park.config.networks['gaia'].binary} query authz grants-by-grantee ${context.stakerIcaAddress} --output json`,
    );
    const out = JSON.parse(res.out);
    expect(out.grants).toHaveLength(1);
    const grant = out.grants[0];
    expect(grant.granter).toEqual(context.icaAddress);
    expect(grant.grantee).toEqual(context.stakerIcaAddress);
  });

  it('query exchange rate', async () => {
    const { coreContractClient } = context;
    context.exchangeRate = parseFloat(
      await coreContractClient.queryExchangeRate(),
    );
    expect(context.exchangeRate).toEqual(1);
  });

  it('bond w/o receiver', async () => {
    const {
      coreContractClient,
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
      denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
      amount: String(Math.floor(500_000 / context.exchangeRate)),
    });

    context.ldDenom = ldBalance?.denom;
  });
  it('verify bonded amount', async () => {
    const { coreContractClient } = context;
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
      `gaiad tx staking delegate ${context.validatorAddress} 1000000stake --from ${context.gaiaUserAddress} --yes --chain-id testgaia --home=/opt --keyring-backend=test --output json`,
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
      `gaiad tx staking tokenize-share ${context.validatorAddress} 600000stake ${context.gaiaUserAddress} --from ${context.gaiaUserAddress} --yes --chain-id testgaia --home=/opt --keyring-backend=test --gas auto --gas-adjustment 2 --output json`,
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
      `gaiad tx ibc-transfer transfer transfer channel-0 ${context.neutronUserAddress} 600000${context.validatorAddress}/1 --from ${context.gaiaUserAddress}  --yes --chain-id testgaia --home=/opt --keyring-backend=test --gas auto --gas-adjustment 2 --output json`,
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
  });

  it('validate unbonding batch', async () => {
    const batch = await context.coreContractClient.queryUnbondBatch({
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
    const { coreContractClient } = context;
    const config = await coreContractClient.queryConfig();
    expect(config.lsm_redeem_threshold).toBe(2);
  });

  it('check staker contract settings', async () => {
    const { stakerContractClient } = context;
    const config = await stakerContractClient.queryConfig();
    expect((config as any).min_ibc_transfer).toBe('10000');
    expect((config as any).min_staking_amount).toBe('10000');
  });

  it('check splitter contract settings', async () => {
    const { splitterContractClient, stakerContractClient, neutronIBCDenom } =
      context;
    const config = await splitterContractClient.queryConfig();
    expect(config).toMatchObject({
      denom: neutronIBCDenom,
      receivers: [[stakerContractClient.contractAddress, '10000']],
    });
  });

  it('upload intermediate code and run migration', async () => {
    const { client, account, contractAddress } = context;

    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../migration_data/v1.1.0/v1.0.2/drop_core.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.core = res.codeId;
    }

    const res = await client.upload(
      account.address,
      fs.readFileSync(
        join(__dirname, '../../migration_data/v1.1.0/v1.0.2/drop_factory.wasm'),
      ),
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

    await awaitBlocks(`http://127.0.0.1:${context.park.ports.neutron.rpc}`, 3);
  });

  it('check core contract settings', async () => {
    const { coreContractClient } = context;
    const config = await coreContractClient.queryConfig();
    expect(config.lsm_redeem_threshold).toBe(1);
  });

  it('check staker contract settings', async () => {
    const { stakerContractClient } = context;
    const config = await stakerContractClient.queryConfig();
    expect((config as any).min_ibc_transfer).toBe('1');
    expect((config as any).min_staking_amount).toBe('1');
  });

  it('check splitter contract settings', async () => {
    const { splitterContractClient } = context;
    const config = await splitterContractClient.queryConfig();
    expect(config).toMatchObject({
      denom:
        'ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9',
      receivers: [],
    });
  });

  it('bond failed temporarily disabled', async () => {
    const { coreContractClient, neutronUserAddress, neutronIBCDenom } = context;
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
    const { coreContractClient, neutronUserAddress } = context;
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
      const res = await client.upload(
        account.address,
        fs.readFileSync(join(__dirname, '../../../artifacts/drop_core.wasm')),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.core = res.codeId;
    }

    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_distribution.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.distribution = res.codeId;
    }

    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(
            __dirname,
            '../../../artifacts/drop_lsm_share_bond_provider.wasm',
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
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_native_bond_provider.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.nativeBondProvider = res.codeId;
    }

    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(join(__dirname, '../../../artifacts/drop_pump.wasm')),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.pump = res.codeId;
    }

    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_puppeteer.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.puppeteer = res.codeId;
    }

    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_strategy.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.strategy = res.codeId;
    }

    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_validators_set.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.validatorsSet = res.codeId;
    }

    const res = await client.upload(
      account.address,
      fs.readFileSync(join(__dirname, '../../../artifacts/drop_factory.wasm')),
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

    await awaitBlocks(`http://127.0.0.1:${context.park.ports.neutron.rpc}`, 3);
  });

  describe.skip('state machine', () => {
    const ica: { balance?: number } = {};
    describe('prepare', () => {
      it('get ICA balance', async () => {
        const { gaiaClient } = context;
        const res = await gaiaClient.getBalance(context.icaAddress, 'stake');
        ica.balance = parseInt(res.amount);
        expect(ica.balance).toEqual(0);
      });
      it('deploy pump', async () => {
        const { client, account, neutronUserAddress } = context;
        const resUpload = await client.upload(
          account.address,
          fs.readFileSync(join(__dirname, '../../../artifacts/drop_pump.wasm')),
          1.5,
        );
        expect(resUpload.codeId).toBeGreaterThan(0);
        const { codeId } = resUpload;
        const res = await DeprecatedDropPump.Client.instantiate(
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
        context.pumpContractClient = new DeprecatedDropPump.Client(
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
        expect(ica).toHaveLength(65);
        expect(ica.startsWith('cosmos')).toBeTruthy();
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
      it('staker ibc transfer', async () => {
        const { neutronUserAddress } = context;
        const res = await context.stakerContractClient.iBCTransfer(
          neutronUserAddress,
          1.5,
          undefined,
          [{ amount: '20000', denom: 'untrn' }],
        );
        expect(res.transactionHash).toHaveLength(64);
        await waitFor(async () => {
          const res = await context.stakerContractClient.queryTxState();
          return res.status === 'idle';
        }, 80_000);
        const balances = await context.gaiaClient.getAllBalances(
          context.stakerIcaAddress,
        );
        expect(balances).toEqual([
          {
            amount: '1000000',
            denom: context.park.config.networks.gaia.denom,
          },
        ]);
      });
      it('tick', async () => {
        const {
          gaiaClient,
          neutronUserAddress,
          coreContractClient,
          puppeteerContractClient,
        } = context;

        await waitForPuppeteerICQ(
          gaiaClient,
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
        expect(state).toEqual('staking_bond');
        const stakerState = await context.stakerContractClient.queryTxState();
        expect(stakerState).toEqual({
          reply_to: context.coreContractClient.contractAddress,
          status: 'waiting_for_ack',
          seq_id: 1,
          transaction: {
            stake: {
              amount: '1000000',
            },
          },
        });
      });
      it('second tick is failed bc no response from puppeteer yet', async () => {
        const { neutronUserAddress } = context;

        await expect(
          context.coreContractClient.tick(
            neutronUserAddress,
            1.5,
            undefined,
            [],
          ),
        ).rejects.toThrowError(/Staker response is not received/);
      });
      it('state of fsm is staking_bond', async () => {
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('staking_bond');
      });
      it('wait for staker to get into idle state', async () => {
        let response;
        await waitFor(async () => {
          try {
            response = await context.stakerContractClient.queryTxState();
          } catch (e) {
            //
          }
          return response.status === 'idle';
        }, 100_000);
      });
      it('get staker ICA zeroed balance', async () => {
        const { gaiaClient } = context;
        const res = await gaiaClient.getBalance(
          context.stakerIcaAddress,
          context.park.config.networks.gaia.denom,
        );
        const balance = parseInt(res.amount);
        expect(balance).toEqual(0);
      });
      it('wait delegations', async () => {
        await waitFor(async () => {
          const res: any = await context.puppeteerContractClient.queryExtension(
            {
              msg: {
                delegations: {},
              },
            },
          );
          return res && res.delegations.delegations.length > 0;
        }, 100_000);
      });
      it('tick goes to unbonding', async () => {
        const {
          neutronUserAddress,
          gaiaClient,
          coreContractClient,
          puppeteerContractClient,
        } = context;

        await waitForPuppeteerICQ(
          gaiaClient,
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
        expect(state).toEqual('unbonding');
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
        expect(batch).toEqual<DeprecatedUnbondBatch>({
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
        expect(firstBatch).toEqual<DeprecatedUnbondBatch>({
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
        expect(secondBatch).toEqual<DeprecatedUnbondBatch>({
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
        expect(firstBatch).toEqual<DeprecatedUnbondBatch>({
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
        expect(secondBatch).toEqual<DeprecatedUnbondBatch>({
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
          gaiaClient,
          coreContractClient,
          puppeteerContractClient,
        } = context;

        await waitForPuppeteerICQ(
          gaiaClient,
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
      it('verify that unbonding batch is in unbonding state', async () => {
        const batch = await context.coreContractClient.queryUnbondBatch({
          batch_id: '0',
        });
        expect(batch).toBeTruthy();
        expect(batch).toEqual<DeprecatedUnbondBatch>({
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
      const balance = 0;
      it('idle tick', async () => {
        const {
          neutronUserAddress,
          gaiaClient,
          coreContractClient,
          puppeteerContractClient,
        } = context;

        await waitForPuppeteerICQ(
          gaiaClient,
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
      it('get rewards pump ICA balance', async () => {
        const { gaiaClient } = context;
        const res = await gaiaClient.getBalance(
          context.rewardsPumpIcaAddress,
          'stake',
        );
        const newBalance = parseInt(res.amount);
        expect(newBalance).toBeGreaterThan(balance);
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
          gaiaClient,
          coreContractClient,
          puppeteerContractClient,
        } = context;

        await waitForPuppeteerICQ(
          gaiaClient,
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
    });
    describe('third cycle (LSM-shares)', () => {
      let lsmDenoms: string[] = [];
      let oldBalanceDenoms: string[] = [];
      describe('prepare', () => {
        describe('create LSM shares and send them to neutron', () => {
          it('get balances', async () => {
            const oldBalances =
              await context.neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
                context.neutronUserAddress,
              );
            oldBalanceDenoms = oldBalances.data.balances.map((b) => b.denom);
          });
          it('update idle interval', async () => {
            const { factoryContractClient, neutronUserAddress } = context;
            const res = await factoryContractClient.updateConfig(
              neutronUserAddress,
              {
                core: {
                  idle_min_interval: 10000,
                },
              },
            );
            expect(res.transactionHash).toHaveLength(64);
          });
          it('delegate', async () => {
            {
              const res = await context.park.executeInNetwork(
                'gaia',
                `gaiad tx staking delegate ${context.validatorAddress} 100000stake --from ${context.gaiaUserAddress} --yes --chain-id testgaia --home=/opt --keyring-backend=test --output json`,
              );
              expect(res.exitCode).toBe(0);
              const out = JSON.parse(res.out);
              expect(out.code).toBe(0);
              expect(out.txhash).toHaveLength(64);
              await waitForTx(context.gaiaClient, out.txhash);
            }
            {
              const res = await context.park.executeInNetwork(
                'gaia',
                `gaiad tx staking delegate ${context.secondValidatorAddress} 100000stake --from ${context.gaiaUserAddress} --yes --chain-id testgaia --home=/opt --keyring-backend=test --output json`,
              );
              expect(res.exitCode).toBe(0);
              const out = JSON.parse(res.out);
              expect(out.code).toBe(0);
              expect(out.txhash).toHaveLength(64);
              await waitForTx(context.gaiaClient, out.txhash);
            }
          });
          it('tokenize shares', async () => {
            {
              const res = await context.park.executeInNetwork(
                'gaia',
                `gaiad tx staking tokenize-share ${context.validatorAddress} 60000stake ${context.gaiaUserAddress} --from ${context.gaiaUserAddress} --yes --chain-id testgaia --home=/opt --keyring-backend=test --gas auto --gas-adjustment 2 --output json`,
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
                balances.find(
                  (a) => a.denom == `${context.validatorAddress}/2`,
                ),
              ).toEqual({
                denom: `${context.validatorAddress}/2`,
                amount: '60000',
              });
            }
            {
              const res = await context.park.executeInNetwork(
                'gaia',
                `gaiad tx staking tokenize-share ${context.secondValidatorAddress} 60000stake ${context.gaiaUserAddress} --from ${context.gaiaUserAddress} --yes --chain-id testgaia --home=/opt --keyring-backend=test --gas auto --gas-adjustment 2 --output json`,
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
                balances.find(
                  (a) => a.denom == `${context.secondValidatorAddress}/3`,
                ),
              ).toEqual({
                denom: `${context.secondValidatorAddress}/3`,
                amount: '60000',
              });
            }
          });
          it('transfer shares to neutron', async () => {
            {
              const res = await context.park.executeInNetwork(
                'gaia',
                `gaiad tx ibc-transfer transfer transfer channel-0 ${context.neutronUserAddress} 60000${context.validatorAddress}/2 --from ${context.gaiaUserAddress}  --yes --chain-id testgaia --home=/opt --keyring-backend=test --gas auto --gas-adjustment 2 --output json`,
              );
              expect(res.exitCode).toBe(0);
              const out = JSON.parse(res.out);
              expect(out.code).toBe(0);
              expect(out.txhash).toHaveLength(64);
              await waitForTx(context.gaiaClient, out.txhash);
            }
            await sleep(10_000);
            {
              const res = await context.park.executeInNetwork(
                'gaia',
                `gaiad tx ibc-transfer transfer transfer channel-0 ${context.neutronUserAddress} 60000${context.secondValidatorAddress}/3 --from ${context.gaiaUserAddress}  --yes --chain-id testgaia --home=/opt --keyring-backend=test --gas auto --gas-adjustment 2 --output json`,
              );
              expect(res.exitCode).toBe(0);
              const out = JSON.parse(res.out);
              expect(out.code).toBe(0);
              expect(out.txhash).toHaveLength(64);
              await waitForTx(context.gaiaClient, out.txhash);
            }
          });
          it('wait for balances to come', async () => {
            await waitFor(async () => {
              const newbalances =
                await context.neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
                  context.neutronUserAddress,
                );
              const newDenoms = newbalances.data.balances.map((b) => b.denom);
              const diff = newDenoms.filter(
                (d) => !oldBalanceDenoms.includes(d),
              );
              lsmDenoms = diff;
              return diff.length === 2;
            }, 500_000);
          });
        });

        it('bond LSM shares', async () => {
          {
            const { coreContractClient, neutronUserAddress } = context;
            const res = await coreContractClient.bond(
              neutronUserAddress,
              {},
              1.6,
              undefined,
              [
                {
                  amount: '60000',
                  denom: lsmDenoms[0],
                },
              ],
            );
            expect(res.transactionHash).toHaveLength(64);
          }
          {
            const { coreContractClient, neutronUserAddress } = context;
            const res = await coreContractClient.bond(
              neutronUserAddress,
              {},
              1.6,
              undefined,
              [
                {
                  amount: '60000',
                  denom: lsmDenoms[1],
                },
              ],
            );
            expect(res.transactionHash).toHaveLength(64);
          }
        });
        it('verify pending lsm shares', async () => {
          const pending =
            await context.coreContractClient.queryPendingLSMShares();
          expect(pending).toHaveLength(2);
        });
      });
      describe('transfering', () => {
        it('tick', async () => {
          const {
            neutronUserAddress,
            gaiaClient,
            coreContractClient,
            puppeteerContractClient,
          } = context;

          await waitForPuppeteerICQ(
            gaiaClient,
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
          expect(state).toEqual('l_s_m_transfer');
        });
        it('wait for the response from puppeteer', async () => {
          let response: DeprecatedResponseHookMsg;
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
          expect(response).toBeTruthy();
          expect<DeprecatedResponseHookMsg>(response).toHaveProperty('success');
        });
        it('wait for ICQ update', async () => {
          await waitForPuppeteerICQ(
            context.gaiaClient,
            context.coreContractClient,
            context.puppeteerContractClient,
          );
        });
        it('one lsm share is gone from the contract balance', async () => {
          const balances =
            await context.neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
              context.coreContractClient.contractAddress,
            );
          expect(
            balances.data.balances.find((one) => one.denom === lsmDenoms[0]),
          ).toBeFalsy();
        });
        it('await for pending length decrease', async () => {
          let pending: any;
          await waitFor(async () => {
            try {
              const res =
                await context.coreContractClient.queryPendingLSMShares();
              pending = res;
            } catch (e) {
              //
            }
            return !!pending && pending.length === 1;
          }, 60_000);
        });
        it('tick to idle', async () => {
          const { neutronUserAddress } = context;
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
        it('tick to lsm transfer', async () => {
          const { neutronUserAddress } = context;
          const res = await context.coreContractClient.tick(
            neutronUserAddress,
            1.5,
            undefined,
            [],
          );
          expect(res.transactionHash).toHaveLength(64);
          const state = await context.coreContractClient.queryContractState();
          expect(state).toEqual('l_s_m_transfer');
        });
        it('wait for the response from puppeteer', async () => {
          let response: DeprecatedResponseHookMsg;
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
          expect(response).toBeTruthy();
          expect<DeprecatedResponseHookMsg>(response).toHaveProperty('success');
        });
        it('wait for ICQ update', async () => {
          await waitForPuppeteerICQ(
            context.gaiaClient,
            context.coreContractClient,
            context.puppeteerContractClient,
          );
        });
        it('second lsm share is gone from the contract balance', async () => {
          const balances =
            await context.neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
              context.coreContractClient.contractAddress,
            );
          expect(
            balances.data.balances.find((one) => one.denom === lsmDenoms[1]),
          ).toBeFalsy();
        });
        it('await for pending length decrease', async () => {
          let pending: any;
          await waitFor(async () => {
            try {
              const res =
                await context.coreContractClient.queryPendingLSMShares();
              pending = res;
            } catch (e) {
              //
            }
            return !!pending && pending.length === 0;
          }, 60_000);
          expect(pending).toEqual([]);
        });
      });
      describe('redeem', () => {
        let delegationsSum = 0;
        it('query delegations', async () => {
          const res: any = await context.puppeteerContractClient.queryExtension(
            {
              msg: {
                delegations: {},
              },
            },
          );
          for (const d of res.delegations.delegations) {
            delegationsSum += parseInt(d.amount.amount);
          }
        });
        it('verify pending lsm shares to unbond', async () => {
          const pending =
            await context.coreContractClient.queryLSMSharesToRedeem();
          expect(pending).toHaveLength(2);
        });
        it('tick to idle', async () => {
          const {
            gaiaClient,
            neutronUserAddress,
            coreContractClient,
            puppeteerContractClient,
          } = context;
          await waitForPuppeteerICQ(
            gaiaClient,
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
        it('tick to redeem', async () => {
          const { neutronUserAddress } = context;
          const res = await context.coreContractClient.tick(
            neutronUserAddress,
            1.5,
            undefined,
            [],
          );
          expect(res.transactionHash).toHaveLength(64);
          const state = await context.coreContractClient.queryContractState();
          expect(state).toEqual('l_s_m_redeem');
        });
        it('imeediately tick again fails', async () => {
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
        it('await for pending length decrease', async () => {
          await waitFor(async () => {
            const pending =
              await context.coreContractClient.queryLSMSharesToRedeem();
            return pending.length === 0;
          }, 30_000);
        });
        it('wait for delegations to come', async () => {
          const { remote_height: currentHeight } =
            await context.puppeteerContractClient.queryExtension({
              msg: {
                delegations: {},
              },
            });
          await waitFor(async () => {
            const { remote_height: nowHeight } =
              await context.puppeteerContractClient.queryExtension({
                msg: {
                  delegations: {},
                },
              });
            return nowHeight !== currentHeight;
          });
        });
        it('query delegations', async () => {
          const res: any = await context.puppeteerContractClient.queryExtension(
            {
              msg: {
                delegations: {},
              },
            },
          );
          let newDelegationsSum = 0;
          for (const d of res.delegations.delegations) {
            newDelegationsSum += parseInt(d.amount.amount);
          }
          expect(newDelegationsSum - delegationsSum).toEqual(120_000);
        });
        it('verify exchange rate', async () => {
          const newExchangeRate =
            await context.coreContractClient.queryExchangeRate();
          expect(parseFloat(newExchangeRate)).toEqual(1);
        });
      });
    });

    describe('forth cycle', () => {
      it('validate NFT', async () => {
        const { withdrawalVoucherContractClient, neutronUserAddress } = context;
        const vouchers = await withdrawalVoucherContractClient.queryTokens({
          owner: context.neutronUserAddress,
        });
        expect(vouchers.tokens.length).toBe(2);
        expect(vouchers.tokens[0]).toBe(`0_${neutronUserAddress}_1`);
        let tokenId = vouchers.tokens[0];
        let voucher = await withdrawalVoucherContractClient.queryNftInfo({
          token_id: tokenId,
        });
        expect(voucher).toBeTruthy();
        expect(voucher).toMatchObject({
          extension: {
            amount: '200000',
            attributes: [
              {
                display_type: null,
                trait_type: 'unbond_batch_id',
                value: '0',
              },
              {
                display_type: null,
                trait_type: 'received_amount',
                value: '200000',
              },
            ],
            batch_id: '0',
            description: 'Withdrawal voucher',
            name: 'LDV voucher',
          },
          token_uri: null,
        });
        expect(vouchers.tokens[1]).toBe(`0_${neutronUserAddress}_2`);
        tokenId = vouchers.tokens[1];
        voucher = await withdrawalVoucherContractClient.queryNftInfo({
          token_id: tokenId,
        });
        expect(voucher).toBeTruthy();
        expect(voucher).toMatchObject({
          extension: {
            amount: '300000',
            attributes: [
              {
                display_type: null,
                trait_type: 'unbond_batch_id',
                value: '0',
              },
              {
                display_type: null,
                trait_type: 'received_amount',
                value: '300000',
              },
            ],
            batch_id: '0',
            description: 'Withdrawal voucher',
            name: 'LDV voucher',
          },
          token_uri: null,
        });
      });
      it('bond tokenized share from registered validator', async () => {
        const { coreContractClient, neutronUserAddress } = context;
        const res = await coreContractClient.bond(
          neutronUserAddress,
          {},
          1.6,
          undefined,
          [
            {
              amount: '20000',
              denom: context.tokenizedDenomOnNeutron,
            },
          ],
        );
        expect(res.transactionHash).toHaveLength(64);
      });
      it('try to withdraw from paused manager', async () => {
        const {
          withdrawalVoucherContractClient,
          neutronUserAddress,
          factoryContractClient: contractClient,
          account,
        } = context;

        await contractClient.pause(account.address);

        const tokenId = `0_${neutronUserAddress}_1`;
        await expect(
          withdrawalVoucherContractClient.sendNft(neutronUserAddress, {
            token_id: tokenId,
            contract: context.withdrawalManagerContractClient.contractAddress,
            msg: Buffer.from(
              JSON.stringify({
                withdraw: {},
              }),
            ).toString('base64'),
          }),
        ).rejects.toThrowError(/Contract execution is paused/);

        await contractClient.unpause(account.address);
      });
      it('try to withdraw before withdrawn', async () => {
        const { withdrawalVoucherContractClient, neutronUserAddress } = context;
        const tokenId = `0_${neutronUserAddress}_1`;
        await expect(
          withdrawalVoucherContractClient.sendNft(neutronUserAddress, {
            token_id: tokenId,
            contract: context.withdrawalManagerContractClient.contractAddress,
            msg: Buffer.from(
              JSON.stringify({
                withdraw: {},
              }),
            ).toString('base64'),
          }),
        ).rejects.toThrowError(/is not withdrawn yet/);
      });
      it('update idle interval', async () => {
        const { factoryContractClient, neutronUserAddress } = context;
        const res = await factoryContractClient.updateConfig(
          neutronUserAddress,
          {
            core: {
              idle_min_interval: 10,
            },
          },
        );
        expect(res.transactionHash).toHaveLength(64);
        await sleep(10 * 1000);
      });
      it('wait until unbonding period is finished', async () => {
        const batchInfo = await context.coreContractClient.queryUnbondBatch({
          batch_id: '0',
        });
        const currentTime = Math.floor(Date.now() / 1000);
        if (batchInfo.expected_release_time > currentTime) {
          const diffMs =
            (batchInfo.expected_release_time - currentTime + 1) * 1000;
          await sleep(diffMs);
        }
      });
      it('wait until fresh ICA balance is delivered', async () => {
        const batchInfo = await context.coreContractClient.queryUnbondBatch({
          batch_id: '0',
        });
        await waitFor(async () => {
          const icaTs = Math.floor(
            (
              (await context.puppeteerContractClient.queryExtension({
                msg: {
                  balances: {},
                },
              })) as any
            ).timestamp / 1e9,
          );
          return icaTs > batchInfo.expected_release_time;
        }, 50_000);
      });
      it('tick to idle', async () => {
        const { coreContractClient, neutronUserAddress } = context;
        await coreContractClient.tick(neutronUserAddress, 1.5, undefined, []);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('idle');
      });
      it('tick to claiming', async () => {
        const {
          coreContractClient,
          neutronUserAddress,
          puppeteerContractClient,
        } = context;
        await waitForPuppeteerICQ(
          context.gaiaClient,
          coreContractClient,
          puppeteerContractClient,
        );
        await coreContractClient.tick(neutronUserAddress, 1.5, undefined, []);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('claiming');
      });
      it('wait for the response from puppeteer', async () => {
        let response: DeprecatedResponseHookMsg;
        await waitFor(async () => {
          try {
            response = (
              await context.coreContractClient.queryLastPuppeteerResponse()
            ).response;
          } catch (e) {
            return false;
          }
          if (!response || !('success' in response)) return false;
          return true;
        }, 30_000);
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
      it('tick to idle', async () => {
        const {
          gaiaClient,
          coreContractClient,
          neutronUserAddress,
          puppeteerContractClient,
        } = context;

        await waitForPuppeteerICQ(
          gaiaClient,
          coreContractClient,
          puppeteerContractClient,
        );

        await coreContractClient.tick(neutronUserAddress, 1.5, undefined, []);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('idle');
      });
      it('validate rewards pump ICA balance', async () => {
        const { gaiaClient, rewardsPumpIcaAddress } = context;
        const res = await gaiaClient.getBalance(rewardsPumpIcaAddress, 'stake');
        const newBalance = parseInt(res.amount);
        expect(newBalance).toBeGreaterThan(0);
      });
      it('validate unbonding batch', async () => {
        const batch = await context.coreContractClient.queryUnbondBatch({
          batch_id: '0',
        });
        expect(batch).toEqual<DeprecatedUnbondBatch>({
          slashing_effect: '1',
          status: 'withdrawn',
          status_timestamps: expect.any(Object),
          expected_release_time: expect.any(Number),
          total_dasset_amount_to_withdraw: '500000',
          expected_native_asset_amount: '500000',
          total_unbond_items: 2,
          unbonded_amount: '500000',
          withdrawn_amount: null,
        });
      });
      it('withdraw with non funded withdrawal manager', async () => {
        const {
          withdrawalVoucherContractClient: voucherContractClient,
          neutronUserAddress,
        } = context;
        const tokenId = `0_${neutronUserAddress}_1`;
        await expect(
          voucherContractClient.sendNft(neutronUserAddress, {
            token_id: tokenId,
            contract: context.withdrawalManagerContractClient.contractAddress,
            msg: Buffer.from(
              JSON.stringify({
                withdraw: {},
              }),
            ).toString('base64'),
          }),
        ).rejects.toThrowError(/spendable balance [\w/]+ is smaller than/);
      });
      it('fund withdrawal manager', async () => {
        const { pumpContractClient, neutronUserAddress } = context;
        await pumpContractClient.push(
          neutronUserAddress,
          {
            coins: [{ amount: '500000', denom: 'stake' }],
          },
          1.5,
          undefined,
          [{ amount: '20000', denom: 'untrn' }],
        );
        await waitFor(async () => {
          const balances =
            await context.neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
              context.withdrawalManagerContractClient.contractAddress,
            );
          return balances.data.balances.length > 0;
        }, 40_000);
      });
      it('withdraw', async () => {
        const {
          withdrawalVoucherContractClient: voucherContractClient,
          neutronUserAddress,
          neutronClient,
          neutronIBCDenom,
        } = context;
        const balanceBefore = parseInt(
          (
            await neutronClient.CosmosBankV1Beta1.query.queryBalance(
              neutronUserAddress,
              { denom: neutronIBCDenom },
            )
          ).data.balance.amount,
        );
        const tokenId = `0_${neutronUserAddress}_1`;
        const res = await voucherContractClient.sendNft(neutronUserAddress, {
          token_id: tokenId,
          contract: context.withdrawalManagerContractClient.contractAddress,
          msg: Buffer.from(
            JSON.stringify({
              withdraw: {},
            }),
          ).toString('base64'),
        });
        expect(res.transactionHash).toHaveLength(64);
        const balance =
          await neutronClient.CosmosBankV1Beta1.query.queryBalance(
            neutronUserAddress,
            { denom: neutronIBCDenom },
          );
        expect(parseInt(balance.data.balance.amount) - balanceBefore).toBe(
          200000,
        );
      });
      it('withdraw to custom receiver', async () => {
        const {
          withdrawalVoucherContractClient: voucherContractClient,
          neutronUserAddress,
          neutronSecondUserAddress,
          neutronClient,
          neutronIBCDenom,
        } = context;
        const balanceBefore = parseInt(
          (
            await neutronClient.CosmosBankV1Beta1.query.queryBalance(
              neutronSecondUserAddress,
              { denom: neutronIBCDenom },
            )
          ).data.balance.amount,
        );
        expect(balanceBefore).toEqual(0);
        const tokenId = `0_${neutronUserAddress}_2`;
        const res = await voucherContractClient.sendNft(neutronUserAddress, {
          token_id: tokenId,
          contract: context.withdrawalManagerContractClient.contractAddress,
          msg: Buffer.from(
            JSON.stringify({
              withdraw: {
                receiver: neutronSecondUserAddress,
              },
            }),
          ).toString('base64'),
        });
        expect(res.transactionHash).toHaveLength(64);
        const balance =
          await neutronClient.CosmosBankV1Beta1.query.queryBalance(
            neutronSecondUserAddress,
            { denom: neutronIBCDenom },
          );
        expect(parseInt(balance.data.balance.amount)).toBe(300000);
      });
    });

    describe('fifth cycle (unbond before delegation)', () => {
      it('tick to claiming', async () => {
        const {
          coreContractClient,
          neutronUserAddress,
          puppeteerContractClient,
        } = context;
        await waitForPuppeteerICQ(
          context.gaiaClient,
          coreContractClient,
          puppeteerContractClient,
        );
        await coreContractClient.tick(neutronUserAddress, 1.5, undefined, []);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('claiming');
      });
      it('wait for the response from puppeteer', async () => {
        let response: DeprecatedResponseHookMsg;
        await waitFor(async () => {
          try {
            response = (
              await context.coreContractClient.queryLastPuppeteerResponse()
            ).response;
          } catch (e) {
            return false;
          }
          if (!response || !('success' in response)) return false;
          return true;
        }, 30_000);
      });
      it('tick to idle', async () => {
        const {
          gaiaClient,
          coreContractClient,
          neutronUserAddress,
          puppeteerContractClient,
        } = context;

        await waitForPuppeteerICQ(
          gaiaClient,
          coreContractClient,
          puppeteerContractClient,
        );

        await coreContractClient.tick(neutronUserAddress, 1.5, undefined, []);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('idle');
      });
      it('bond and unbond ibc coins', async () => {
        const {
          coreContractClient,
          ldDenom,
          neutronUserAddress,
          neutronIBCDenom,
        } = context;
        let res = await coreContractClient.bond(
          neutronUserAddress,
          {},
          1.6,
          undefined,
          [
            {
              amount: '1000000',
              denom: neutronIBCDenom,
            },
          ],
        );

        expect(res.transactionHash).toHaveLength(64);

        await awaitBlocks(`http://127.0.0.1:${context.park.ports.gaia.rpc}`, 1);

        res = await context.stakerContractClient.iBCTransfer(
          neutronUserAddress,
          1.5,
          undefined,
          [{ amount: '20000', denom: 'untrn' }],
        );

        expect(res.transactionHash).toHaveLength(64);

        await awaitBlocks(`http://127.0.0.1:${context.park.ports.gaia.rpc}`, 1);

        res = await coreContractClient.unbond(
          neutronUserAddress,
          1.6,
          undefined,
          [
            {
              amount: '1000000',
              denom: ldDenom,
            },
          ],
        );

        expect(res.transactionHash).toHaveLength(64);
      });
    });
    describe('sixth stake rewards', () => {
      let rewardsPumpIcaBalance = 0;
      it('pump rewards', async () => {
        const { rewardsPumpContractClient, neutronUserAddress, gaiaClient } =
          context;
        rewardsPumpIcaBalance = parseInt(
          (await gaiaClient.getBalance(context.rewardsPumpIcaAddress, 'stake'))
            .amount,
          10,
        );
        await rewardsPumpContractClient.push(
          neutronUserAddress,
          {
            coins: [
              { amount: rewardsPumpIcaBalance.toString(), denom: 'stake' },
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
      it('top up splitter', async () => {
        const res = await context.client.sendTokens(
          context.neutronUserAddress,
          context.splitterContractClient.contractAddress,
          [
            {
              amount: (10000 - rewardsPumpIcaBalance).toString(),
              denom: context.neutronIBCDenom,
            },
          ],
          1.5,
        );
        expect(res.transactionHash).toHaveLength(64);
      });
      it('split it', async () => {
        const res = await context.splitterContractClient.distribute(
          context.neutronUserAddress,
          1.5,
          undefined,
        );
        expect(res.transactionHash).toHaveLength(64);
        const stakerBalance = (
          await context.neutronClient.CosmosBankV1Beta1.query.queryBalance(
            context.stakerContractClient.contractAddress,
            { denom: context.neutronIBCDenom },
          )
        ).data.balance.amount;
        expect(parseInt(stakerBalance, 10)).toEqual(10000);
      });
      it('staker ibc transfer', async () => {
        const { neutronUserAddress } = context;
        const res = await context.stakerContractClient.iBCTransfer(
          neutronUserAddress,
          1.5,
          undefined,
          [{ amount: '20000', denom: 'untrn' }],
        );
        expect(res.transactionHash).toHaveLength(64);
        await waitFor(async () => {
          const res = await context.stakerContractClient.queryTxState();
          return res.status === 'idle';
        }, 80_000);
        const balances = await context.gaiaClient.getAllBalances(
          context.stakerIcaAddress,
        );
        expect(balances).toEqual([
          {
            amount: '1010000',
            denom: context.park.config.networks.gaia.denom,
          },
        ]);
      });
    });
  });
});
