import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import {
  DropCore,
  DropFactory,
  DropPump,
  DropPuppeteer,
  DropStrategy,
  DropStaker,
  DropWithdrawalManager,
  DropWithdrawalVoucher,
  DropSplitter,
  DropToken,
} from 'drop-ts-client';
import {
  QueryClient,
  StakingExtension,
  BankExtension,
  setupStakingExtension,
  setupBankExtension,
  setupSlashingExtension,
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
import { UnbondBatch } from 'drop-ts-client/lib/contractLib/dropCore';
import { stringToPath } from '@cosmjs/crypto';
import { sleep } from '../helpers/sleep';
import dockerCompose from 'docker-compose';
import { SlashingExtension } from '@cosmjs/stargate/build/modules';
import { waitForPuppeteerICQ } from '../helpers/waitForPuppeteerICQ';
import { instrumentCoreClass } from '../helpers/knot';
import { checkExchangeRate } from '../helpers/exchangeRate';

const DropTokenClass = DropToken.Client;
const DropFactoryClass = DropFactory.Client;
const DropCoreClass = DropCore.Client;
const DropPumpClass = DropPump.Client;
const DropPuppeteerClass = DropPuppeteer.Client;
const DropStakerClass = DropStaker.Client;
const DropStrategyClass = DropStrategy.Client;
const DropWithdrawalVoucherClass = DropWithdrawalVoucher.Client;
const DropWithdrawalManagerClass = DropWithdrawalManager.Client;
const DropRewardsPumpClass = DropPump.Client;
const DropSplitterClass = DropSplitter.Client;

const UNBONDING_TIME = 360;

describe('Core Slashing', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    gaiaWallet2?: DirectSecp256k1HdWallet;
    factoryContractClient?: InstanceType<typeof DropFactoryClass>;
    splitterContractClient?: InstanceType<typeof DropSplitterClass>;
    rewardsPumpContractClient?: InstanceType<typeof DropRewardsPumpClass>;
    coreContractClient?: InstanceType<typeof DropCoreClass>;
    strategyContractClient?: InstanceType<typeof DropStrategyClass>;
    stakerContractClient?: InstanceType<typeof DropStakerClass>;
    pumpContractClient?: InstanceType<typeof DropPumpClass>;
    puppeteerContractClient?: InstanceType<typeof DropPuppeteerClass>;
    tokenContractClient?: InstanceType<typeof DropTokenClass>;
    withdrawalVoucherContractClient?: InstanceType<
      typeof DropWithdrawalVoucherClass
    >;
    withdrawalManagerContractClient?: InstanceType<
      typeof DropWithdrawalManagerClass
    >;
    account?: AccountData;
    icaAddress?: string;
    rewardsPumpIcaAddress?: string;
    stakerIcaAddress?: string;
    client?: SigningCosmWasmClient;
    gaiaClient?: SigningStargateClient;
    gaiaUserAddress?: string;
    gaiaUserAddress2?: string;
    gaiaQueryClient?: QueryClient &
      StakingExtension &
      SlashingExtension &
      BankExtension;
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
    };
    neutronIBCDenom?: string;
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
      setupSlashingExtension,
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
        base_denom: context.neutronIBCDenom,
        core_params: {
          idle_min_interval: 10,
          unbond_batch_switch_time: 60,
          unbonding_safe_period: 10,
          unbonding_period: 360,
          lsm_redeem_threshold: 2,
          lsm_min_bond_amount: '1',
          lsm_redeem_max_interval: 60_000,
          bond_limit: '0',
          min_stake_amount: '2',
          icq_update_delay: 5,
        },
        staker_params: {
          min_stake_amount: '100',
          min_ibc_transfer: '100',
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
    const { factoryContractClient: contractClient } = context;
    const res = await contractClient.queryState();
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
    context.tokenContractClient = new DropToken.Client(
      context.client,
      res.token_contract,
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
  it('add validators into validators set', async () => {
    const {
      neutronUserAddress,
      factoryContractClient,
      validatorAddress,
      secondValidatorAddress,
    } = context;
    await factoryContractClient.proxy(
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
  });
  it('set emergency address', async () => {
    const { neutronUserAddress, factoryContractClient } = context;
    await factoryContractClient.updateConfig(
      neutronUserAddress,
      {
        core: {
          emergency_address: 'cosmos1tqchhqtug30lmz9y6zltdp7cmyctnkshm850rz',
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
        dest_address: context.withdrawalManagerContractClient.contractAddress,
        dest_port: 'transfer',
        dest_channel: 'channel-0',
        refundee: neutronUserAddress,
        owner: account.address,
      },
      'Drop-staking-pump',
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

  describe('prepare unbonding batch 0', () => {
    it('bond', async () => {
      const { coreContractClient, neutronUserAddress, neutronIBCDenom } =
        context;
      await coreContractClient.bond(neutronUserAddress, {}, 1.6, undefined, [
        {
          amount: '1000',
          denom: neutronIBCDenom,
        },
      ]);
      await checkExchangeRate(context);
    });
    it('unbond', async () => {
      const { coreContractClient, neutronUserAddress } = context;
      await coreContractClient.unbond(neutronUserAddress, 1.6, undefined, [
        {
          // we need to leave at least one coin on every resgistered validator
          // (two validators in this case)
          amount: '500',
          denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
        },
      ]);
      await checkExchangeRate(context);
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
      }, 200_000);
      const balances = await context.gaiaClient.getAllBalances(
        context.stakerIcaAddress,
      );
      expect(balances).toEqual([
        {
          amount: '1000',
          denom: context.park.config.networks.gaia.denom,
        },
      ]);
      await checkExchangeRate(context);
    });
    it('tick 1 (staking_bond)', async () => {
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
      expect(state).toEqual('staking_bond');
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
    it('get staker ICA zeroed balance', async () => {
      const { gaiaClient } = context;
      const res = await gaiaClient.getBalance(
        context.stakerIcaAddress,
        context.park.config.networks.gaia.denom,
      );
      const balance = parseInt(res.amount);
      expect(balance).toEqual(0);
    });
    it('tick 2 (unbonding)', async () => {
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
      expect(state).toEqual('unbonding');
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
      await checkExchangeRate(context);
    });
    it('tick 3 (idle)', async () => {
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

      await context.coreContractClient.tick(
        neutronUserAddress,
        1.5,
        undefined,
        [],
      );
      const state = await context.coreContractClient.queryContractState();
      expect(state).toEqual('idle');
      await sleep(10_000); // wait for idle min interval
      await checkExchangeRate(context);
    });
    it('verify that unbonding batch 0 is in unbonding state', async () => {
      const batch = await context.coreContractClient.queryUnbondBatch({
        batch_id: '0',
      });
      expect(batch).toBeTruthy();
      expect(batch).toEqual<UnbondBatch>({
        slashing_effect: null,
        status: 'unbonding',
        status_timestamps: expect.any(Object),
        expected_release_time: expect.any(Number),
        total_dasset_amount_to_withdraw: '500',
        expected_native_asset_amount: '500',
        total_unbond_items: 1,
        unbonded_amount: null,
        withdrawn_amount: null,
      });
    });
  });

  describe('prepare unbonding batch 1', () => {
    it('bond', async () => {
      const { coreContractClient, neutronUserAddress, neutronIBCDenom } =
        context;
      await coreContractClient.bond(neutronUserAddress, {}, 1.6, undefined, [
        {
          amount: '3000',
          denom: neutronIBCDenom,
        },
      ]);
      await checkExchangeRate(context);
    });
    it('unbond', async () => {
      const { coreContractClient, neutronUserAddress } = context;
      await coreContractClient.unbond(neutronUserAddress, 1.6, undefined, [
        {
          amount: '3000',
          denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
        },
      ]);
      await checkExchangeRate(context);
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
      }, 200_000);
      const balances = await context.gaiaClient.getAllBalances(
        context.stakerIcaAddress,
      );
      expect(balances).toEqual([
        {
          amount: '3000',
          denom: context.park.config.networks.gaia.denom,
        },
      ]);
      await checkExchangeRate(context);
    });
    it('tick 1 (claiming)', async () => {
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

      await context.coreContractClient.tick(
        neutronUserAddress,
        1.5,
        undefined,
        [],
      );
      const state = await context.coreContractClient.queryContractState();
      expect(state).toEqual('claiming');
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
      await checkExchangeRate(context);
    });
    it('tick 2 (staking_bond)', async () => {
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
      expect(state).toEqual('staking_bond');
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
      });
      await checkExchangeRate(context);
    });
    it('tick 3 (unbonding)', async () => {
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
      expect(state).toEqual('unbonding');
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
      await checkExchangeRate(context);
    });
    it('tick 4 (idle)', async () => {
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

      await context.coreContractClient.tick(
        neutronUserAddress,
        1.5,
        undefined,
        [],
      );
      const state = await context.coreContractClient.queryContractState();
      expect(state).toEqual('idle');
      await sleep(10_000); // wait for idle min interval
      await checkExchangeRate(context);
    });
    it('verify that unbonding batch 1 is in unbonding state', async () => {
      const batch = await context.coreContractClient.queryUnbondBatch({
        batch_id: '1',
      });
      expect(batch).toBeTruthy();
      expect(batch).toEqual<UnbondBatch>({
        slashing_effect: null,
        status: 'unbonding',
        status_timestamps: expect.any(Object),
        expected_release_time: expect.any(Number),
        total_dasset_amount_to_withdraw: '3000',
        expected_native_asset_amount: '3000',
        total_unbond_items: 1,
        unbonded_amount: null,
        withdrawn_amount: null,
      });
    });
  });
  it('trigger slashing', async () => {
    await dockerCompose.pauseOne('gaia_val2');
    await waitFor(async () => {
      const signingInfos =
        await context.gaiaQueryClient.slashing.signingInfos();
      const v1 = signingInfos.info[0];
      const v2 = signingInfos.info[1];
      return v1.jailedUntil.seconds > 0 || v2.jailedUntil.seconds > 0;
    }, 60_000);
  });
  it('wait until unbonding period for unbonding batch 0 is finished', async () => {
    const batchInfo = await context.coreContractClient.queryUnbondBatch({
      batch_id: '0',
    });
    const currentTime = Math.floor(Date.now() / 1000);
    if (batchInfo.expected_release_time > currentTime) {
      const diffMs = (batchInfo.expected_release_time - currentTime + 1) * 1000;
      await sleep(diffMs);
    }
  });
  it('wait until unbonding period for unbonding batch 1 is finished', async () => {
    const batchInfo = await context.coreContractClient.queryUnbondBatch({
      batch_id: '1',
    });
    const currentTime = Math.floor(Date.now() / 1000);
    if (batchInfo.expected_release_time > currentTime) {
      const diffMs = (batchInfo.expected_release_time - currentTime + 1) * 1000;
      await sleep(diffMs);
    }
  });
  it('wait until fresh ICA balance for unbonding batch 0 is delivered', async () => {
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
  it('wait until fresh ICA balance for unbonding batch 1 is delivered', async () => {
    const batchInfo = await context.coreContractClient.queryUnbondBatch({
      batch_id: '1',
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
  it('tick (claiming)', async () => {
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
    expect(state).toEqual('claiming');
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
    await checkExchangeRate(context);
  });
  it.skip('verify that unbonding batch 0 is in withdrawing emergency state', async () => {
    const batch = await context.coreContractClient.queryUnbondBatch({
      batch_id: '0',
    });
    expect(batch).toBeTruthy();
    expect(batch).toEqual<UnbondBatch>({
      slashing_effect: null,
      status: 'withdrawing_emergency',
      status_timestamps: expect.any(Object),
      expected_release_time: expect.any(Number),
      total_dasset_amount_to_withdraw: '1000',
      expected_native_asset_amount: '1000',
      total_unbond_items: 1,
      unbonded_amount: null,
      withdrawn_amount: null,
    });
  });
  it.skip('verify that unbonding batch 1 is in withdrawing emergency state', async () => {
    const batch = await context.coreContractClient.queryUnbondBatch({
      batch_id: '1',
    });
    expect(batch).toBeTruthy();
    expect(batch).toEqual<UnbondBatch>({
      slashing_effect: null,
      status: 'withdrawing_emergency',
      status_timestamps: expect.any(Object),
      expected_release_time: expect.any(Number),
      total_dasset_amount_to_withdraw: '3000',
      expected_native_asset_amount: '3499',
      total_unbond_items: 1,
      unbonded_amount: null,
      withdrawn_amount: null,
    });
  });
  it.skip('tick (idle)', async () => {
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
  it.skip('verify that unbonding batch 0 is in withdrawn emergency state', async () => {
    const batch = await context.coreContractClient.queryUnbondBatch({
      batch_id: '0',
    });
    expect(batch).toBeTruthy();
    expect(batch).toEqual<UnbondBatch>({
      slashing_effect: null,
      status: 'withdrawn_emergency',
      status_timestamps: expect.any(Object),
      expected_release_time: expect.any(Number),
      total_dasset_amount_to_withdraw: '1000',
      expected_native_asset_amount: '1000',
      total_unbond_items: 1,
      unbonded_amount: null,
      withdrawn_amount: null,
    });
  });
  it.skip('verify that unbonding batch 1 is in withdrawn emergency state', async () => {
    const batch = await context.coreContractClient.queryUnbondBatch({
      batch_id: '1',
    });
    expect(batch).toBeTruthy();
    expect(batch).toEqual<UnbondBatch>({
      slashing_effect: null,
      status: 'withdrawn_emergency',
      status_timestamps: expect.any(Object),
      expected_release_time: expect.any(Number),
      total_dasset_amount_to_withdraw: '3000',
      expected_native_asset_amount: '3499',
      total_unbond_items: 1,
      unbonded_amount: null,
      withdrawn_amount: null,
    });
  });

  it.skip('verify that emergency account has received unbonded funds', async () => {
    const emergencyBalance = parseInt(
      (
        await context.gaiaClient.getBalance(
          'cosmos1tqchhqtug30lmz9y6zltdp7cmyctnkshm850rz',
          'stake',
        )
      ).amount,
    );
    expect(emergencyBalance).toBeGreaterThan(0);
    expect(emergencyBalance).toBeLessThan(1000 + 3499);
  });
});
