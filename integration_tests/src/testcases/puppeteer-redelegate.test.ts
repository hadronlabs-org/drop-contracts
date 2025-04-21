import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import {
  DropCore,
  DropFactory,
  DropPump,
  DropPuppeteer,
  DropStrategy,
  DropWithdrawalManager,
  DropRewardsManager,
  DropToken,
  DropNativeBondProvider,
  DropValidatorsSet,
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
import {
  instantiate2Address,
  SigningCosmWasmClient,
} from '@cosmjs/cosmwasm-stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { awaitBlocks, awaitTargetChannels, setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import { stringToPath } from '@cosmjs/crypto';
import { instrumentCoreClass } from '../helpers/knot';
import { checkExchangeRate } from '../helpers/exchangeRate';
import { fromHex, toAscii } from '@cosmjs/encoding';
import { waitForPuppeteerICQ } from '../helpers/waitForPuppeteerICQ';
import { ResponseHookMsg } from 'drop-ts-client/lib/contractLib/dropCore';

const DropTokenClass = DropToken.Client;
const DropFactoryClass = DropFactory.Client;
const DropCoreClass = DropCore.Client;
const DropPumpClass = DropPump.Client;
const DropPuppeteerClass = DropPuppeteer.Client;
const DropStrategyClass = DropStrategy.Client;
const DropWithdrawalManagerClass = DropWithdrawalManager.Client;
const DropRewardsManagerClass = DropRewardsManager.Client;
const DropRewardsPumpClass = DropPump.Client;
const DropNativeBondProviderClass = DropNativeBondProvider.Client;
const DropValidatorsSetClass = DropValidatorsSet.Client;

const UNBONDING_TIME = 360;

const SALT = 'salt';

describe('Puppeteer ICA redelegate', () => {
  const context: {
    park?: Cosmopark;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    factoryContractClient?: InstanceType<typeof DropFactoryClass>;
    coreContractClient?: InstanceType<typeof DropCoreClass>;
    strategyContractClient?: InstanceType<typeof DropStrategyClass>;
    pumpContractClient?: InstanceType<typeof DropPumpClass>;
    puppeteerContractClient?: InstanceType<typeof DropPuppeteerClass>;
    tokenContractClient?: InstanceType<typeof DropTokenClass>;
    withdrawalManagerContractClient?: InstanceType<
      typeof DropWithdrawalManagerClass
    >;
    rewardsManagerContractClient?: InstanceType<typeof DropRewardsManagerClass>;
    rewardsPumpContractClient?: InstanceType<typeof DropRewardsPumpClass>;
    nativeBondProviderContractClient?: InstanceType<
      typeof DropNativeBondProviderClass
    >;
    validatorsSetClient?: InstanceType<typeof DropValidatorsSetClass>;
    account?: AccountData;
    puppeteerIcaAddress?: string;
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
      factory?: number;
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
      nativeBondProvider?: number;
    };
    predefinedContractAddresses: {
      factoryAddress?: string;
      coreAddress?: string;
      puppeteerAddress?: string;
      strategyAddress?: string;
      validatorSetAddress?: string;
      withdrawalManagerAddress?: string;
      splitterAddress?: string;
    };
    exchangeRate?: number;
    neutronIBCDenom?: string;
    ldDenom?: string;
  } = { codeIds: {}, predefinedContractAddresses: {} };

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
        join(__dirname, '../../../artifacts/drop_factory.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.factory = res.codeId;

      context.predefinedContractAddresses.factoryAddress = instantiate2Address(
        fromHex(res.checksum),
        account.address,
        toAscii(SALT),
        'neutron',
      );
    }

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

      context.predefinedContractAddresses.coreAddress = instantiate2Address(
        fromHex(res.checksum),
        context.predefinedContractAddresses.factoryAddress,
        toAscii(SALT),
        'neutron',
      );
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

    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../../artifacts/drop_withdrawal_voucher.wasm'),
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
        join(__dirname, '../../../artifacts/drop_withdrawal_manager.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.withdrawalManager = res.codeId;

      context.predefinedContractAddresses.withdrawalManagerAddress =
        instantiate2Address(
          fromHex(res.checksum),
          context.predefinedContractAddresses.factoryAddress,
          toAscii(SALT),
          'neutron',
        );
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

      let instantiateRes = await DropPump.Client.instantiate(
        context.client,
        context.account.address,
        context.codeIds.pump,
        {
          connection_id: 'connection-0',
          local_denom: 'untrn',
          timeout: {
            local: 60,
            remote: 60,
          },
          dest_address: context.predefinedContractAddresses.splitterAddress,
          dest_port: 'transfer',
          dest_channel: 'channel-0',
          refundee: context.account.address,
          owner: context.predefinedContractAddresses.factoryAddress,
        },
        'drop-staking-rewards-pump',
        1.5,
        [],
        context.predefinedContractAddresses.factoryAddress,
      );

      context.rewardsPumpContractClient = new DropPump.Client(
        context.client,
        instantiateRes.contractAddress,
      );

      instantiateRes = await DropPump.Client.instantiate(
        context.client,
        context.account.address,
        context.codeIds.pump,
        {
          connection_id: 'connection-0',
          local_denom: 'untrn',
          timeout: {
            local: 60,
            remote: 60,
          },
          dest_address:
            context.predefinedContractAddresses.withdrawalManagerAddress,
          dest_port: 'transfer',
          dest_channel: 'channel-0',
          refundee: context.account.address,
          owner: context.predefinedContractAddresses.factoryAddress,
        },
        'drop-staking-unbonding-pump',
        1.5,
        [],
        context.predefinedContractAddresses.factoryAddress,
      );

      context.pumpContractClient = new DropPump.Client(
        context.client,
        instantiateRes.contractAddress,
      );
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

      context.predefinedContractAddresses.strategyAddress = instantiate2Address(
        fromHex(res.checksum),
        context.predefinedContractAddresses.factoryAddress,
        toAscii(SALT),
        'neutron',
      );
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
        join(__dirname, '../../../artifacts/drop_validators_set.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.validatorsSet = res.codeId;

      context.predefinedContractAddresses.validatorSetAddress =
        instantiate2Address(
          fromHex(res.checksum),
          context.predefinedContractAddresses.factoryAddress,
          toAscii(SALT),
          'neutron',
        );
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
        join(__dirname, '../../../artifacts/drop_splitter.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.splitter = res.codeId;

      context.predefinedContractAddresses.splitterAddress = instantiate2Address(
        fromHex(res.checksum),
        context.predefinedContractAddresses.factoryAddress,
        toAscii(SALT),
        'neutron',
      );
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
        join(__dirname, '../../../artifacts/drop_puppeteer.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.puppeteer = res.codeId;

      context.predefinedContractAddresses.puppeteerAddress =
        instantiate2Address(
          fromHex(res.checksum),
          account.address,
          toAscii(SALT),
          'neutron',
        );

      let instantiateRes = await DropNativeBondProvider.Client.instantiate(
        context.client,
        context.account.address,
        context.codeIds.nativeBondProvider,
        {
          owner: context.predefinedContractAddresses.factoryAddress,
          base_denom: context.neutronIBCDenom,
          factory_contract: context.predefinedContractAddresses.factoryAddress,
          min_stake_amount: '10000',
          min_ibc_transfer: '10000',
          port_id: 'transfer',
          transfer_channel_id: 'channel-0',
          timeout: 60,
        },
        'drop-staking-native-bond-provider',
        1.5,
        [],
        context.predefinedContractAddresses.factoryAddress,
      );

      context.nativeBondProviderContractClient =
        new DropNativeBondProvider.Client(
          context.client,
          instantiateRes.contractAddress,
        );

      instantiateRes = await DropPuppeteer.Client.instantiate2(
        context.client,
        account.address,
        context.codeIds.puppeteer,
        toAscii(SALT),
        {
          allowed_senders: [
            context.nativeBondProviderContractClient.contractAddress,
            context.predefinedContractAddresses.coreAddress,
            context.predefinedContractAddresses.factoryAddress,
          ],
          owner: context.predefinedContractAddresses.factoryAddress,
          remote_denom: 'stake',
          update_period: 5,
          connection_id: 'connection-0',
          port_id: 'transfer',
          transfer_channel_id: 'channel-0',
          sdk_version: process.env.SDK_VERSION || '0.47.16',
          timeout: 60,
          factory_contract: context.predefinedContractAddresses.factoryAddress,
        },
        'drop-staking-puppeteer',
        1.5,
        [],
        context.predefinedContractAddresses.factoryAddress,
      );

      context.puppeteerContractClient = new DropPuppeteer.Client(
        context.client,
        instantiateRes.contractAddress,
      );
    }

    const instantiateRes = await DropFactory.Client.instantiate2(
      client,
      account.address,
      context.codeIds.factory,
      toAscii(SALT),
      {
        local_denom: 'untrn',
        code_ids: {
          core_code_id: context.codeIds.core,
          token_code_id: context.codeIds.token,
          withdrawal_voucher_code_id: context.codeIds.withdrawalVoucher,
          withdrawal_manager_code_id: context.codeIds.withdrawalManager,
          strategy_code_id: context.codeIds.strategy,
          distribution_code_id: context.codeIds.distribution,
          validators_set_code_id: context.codeIds.validatorsSet,
          rewards_manager_code_id: context.codeIds.rewardsManager,
          splitter_code_id: context.codeIds.splitter,
        },
        pre_instantiated_contracts: {
          native_bond_provider_address:
            context.nativeBondProviderContractClient.contractAddress,
          puppeteer_address:
            context.predefinedContractAddresses.puppeteerAddress,
          unbonding_pump_address: context.pumpContractClient.contractAddress,
          rewards_pump_address:
            context.rewardsPumpContractClient.contractAddress,
        },
        remote_opts: {
          connection_id: 'connection-0',
          transfer_channel_id: 'channel-0',
          denom: 'stake',
          timeout: {
            local: 60,
            remote: 60,
          },
        },
        salt: SALT,
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
          idle_min_interval: 120,
          unbond_batch_switch_time: 60,
          unbonding_safe_period: 10,
          unbonding_period: 360,
          icq_update_delay: 5,
        },
      },
      'drop-staking-factory',
      'auto',
      [],
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.factoryContractClient = new DropFactory.Client(
      client,
      instantiateRes.contractAddress,
    );
  });

  it('query factory state', async () => {
    const { factoryContractClient: contractClient } = context;
    const res = await contractClient.queryState();
    expect(res).toBeTruthy();
    context.coreContractClient = instrumentCoreClass(
      new DropCore.Client(context.client, res.core_contract),
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
    context.tokenContractClient = new DropToken.Client(
      context.client,
      res.token_contract,
    );
    context.puppeteerContractClient = new DropPuppeteer.Client(
      context.client,
      res.puppeteer_contract,
    );
    context.validatorsSetClient = new DropValidatorsSet.Client(
      context.client,
      res.validators_set_contract,
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
    context.puppeteerIcaAddress = ica;
  });

  it('set up rewards receiver', async () => {
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

  it('query exchange rate', async () => {
    const { coreContractClient } = context;
    context.exchangeRate = parseFloat(
      await coreContractClient.queryExchangeRate(),
    );
    expect(context.exchangeRate).toEqual(1);
    await checkExchangeRate(context);
  });

  it('bond', async () => {
    const {
      coreContractClient,
      neutronClient,
      neutronUserAddress,
      neutronIBCDenom,
      neutronSecondUserAddress,
    } = context;
    const res = await coreContractClient.bond(
      neutronUserAddress,
      { receiver: neutronSecondUserAddress },
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
    const balances =
      await neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
        neutronSecondUserAddress,
      );
    const ldBalance = balances.data.balances.find((one) =>
      one.denom.startsWith('factory'),
    );
    context.ldDenom = ldBalance?.denom;
  });

  it('setup unbonding pump', async () => {
    const { neutronUserAddress } = context;

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

  it('make two ticks', async () => {
    const {
      gaiaClient,
      neutronUserAddress,
      coreContractClient,
      puppeteerContractClient,
    } = context;

    for (let i = 0; i < 2; i += 1) {
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
    }
  });

  it('wait for native bond provider to get into idle state', async () => {
    let response;
    await waitFor(async () => {
      try {
        response =
          await context.nativeBondProviderContractClient.queryTxState();
      } catch (e) {
        //
      }
      return response.status === 'idle';
    }, 100_000);
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
    }, 100_000);
  });

  it('make two ticks', async () => {
    const {
      gaiaClient,
      neutronUserAddress,
      coreContractClient,
      puppeteerContractClient,
    } = context;

    for (let i = 0; i < 2; i += 1) {
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
    }
  });

  it('wait delegations before redelegate', async () => {
    const {
      puppeteerContractClient,
      validatorAddress,
      secondValidatorAddress,
      puppeteerIcaAddress,
    } = context;
    await waitFor(async () => {
      const res: any = await puppeteerContractClient.queryExtension({
        msg: {
          delegations: {},
        },
      });
      return res && res.delegations.delegations.length > 0;
    }, 100_000);
    const delegations = (
      (await puppeteerContractClient.queryExtension({
        msg: {
          delegations: {},
        },
      })) as any
    ).delegations.delegations;
    delegations.sort((a, b) => a.validator.localeCompare(b.validator));
    const expectedDelegations = [
      {
        delegator: puppeteerIcaAddress,
        validator: validatorAddress,
        amount: {
          denom: context.park.config.networks.gaia.denom,
          amount: String(500000),
        },
        share_ratio: '1',
      },
      {
        delegator: puppeteerIcaAddress,
        validator: secondValidatorAddress,
        amount: {
          denom: context.park.config.networks.gaia.denom,
          amount: String(500000),
        },
        share_ratio: '1',
      },
    ];
    expectedDelegations.sort((a, b) => a.validator.localeCompare(b.validator));
    expect(delegations).toEqual(expectedDelegations);
  });

  it('redelegate specific amount from one validator to another', async () => {
    const {
      factoryContractClient,
      puppeteerContractClient,
      neutronUserAddress,
      validatorAddress,
      secondValidatorAddress,
    } = context;
    await factoryContractClient.adminExecute(
      neutronUserAddress,
      {
        msgs: [
          {
            wasm: {
              execute: {
                contract_addr: puppeteerContractClient.contractAddress,
                msg: Buffer.from(
                  JSON.stringify({
                    redelegate: {
                      amount: '250000',
                      validator_from: validatorAddress,
                      validator_to: secondValidatorAddress,
                      reply_to: puppeteerContractClient.contractAddress,
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
  });

  it('wait delegations after redelegate', async () => {
    const {
      gaiaClient,
      coreContractClient,
      puppeteerContractClient,
      validatorAddress,
      secondValidatorAddress,
      puppeteerIcaAddress,
    } = context;

    await waitForPuppeteerICQ(
      gaiaClient,
      coreContractClient,
      puppeteerContractClient,
    );

    await waitFor(async () => {
      const res: any = await puppeteerContractClient.queryExtension({
        msg: {
          delegations: {},
        },
      });
      return res && res.delegations.delegations.length > 0;
    }, 100_000);
    const delegations = (
      (await puppeteerContractClient.queryExtension({
        msg: {
          delegations: {},
        },
      })) as any
    ).delegations.delegations;
    delegations.sort((a, b) => a.validator.localeCompare(b.validator));
    const expectedDelegations = [
      {
        delegator: puppeteerIcaAddress,
        validator: validatorAddress,
        amount: {
          denom: context.park.config.networks.gaia.denom,
          amount: String(250000),
        },
        share_ratio: '1',
      },
      {
        delegator: puppeteerIcaAddress,
        validator: secondValidatorAddress,
        amount: {
          denom: context.park.config.networks.gaia.denom,
          amount: String(750000),
        },
        share_ratio: '1',
      },
    ];
    expectedDelegations.sort((a, b) => a.validator.localeCompare(b.validator));
    expect(delegations).toEqual(expectedDelegations);
  });
});
