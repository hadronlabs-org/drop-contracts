import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import {
  DropAutoWithdrawer,
  DropCore,
  DropFactory,
  DropPump,
  DropPuppeteer,
  DropStrategy,
  DropStaker,
  DropWithdrawalManager,
  DropWithdrawalVoucher,
  DropSplitter,
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
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import { ResponseHookMsg } from 'drop-ts-client/lib/contractLib/dropCore';
import { stringToPath } from '@cosmjs/crypto';
import { sleep } from '../helpers/sleep';
import { waitForPuppeteerICQ } from '../helpers/waitForPuppeteerICQ';
import { instrumentCoreClass } from '../helpers/knot';
import { checkExchangeRate } from '../helpers/exchangeRate';

const DropFactoryClass = DropFactory.Client;
const DropCoreClass = DropCore.Client;
const DropPumpClass = DropPump.Client;
const DropPuppeteerClass = DropPuppeteer.Client;
const DropStrategyClass = DropStrategy.Client;
const DropStakerClass = DropStaker.Client;
const DropWithdrawalVoucherClass = DropWithdrawalVoucher.Client;
const DropWithdrawalManagerClass = DropWithdrawalManager.Client;
const DropAutoWithdrawerClass = DropAutoWithdrawer.Client;
const DropRewardsPumpClass = DropPump.Client;
const DropSplitterClass = DropSplitter.Client;
const UNBONDING_TIME = 360;

describe('Auto withdrawer', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    gaiaWallet2?: DirectSecp256k1HdWallet;
    factoryContractClient?: InstanceType<typeof DropFactoryClass>;
    coreContractClient?: InstanceType<typeof DropCoreClass>;
    strategyContractClient?: InstanceType<typeof DropStrategyClass>;
    stakerContractClient?: InstanceType<typeof DropStakerClass>;
    pumpContractClient?: InstanceType<typeof DropPumpClass>;
    splitterContractClient?: InstanceType<typeof DropSplitterClass>;
    rewardsPumpContractClient?: InstanceType<typeof DropRewardsPumpClass>;
    puppeteerContractClient?: InstanceType<typeof DropPuppeteerClass>;
    withdrawalVoucherContractClient?: InstanceType<
      typeof DropWithdrawalVoucherClass
    >;
    withdrawalManagerContractClient?: InstanceType<
      typeof DropWithdrawalManagerClass
    >;
    autoWithdrawerContractClient?: InstanceType<typeof DropAutoWithdrawerClass>;
    account?: AccountData;
    icaAddress?: string;
    rewardsPumpIcaAddress?: string;
    stakerIcaAddress?: string;
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
      puppeteer?: number;
      staker?: number;
      validatorsSet?: number;
      distribution?: number;
      rewardsManager?: number;
      splitter?: number;
      pump?: number;
    };
    exchangeRate?: number;
    tokenContractAddress?: string;
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
      1.5,
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
        fs.readFileSync(join(__dirname, '../../../artifacts/drop_core.wasm')),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.core = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(join(__dirname, '../../../artifacts/drop_token.wasm')),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.token = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_withdrawal_voucher.wasm'),
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
          join(__dirname, '../../../artifacts/drop_withdrawal_manager.wasm'),
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
          join(__dirname, '../../../artifacts/drop_validators_set.wasm'),
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
          join(__dirname, '../../../artifacts/drop_rewards_manager.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.rewardsManager = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(join(__dirname, '../../../artifacts/drop_staker.wasm')),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.staker = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_splitter.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.splitter = res.codeId;
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
    const res = await client.upload(
      account.address,
      fs.readFileSync(join(__dirname, '../../../artifacts/drop_factory.wasm')),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await DropFactory.Client.instantiate(
      client,
      account.address,
      res.codeId,
      {
        sdk_version: process.env.SDK_VERSION || '0.46.0',
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
          staker_code_id: context.codeIds.staker,
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
        local_denom: 'untrn',
        base_denom: context.neutronIBCDenom,
        core_params: {
          idle_min_interval: 40,
          unbond_batch_switch_time: 60,
          unbonding_safe_period: 10,
          unbonding_period: UNBONDING_TIME,
          lsm_redeem_threshold: 10,
          lsm_redeem_max_interval: 60_000,
          lsm_min_bond_amount: '1',
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
    context.strategyContractClient = new DropStrategy.Client(
      context.client,
      res.strategy_contract,
    );
    context.tokenContractAddress = res.token_contract;
    context.puppeteerContractClient = new DropPuppeteer.Client(
      context.client,
      res.puppeteer_contract,
    );
    context.stakerContractClient = new DropStaker.Client(
      context.client,
      res.staker_contract,
    );
    context.splitterContractClient = new DropSplitter.Client(
      context.client,
      res.splitter_contract,
    );
    context.rewardsPumpContractClient = new DropPump.Client(
      context.client,
      res.rewards_pump_contract,
    );
    context.ldDenom = `factory/${context.tokenContractAddress}/drop`;
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

  it('setup auto withdrawer', async () => {
    const { client, account, ldDenom, neutronUserAddress, neutronIBCDenom } =
      context;
    {
      const res = await context.coreContractClient.bond(
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
    }
    {
      const res = await context.coreContractClient.unbond(
        neutronUserAddress,
        1.6,
        undefined,
        [
          {
            amount: Math.floor(500_000 / context.exchangeRate).toString(),
            denom: context.ldDenom,
          },
        ],
      );
      expect(res.transactionHash).toHaveLength(64);
    }
    const res = await client.upload(
      account.address,
      fs.readFileSync(
        join(__dirname, '../../../artifacts/drop_auto_withdrawer.wasm'),
      ),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await DropAutoWithdrawer.Client.instantiate(
      client,
      account.address,
      res.codeId,
      {
        core_address: context.coreContractClient.contractAddress,
        withdrawal_voucher_address:
          context.withdrawalVoucherContractClient.contractAddress,
        withdrawal_manager_address:
          context.withdrawalManagerContractClient.contractAddress,
        ld_token: ldDenom,
      },
      'drop-auto-withdrawer',
      'auto',
      [],
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.autoWithdrawerContractClient = new DropAutoWithdrawer.Client(
      client,
      instantiateRes.contractAddress,
    );
  });
  it('bond with ld assets', async () => {
    const { neutronUserAddress, ldDenom, autoWithdrawerContractClient } =
      context;
    const res = await autoWithdrawerContractClient.bond(
      neutronUserAddress,
      {
        with_ld_assets: {},
      },
      1.6,
      undefined,
      [
        {
          amount: String(20000),
          denom: ldDenom,
        },
        {
          amount: String(50000),
          denom: 'untrn',
        },
      ],
    );
    expect(res.transactionHash).toHaveLength(64);

    const bondings = await autoWithdrawerContractClient.queryBondings({
      user: neutronUserAddress,
    });
    expect(bondings).toEqual({
      bondings: [
        {
          bonder: neutronUserAddress,
          deposit: [
            {
              amount: '50000',
              denom: 'untrn',
            },
          ],
          token_id: `0_${autoWithdrawerContractClient.contractAddress}_2`,
        },
      ],
      next_page_key: null,
    });

    await checkExchangeRate(context);
  });
  it('unbond', async () => {
    const {
      neutronUserAddress,
      autoWithdrawerContractClient,
      withdrawalVoucherContractClient,
    } = context;
    const res = await autoWithdrawerContractClient.unbond(
      neutronUserAddress,
      {
        token_id: `0_${autoWithdrawerContractClient.contractAddress}_2`,
      },
      1.6,
      undefined,
      [],
    );
    expect(res.transactionHash).toHaveLength(64);

    const owner = await withdrawalVoucherContractClient.queryOwnerOf({
      token_id: `0_${autoWithdrawerContractClient.contractAddress}_2`,
    });
    expect(owner.owner).toEqual(neutronUserAddress);

    const bondings = await autoWithdrawerContractClient.queryBondings({
      user: neutronUserAddress,
    });
    expect(bondings).toEqual({
      bondings: [],
      next_page_key: null,
    });
    await checkExchangeRate(context);
  });
  it('bond with NFT', async () => {
    const {
      neutronUserAddress,
      autoWithdrawerContractClient,
      withdrawalVoucherContractClient,
    } = context;

    {
      const res = await withdrawalVoucherContractClient.approve(
        neutronUserAddress,
        {
          spender: autoWithdrawerContractClient.contractAddress,
          token_id: `0_${autoWithdrawerContractClient.contractAddress}_2`,
        },
        1.6,
        undefined,
        [],
      );
      expect(res.transactionHash).toHaveLength(64);
    }
    {
      const res = await autoWithdrawerContractClient.bond(
        neutronUserAddress,
        {
          with_n_f_t: {
            token_id: `0_${autoWithdrawerContractClient.contractAddress}_2`,
          },
        },
        1.6,
        undefined,
        [{ amount: '40000', denom: 'untrn' }],
      );
      expect(res.transactionHash).toHaveLength(64);
    }

    const bondings = await autoWithdrawerContractClient.queryBondings({
      user: neutronUserAddress,
    });
    expect(bondings).toEqual({
      bondings: [
        {
          bonder: neutronUserAddress,
          deposit: [
            {
              amount: '40000',
              denom: 'untrn',
            },
          ],
          token_id: `0_${autoWithdrawerContractClient.contractAddress}_2`,
        },
      ],
      next_page_key: null,
    });

    await checkExchangeRate(context);
  });

  describe('state machine', () => {
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
        expect(state).toEqual('staking_bond');
        await checkExchangeRate(context);
      });
      it('wait for response from staker', async () => {
        let response;
        await waitFor(async () => {
          try {
            response = (
              await context.coreContractClient.queryLastStakerResponse()
            ).response;
          } catch (e) {
            //
          }
          return !!response;
        }, 100_000);
      });
      it('get staker ICA zeroed balance', async () => {
        const { gaiaClient } = context;
        const res = await gaiaClient.getBalance(
          context.stakerIcaAddress,
          'stake',
        );
        const balance = parseInt(res.amount);
        expect(0).toEqual(ica.balance);
        ica.balance = balance;
      });
      it('wait for balances to come', async () => {
        let res;
        const { remote_height: currentHeight } =
          await context.puppeteerContractClient.queryExtension({
            msg: {
              balances: {},
            },
          });
        await waitFor(async () => {
          try {
            res = await context.puppeteerContractClient.queryExtension({
              msg: {
                balances: {},
              },
            });
            return res.remote_height !== currentHeight;
          } catch (e) {
            //
          }
        }, 100_000);
      });
      it('second tick goes to unbonding', async () => {
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
        await checkExchangeRate(context);
      });
      it('wait for response from puppeteer', async () => {
        let response: ResponseHookMsg;
        await waitFor(async () => {
          try {
            response = (
              await context.coreContractClient.queryLastPuppeteerResponse()
            ).response;
          } catch (e) {
            return false;
          }
          if (response === null) {
            return false;
          }
          if ('error' in response) {
            throw new Error(response.error.details);
          }
          return response && 'success' in response;
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
        await checkExchangeRate(context);
      });
    });
    describe('second cycle', () => {
      let balance = 0;
      it('get ICA balance', async () => {
        const { gaiaClient } = context;
        const res = await gaiaClient.getBalance(context.icaAddress, 'stake');
        balance = parseInt(res.amount);
      });
      it('wait for 30 seconds', async () => {
        await sleep(30_000);
      });
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
      it('get ICA balance', async () => {
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
        await checkExchangeRate(context);
      });
    });
    describe('third cycle', () => {
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
      it(`wait until unbonding period is finished`, async () => {
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
      it('wait for ICA balance', async () => {
        const { gaiaClient } = context;
        await waitFor(async () => {
          const res = await gaiaClient.getBalance(context.icaAddress, 'stake');
          return parseInt(res.amount) > 0;
        }, 60_000);
      });
      it('wait until fresh ICA balance is delivered', async () => {
        const batchInfo = await context.coreContractClient.queryUnbondBatch({
          batch_id: '0',
        });
        await waitFor(async () => {
          const res = (await context.puppeteerContractClient.queryExtension({
            msg: {
              balances: {},
            },
          })) as any;
          const icaTs = Math.floor(res.timestamp / 1e9);
          return icaTs > batchInfo.expected_release_time;
        }, 500_000);
      });
      it('tick', async () => {
        const { coreContractClient, neutronUserAddress } = context;
        await coreContractClient.tick(neutronUserAddress, 1.5, undefined, []);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('claiming');
        await checkExchangeRate(context);
      });
      it('wait for the response from puppeteer', async () => {
        let response: ResponseHookMsg;
        await waitFor(async () => {
          try {
            response = (
              await context.coreContractClient.queryLastPuppeteerResponse()
            ).response;
          } catch (e) {
            //
          }
          return !!response;
        }, 200_000);
        expect(response).toBeTruthy();
        expect<ResponseHookMsg>(response).toHaveProperty('success');
      });
      it('wait for ICQ update', async () => {
        await waitForPuppeteerICQ(
          context.gaiaClient,
          context.coreContractClient,
          context.puppeteerContractClient,
        );
      });
      it('tick', async () => {
        const {
          coreContractClient,
          neutronUserAddress,
          gaiaClient,
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
        await checkExchangeRate(context);
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
        }, 200_000);
      });
      it('withdraw', async () => {
        const {
          neutronUserAddress,
          neutronClient,
          neutronIBCDenom,
          autoWithdrawerContractClient,
        } = context;
        const expectedWithdrawnAmount = 20000;

        const balanceBefore = parseInt(
          (
            await neutronClient.CosmosBankV1Beta1.query.queryBalance(
              neutronUserAddress,
              { denom: neutronIBCDenom },
            )
          ).data.balance.amount,
        );

        const res = await autoWithdrawerContractClient.withdraw(
          neutronUserAddress,
          {
            token_id: `0_${autoWithdrawerContractClient.contractAddress}_2`,
          },
          1.6,
          undefined,
          [],
        );
        expect(res.transactionHash).toHaveLength(64);

        const withdrawnBatch =
          await context.coreContractClient.queryUnbondBatch({
            batch_id: '0',
          });
        expect(parseInt(withdrawnBatch.withdrawn_amount, 10)).toBeCloseTo(
          expectedWithdrawnAmount,
          -1,
        );

        const balance =
          await neutronClient.CosmosBankV1Beta1.query.queryBalance(
            neutronUserAddress,
            { denom: neutronIBCDenom },
          );
        expect(
          parseInt(balance.data.balance.amount, 10) - balanceBefore,
        ).toBeCloseTo(expectedWithdrawnAmount, -1);

        const bondings = await autoWithdrawerContractClient.queryBondings({
          user: neutronUserAddress,
        });
        expect(bondings).toEqual({
          bondings: [],
          next_page_key: null,
        });
        await checkExchangeRate(context);
      });
    });
  });
});
