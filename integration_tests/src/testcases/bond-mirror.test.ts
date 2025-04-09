import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import {
  DropCore,
  DropFactory,
  DropMirror,
  DropNativeBondProvider,
  DropLsmShareBondProvider,
  DropPuppeteer,
  DropPump,
  DropToken,
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
import { setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import { stringToPath } from '@cosmjs/crypto';
import { instrumentCoreClass } from '../helpers/knot';
import { sleep } from '../helpers/sleep';
import { fromHex, toAscii } from '@cosmjs/encoding';

const DropMirrorClass = DropMirror.Client;
const DropFactoryClass = DropFactory.Client;
const DropCoreClass = DropCore.Client;
const DropPumpClass = DropPump.Client;
const DropNativeBondProviderClass = DropNativeBondProvider.Client;
const DropLsmShareBondProviderClass = DropLsmShareBondProvider.Client;
const DropPuppeteerClass = DropPuppeteer.Client;
const DropTokenClass = DropToken.Client;
const UNBONDING_TIME = 360;

const SALT = 'salt';

describe('Mirror', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    gaiaWallet2?: DirectSecp256k1HdWallet;
    factoryContractClient?: InstanceType<typeof DropFactoryClass>;
    coreContractClient?: InstanceType<typeof DropCoreClass>;
    mirrorContractClient?: InstanceType<typeof DropMirrorClass>;
    pumpContractClient?: InstanceType<typeof DropPumpClass>;
    rewardsPumpContractClient?: InstanceType<typeof DropPumpClass>;
    tokenContractClient?: InstanceType<typeof DropTokenClass>;
    account?: AccountData;
    icaAddress?: string;
    rewardsPumpIcaAddress?: string;
    client?: SigningCosmWasmClient;
    neutronStargateClient?: SigningStargateClient;
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
    nativeBondProviderContractClient?: InstanceType<
      typeof DropNativeBondProviderClass
    >;
    lsmShareBondProviderContractClient?: InstanceType<
      typeof DropLsmShareBondProviderClass
    >;
    puppeteerContractClient?: InstanceType<typeof DropPuppeteerClass>;
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
      mirror?: number;
      lsmShareBondProvider?: number;
      nativeBondProvider?: number;
    };
    predefinedContractAddresses: {
      factoryAddress?: string;
      coreAddress?: string;
      puppeteerAddress?: string;
      strategyAddress?: string;
      validatorSetAddress?: string;
      lsmShareBondProviderAddress?: string;
      withdrawalManagerAddress?: string;
      splitterAddress?: string;
    };
    exchangeRate?: number;
    neutronIBCDenom?: string;
    gaiaIBCDenom?: string;
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
    context.neutronStargateClient =
      await SigningStargateClient.connectWithSigner(
        `http://127.0.0.1:${context.park.ports.neutron.rpc}`,
        context.wallet,
        {
          gasPrice: GasPrice.fromString('0.025untrn'),
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
    await sleep(30_000);
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
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(join(__dirname, '../../../artifacts/drop_core.wasm')),
        ),
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
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_splitter.wasm'),
          ),
        ),
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

      context.predefinedContractAddresses.lsmShareBondProviderAddress =
        instantiate2Address(
          fromHex(res.checksum),
          account.address,
          toAscii(SALT),
          'neutron',
        );
    }
    {
      const res = await client.upload(
        account.address,
        Uint8Array.from(
          fs.readFileSync(
            join(__dirname, '../../../artifacts/drop_puppeteer.wasm'),
          ),
        ),
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

      let instantiateRes = await DropLsmShareBondProvider.Client.instantiate2(
        context.client,
        context.account.address,
        context.codeIds.lsmShareBondProvider,
        toAscii(SALT),
        {
          factory_contract: context.predefinedContractAddresses.factoryAddress,
          lsm_redeem_threshold: 2,
          lsm_min_bond_amount: '1000',
          lsm_redeem_maximum_interval: 60_000,
          owner: context.predefinedContractAddresses.factoryAddress,
          port_id: 'transfer',
          transfer_channel_id: 'channel-0',
          timeout: 60,
        },
        'drop-staking-lsm-share-bond-provider',
        1.5,
        [],
        context.predefinedContractAddresses.factoryAddress,
      );

      context.lsmShareBondProviderContractClient =
        new DropLsmShareBondProvider.Client(
          context.client,
          instantiateRes.contractAddress,
        );

      instantiateRes = await DropNativeBondProvider.Client.instantiate2(
        context.client,
        context.account.address,
        context.codeIds.nativeBondProvider,
        toAscii(SALT),
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
            context.predefinedContractAddresses.lsmShareBondProviderAddress,
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
            join(__dirname, '../../../artifacts/drop_mirror.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.mirror = res.codeId;
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
          lsm_share_bond_provider_address:
            context.predefinedContractAddresses.lsmShareBondProviderAddress,
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
    context.tokenContractClient = new DropToken.Client(
      context.client,
      res.token_contract,
    );
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
  it('wait puppeteer response', async () => {
    const { puppeteerContractClient } = context;
    await waitFor(async () => {
      const res = await puppeteerContractClient.queryTxState();
      return res.status === 'idle';
    }, 100_000);
  });

  it('instantiate mirror', async () => {
    const res = await DropMirror.Client.instantiate(
      context.client,
      context.neutronUserAddress,
      context.codeIds.mirror,
      {
        core_contract: context.coreContractClient.contractAddress,
        source_channel: 'channel-0',
        source_port: 'transfer',
        ibc_timeout: 10,
        prefix: 'cosmos',
      },
      'mirror',
      1.6,
    );
    expect(res.contractAddress).toHaveLength(66);
    context.mirrorContractClient = new DropMirror.Client(
      context.client,
      res.contractAddress,
    );
  });

  it('send neutrons on mirror', async () => {
    await context.client.sendTokens(
      context.account.address,
      context.mirrorContractClient.contractAddress,
      [{ denom: 'untrn', amount: '10000000' }],
      {
        gas: '200000',
        amount: [
          {
            denom: 'untrn',
            amount: '10000',
          },
        ],
      },
    );
  });

  it('proper bond', async () => {
    const { neutronIBCDenom, mirrorContractClient, gaiaUserAddress } = context;
    await mirrorContractClient.bond(
      context.neutronUserAddress,
      {
        receiver: gaiaUserAddress,
      },
      1.6,
      undefined,
      [
        {
          denom: neutronIBCDenom,
          amount: '1000',
        },
      ],
    );
    await waitFor(
      async () =>
        (await context.gaiaClient.getAllBalances(context.gaiaUserAddress))
          .length > 1,
      20000,
      1000,
    );
    context.gaiaIBCDenom = (
      await context.gaiaClient.getAllBalances(context.gaiaUserAddress)
    ).find((one) => one.denom.startsWith('ibc/')).denom;
    expect(
      (
        await context.gaiaClient.getBalance(
          context.gaiaUserAddress,
          context.gaiaIBCDenom,
        )
      ).amount,
    ).toBe('1000');
  });

  describe('wrong behaviour', () => {
    it('set timeout to 0', async () => {
      await context.mirrorContractClient.updateConfig(
        context.neutronUserAddress,
        {
          new_config: {
            ibc_timeout: 0,
          },
        },
      );
    });

    describe('bond, timeout packet', () => {
      it('turn off relayer', async () => {
        await context.park.pauseRelayer('hermes', 0);
      });

      it('bond', async () => {
        await context.mirrorContractClient.bond(
          context.neutronUserAddress,
          {
            receiver: context.gaiaUserAddress,
          },
          1.6,
          undefined,
          [
            {
              denom: context.neutronIBCDenom,
              amount: '1000',
            },
          ],
        );
        await sleep(10_000); // make this packet to outlive it's validity
      });

      it('resume relayer', async () => {
        await context.park.resumeRelayer('hermes', 0);
        await sleep(40_000); // sudo-timeout
      });
    });

    it("expect new assets to appear in contract's state", async () => {
      expect(await context.mirrorContractClient.queryAllFailed()).toStrictEqual(
        [
          {
            receiver: context.gaiaUserAddress,
            failed_transfers: [
              {
                denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
                amount: '1000',
              },
            ],
          },
        ],
      );
    });

    describe('bond, timeout packet', () => {
      it('turn off relayer', async () => {
        await context.park.pauseRelayer('hermes', 0);
      });

      it('bond', async () => {
        const {
          mirrorContractClient,
          gaiaUserAddress2,
          neutronUserAddress,
          neutronIBCDenom,
        } = context;
        await mirrorContractClient.bond(
          neutronUserAddress,
          {
            receiver: gaiaUserAddress2,
          },
          1.6,
          undefined,
          [
            {
              denom: neutronIBCDenom,
              amount: '1000',
            },
          ],
        );
        await sleep(10_000); // make this packet to outlive it's validity
      });

      it('resume relayer', async () => {
        await context.park.resumeRelayer('hermes', 0);
        await sleep(10_000); // sudo-timeout
      });
    });

    it("expect new assets to appear in contract's state", async () => {
      const { mirrorContractClient, gaiaUserAddress, gaiaUserAddress2 } =
        context;
      expect(await mirrorContractClient.queryAllFailed()).toEqual(
        expect.arrayContaining([
          {
            receiver: gaiaUserAddress2,
            failed_transfers: [
              {
                denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
                amount: '1000',
              },
            ],
          },
          {
            receiver: gaiaUserAddress,
            failed_transfers: [
              {
                denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
                amount: '1000',
              },
            ],
          },
        ]),
      );
    });

    describe('bond, timeout packet', () => {
      it('turn off relayer', async () => {
        await context.park.pauseRelayer('hermes', 0);
      });

      it('bond', async () => {
        await context.mirrorContractClient.bond(
          context.neutronUserAddress,
          {
            receiver: context.gaiaUserAddress,
          },
          1.6,
          undefined,
          [
            {
              denom: context.neutronIBCDenom,
              amount: '1000',
            },
          ],
        );
        await sleep(10_000); // make this packet to outlive it's validity
      });

      it('resume relayer', async () => {
        await context.park.resumeRelayer('hermes', 0);
        await sleep(10_000); // sudo-timeout
      });
    });

    it("expect new assets to appear in contract's state", async () => {
      const { mirrorContractClient, gaiaUserAddress, gaiaUserAddress2 } =
        context;
      expect(await mirrorContractClient.queryAllFailed()).toEqual(
        expect.arrayContaining([
          {
            receiver: gaiaUserAddress,
            failed_transfers: [
              {
                denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
                amount: '1000',
              },
              {
                denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
                amount: '1000',
              },
            ],
          },
          {
            receiver: gaiaUserAddress2,
            failed_transfers: [
              {
                denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
                amount: '1000',
              },
            ],
          },
        ]),
      );
    });

    describe('retry, timeout packet', () => {
      it('turn off relayer', async () => {
        await context.park.pauseRelayer('hermes', 0);
      });

      it('failed retry', async () => {
        const {
          mirrorContractClient,
          gaiaUserAddress,
          gaiaUserAddress2,
          neutronUserAddress,
        } = context;

        await mirrorContractClient.retry(
          neutronUserAddress,
          {
            receiver: gaiaUserAddress,
          },
          1.6,
        );
        expect(await mirrorContractClient.queryAllFailed()).toEqual(
          expect.arrayContaining([
            {
              receiver: gaiaUserAddress,
              failed_transfers: [
                {
                  denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
                  amount: '1000',
                },
              ],
            },
            {
              receiver: gaiaUserAddress2,
              failed_transfers: [
                {
                  denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
                  amount: '1000',
                },
              ],
            },
          ]),
        );
        await sleep(10_000); // make this packet to outlive it's validity
      });

      it('resume relayer', async () => {
        await context.park.resumeRelayer('hermes', 0);
        await sleep(10_000); // sudo-timeout
      });

      it('restored values after sudo-timeout', async () => {
        const { mirrorContractClient, gaiaUserAddress, gaiaUserAddress2 } =
          context;

        await waitFor(
          async () =>
            (await mirrorContractClient.queryAllFailed()).length === 2,
          60_000,
          5_000,
        );
        expect(await mirrorContractClient.queryAllFailed()).toEqual(
          expect.arrayContaining([
            {
              receiver: gaiaUserAddress,
              failed_transfers: [
                {
                  denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
                  amount: '1000',
                },
                {
                  denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
                  amount: '1000',
                },
              ],
            },
            {
              receiver: gaiaUserAddress2,
              failed_transfers: [
                {
                  denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
                  amount: '1000',
                },
              ],
            },
          ]),
        );
      });
    });

    it('retry with the working relayer (1)', async () => {
      const {
        mirrorContractClient,
        gaiaUserAddress,
        gaiaUserAddress2,
        neutronUserAddress,
      } = context;

      await mirrorContractClient.retry(
        neutronUserAddress,
        {
          receiver: gaiaUserAddress,
        },
        1.6,
      );
      await waitFor(
        async () =>
          (
            await context.gaiaClient.getBalance(
              context.gaiaUserAddress,
              context.gaiaIBCDenom,
            )
          ).amount !== '2000',
        20000,
        1000,
      );
      expect(await mirrorContractClient.queryAllFailed()).toEqual(
        expect.arrayContaining([
          {
            receiver: gaiaUserAddress,
            failed_transfers: [
              {
                denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
                amount: '1000',
              },
            ],
          },
          {
            receiver: gaiaUserAddress2,
            failed_transfers: [
              {
                denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
                amount: '1000',
              },
            ],
          },
        ]),
      );
    });

    it('retry with the working relayer (2)', async () => {
      const {
        mirrorContractClient,
        gaiaUserAddress,
        gaiaUserAddress2,
        neutronUserAddress,
      } = context;

      await mirrorContractClient.retry(
        neutronUserAddress,
        {
          receiver: gaiaUserAddress2,
        },
        1.6,
      );
      await waitFor(
        async () =>
          (
            await context.gaiaClient.getBalance(
              context.gaiaUserAddress2,
              context.gaiaIBCDenom,
            )
          ).amount !== '1000',
        20000,
        1000,
      );
      expect(await mirrorContractClient.queryAllFailed()).toEqual(
        expect.arrayContaining([
          {
            receiver: gaiaUserAddress,
            failed_transfers: [
              {
                denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
                amount: '1000',
              },
            ],
          },
        ]),
      );
    });

    it('retry with the working relayer (3)', async () => {
      const { mirrorContractClient, gaiaUserAddress, neutronUserAddress } =
        context;

      await mirrorContractClient.retry(
        neutronUserAddress,
        {
          receiver: gaiaUserAddress,
        },
        1.6,
      );
      await waitFor(
        async () =>
          (
            await context.gaiaClient.getBalance(
              context.gaiaUserAddress,
              context.gaiaIBCDenom,
            )
          ).amount !== '3000',
        20000,
        1000,
      );
      expect(await mirrorContractClient.queryAllFailed()).toStrictEqual([]);
    });
  });
});
