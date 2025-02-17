import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import {
  DropCore,
  DropPump,
  DropFactory,
  DropNativeBondProvider,
  DropPuppeteer,
  DropUnbondingMirror,
  DropWithdrawalManager,
  DropWithdrawalVoucher,
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
import { waitForPuppeteerICQ } from '../helpers/waitForPuppeteerICQ';
import { UnbondBatch } from 'drop-ts-client/lib/contractLib/dropCore';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import { stringToPath } from '@cosmjs/crypto';
import { instrumentCoreClass } from '../helpers/knot';
import { sleep } from '../helpers/sleep';
import { sha256 } from '@cosmjs/crypto';

const DropUnbondingMirrorClass = DropUnbondingMirror.Client;
const DropFactoryClass = DropFactory.Client;
const DropCoreClass = DropCore.Client;
const DropWithdrawalManagerClass = DropWithdrawalManager.Client;
const DropWithdrawalVoucherClass = DropWithdrawalVoucher.Client;
const DropNativeBondProviderClass = DropNativeBondProvider.Client;
const DropRewardsPumpClass = DropPump.Client;
const DropPumpClass = DropPump.Client;
const DropPuppeteerClass = DropPuppeteer.Client;
const UNBONDING_TIME = 360;

describe('Unbonding mirror', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    gaiaWallet2?: DirectSecp256k1HdWallet;
    factoryContractClient?: InstanceType<typeof DropFactoryClass>;
    coreContractClient?: InstanceType<typeof DropCoreClass>;
    withdrawalManagerClient?: InstanceType<typeof DropWithdrawalManagerClass>;
    withdrawalVoucherClient?: InstanceType<typeof DropWithdrawalVoucherClass>;
    unbondingMirrorClient?: InstanceType<typeof DropUnbondingMirrorClass>;
    rewardsPumpContractClient?: InstanceType<typeof DropRewardsPumpClass>;
    pumpContractClient?: InstanceType<typeof DropPumpClass>;
    account?: AccountData;
    icaAddress?: string;
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
    nativeBondProviderContractClient?: InstanceType<
      typeof DropNativeBondProviderClass
    >;
    puppeteerContractClient?: InstanceType<typeof DropPuppeteerClass>;
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
      mirror?: number;
      unbondingMirror?: number;
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
            join(__dirname, '../../../artifacts/drop_puppeteer.wasm'),
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
            join(__dirname, '../../../artifacts/drop_unbonding_mirror.wasm'),
          ),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.unbondingMirror = res.codeId;
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
        sdk_version: process.env.SDK_VERSION || '0.47.10',
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
          idle_min_interval: 120,
          unbond_batch_switch_time: 60,
          unbonding_safe_period: 10,
          unbonding_period: UNBONDING_TIME,
          icq_update_delay: 5,
        },
        native_bond_params: {
          min_stake_amount: '100',
          min_ibc_transfer: '100',
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
    context.nativeBondProviderContractClient =
      new DropNativeBondProvider.Client(
        context.client,
        res.native_bond_provider_contract,
      );
    context.puppeteerContractClient = new DropPuppeteer.Client(
      context.client,
      res.puppeteer_contract,
    );
    context.withdrawalManagerClient = new DropWithdrawalManager.Client(
      context.client,
      res.withdrawal_manager_contract,
    );
    context.withdrawalVoucherClient = new DropWithdrawalVoucher.Client(
      context.client,
      res.withdrawal_voucher_contract,
    );
    context.rewardsPumpContractClient = new DropPump.Client(
      context.client,
      res.rewards_pump_contract,
    );
    context.ldDenom = `factory/${res.token_contract}/drop`;
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

  it('deploy pump', async () => {
    const { client, account, neutronUserAddress } = context;
    const resUpload = await client.upload(
      account.address,
      Uint8Array.from(
        fs.readFileSync(join(__dirname, '../../../artifacts/drop_pump.wasm')),
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
        dest_address: context.withdrawalManagerClient.contractAddress,
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

  it('instantiate unbonding mirror', async () => {
    const res = await DropUnbondingMirror.Client.instantiate(
      context.client,
      context.neutronUserAddress,
      context.codeIds.unbondingMirror,
      {
        core_contract: context.coreContractClient.contractAddress,
        withdrawal_manager: context.withdrawalManagerClient.contractAddress,
        withdrawal_voucher: context.withdrawalVoucherClient.contractAddress,
        source_channel: 'channel-0',
        source_port: 'transfer',
        ibc_timeout: 10,
        prefix: 'cosmos',
        retry_limit: 1,
        ibc_denom: context.neutronIBCDenom,
      },
      'mirror',
      1.6,
    );
    expect(res.contractAddress).toHaveLength(66);
    context.unbondingMirrorClient = new DropUnbondingMirror.Client(
      context.client,
      res.contractAddress,
    );
  });

  it('bond 1m uAssets', async () => {
    const { neutronUserAddress, coreContractClient } = context;
    await coreContractClient.bond(
      neutronUserAddress,
      {},
      undefined,
      undefined,
      [
        {
          denom: context.neutronIBCDenom,
          amount: '1000000',
        },
      ],
    );
  });

  it('send 10 ntrn to the unbonding mirror contract', async () => {
    const { neutronUserAddress, client, unbondingMirrorClient } = context;
    await client.sendTokens(
      neutronUserAddress,
      unbondingMirrorClient.contractAddress,
      [{ denom: 'untrn', amount: '10000000' }],
      {
        gas: '100000',
        amount: [{ denom: 'untrn', amount: '10000' }],
      },
    );
    expect(
      (await client.getBalance(unbondingMirrorClient.contractAddress, 'untrn'))
        .amount,
    ).toBe('10000000');
  });

  describe('Expected behavior', () => {
    const denomsMirror: Array<{ gaiaDenom: string; neutronDenom: string }> = [];
    it('unbond 1k, wait for the new voucher on gaia', async () => {
      const {
        neutronUserAddress,
        gaiaUserAddress,
        gaiaClient,
        unbondingMirrorClient,
      } = context;
      const { events } = await unbondingMirrorClient.unbond(
        neutronUserAddress,
        {
          receiver: gaiaUserAddress,
        },
        undefined,
        undefined,
        [
          {
            denom: context.ldDenom,
            amount: '1000',
          },
        ],
      );
      const neutronDenom = events
        .filter(
          (event) =>
            event.type ===
            'wasm-crates.io:drop-staking__drop-unbonding-mirror-reply_finalize_unbond',
        )[0]
        .attributes.filter(
          (attribute) => attribute.key === 'tf_denom',
        )[0].value;
      denomsMirror.push({
        neutronDenom: neutronDenom,
        gaiaDenom: `ibc/${sha256(
          new TextEncoder().encode(`transfer/channel-0/${neutronDenom}`),
        )
          .toString()
          .toUpperCase()}`,
      });
      await waitFor(async () => {
        const balances = await gaiaClient.getAllBalances(gaiaUserAddress);
        return (
          balances.filter((denom) => denom.denom.startsWith('ibc/')).length ===
          1
        );
      }, 60_000);
    });
    it('unbond 10k, wait for the new voucher on gaia', async () => {
      const {
        neutronUserAddress,
        gaiaUserAddress,
        gaiaClient,
        unbondingMirrorClient,
      } = context;
      const { events } = await unbondingMirrorClient.unbond(
        neutronUserAddress,
        {
          receiver: gaiaUserAddress,
        },
        undefined,
        undefined,
        [
          {
            denom: context.ldDenom,
            amount: '10000',
          },
        ],
      );
      const neutronDenom = events
        .filter(
          (event) =>
            event.type ===
            'wasm-crates.io:drop-staking__drop-unbonding-mirror-reply_finalize_unbond',
        )[0]
        .attributes.filter(
          (attribute) => attribute.key === 'tf_denom',
        )[0].value;
      denomsMirror.push({
        neutronDenom: neutronDenom,
        gaiaDenom: `ibc/${sha256(
          new TextEncoder().encode(`transfer/channel-0/${neutronDenom}`),
        )
          .toString()
          .toUpperCase()}`,
      });
      await waitFor(async () => {
        const balances = await gaiaClient.getAllBalances(gaiaUserAddress);
        return (
          balances.filter((denom) => denom.denom.startsWith('ibc/')).length ===
          2
        );
      }, 60_000);
    });

    describe('do a protocol full rotation', () => {
      it('first tick did nothing and stays in idle', async () => {
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
      it('tick 1 (peripheral) transfer coins from neutron to target chain', async () => {
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
        expect(state).toEqual('peripheral');
      });
      it('tick 2 (idle)', async () => {
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
      it('tick 3 (peripheral) stake collected coins on remote chain', async () => {
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
        }, 200_000);
        const delegations =
          (await context.puppeteerContractClient.queryExtension({
            msg: {
              delegations: {},
            },
          })) as any;

        expect(delegations.delegations.delegations).toHaveLength(2);
        console.log(delegations.delegations.delegations);
        expect(
          parseInt(delegations.delegations.delegations[0].amount.amount, 10),
        ).toEqual(500000);
        expect(
          parseInt(delegations.delegations.delegations[1].amount.amount, 10),
        ).toEqual(500000);
      });
      it('tick 4 (idle)', async () => {
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
      it('tick 5 (claiming)', async () => {
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
      });
      it('tick 6 (unbonding)', async () => {
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
        expect(state).toEqual('unbonding');
      });
      it('tick 7 (idle)', async () => {
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
          total_dasset_amount_to_withdraw: '11000',
          expected_native_asset_amount: '11000',
          total_unbond_items: 2,
          unbonded_amount: null,
          withdrawn_amount: null,
        });
      });
      it('wait until unbonding period for unbonding batch 0 is finished', async () => {
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
      });
      it('verify that unbonding batch 0 is in withdrawing state', async () => {
        const batch = await context.coreContractClient.queryUnbondBatch({
          batch_id: '0',
        });
        expect(batch).toBeTruthy();
        expect(batch).toEqual<UnbondBatch>({
          slashing_effect: '1',
          status: 'withdrawing',
          status_timestamps: expect.any(Object),
          expected_release_time: expect.any(Number),
          total_dasset_amount_to_withdraw: '11000',
          expected_native_asset_amount: '11000',
          total_unbond_items: 2,
          unbonded_amount: '11000',
          withdrawn_amount: null,
        });
      });
      it('tick (idle)', async () => {
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
      });
      it('fund withdrawal manager', async () => {
        const { pumpContractClient, neutronUserAddress } = context;
        await pumpContractClient.push(
          neutronUserAddress,
          {
            coins: [{ amount: '11000', denom: 'stake' }],
          },
          1.5,
          undefined,
          [{ amount: '20000', denom: 'untrn' }],
        );
        await waitFor(async () => {
          const balances =
            await context.neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
              context.withdrawalManagerClient.contractAddress,
            );
          return balances.data.balances.length > 0;
        }, 40_000);
      });
    });

    it('send vouchers back on neutron', async () => {
      const { gaiaClient, gaiaUserAddress, neutronUserAddress, client } =
        context;
      for (const denom of denomsMirror.map((denom) => denom.gaiaDenom)) {
        await gaiaClient.signAndBroadcast(
          gaiaUserAddress,
          [
            {
              typeUrl: '/ibc.applications.transfer.v1.MsgTransfer',
              value: MsgTransfer.fromPartial({
                sender: gaiaUserAddress,
                sourceChannel: 'channel-0',
                sourcePort: 'transfer',
                receiver: neutronUserAddress,
                token: { denom: denom, amount: '1' },
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
      }
      await waitFor(
        async () =>
          (
            await client.getBalance(
              neutronUserAddress,
              denomsMirror[0].neutronDenom,
            )
          ).amount === '1' &&
          (
            await client.getBalance(
              neutronUserAddress,
              denomsMirror[1].neutronDenom,
            )
          ).amount === '1',
        60_000,
      );
    });

    it('withdraw', async () => {
      const {
        unbondingMirrorClient,
        neutronUserAddress,
        gaiaUserAddress,
        gaiaClient,
      } = context;
      const balanceBefore = (
        await gaiaClient.getBalance(gaiaUserAddress, 'stake')
      ).amount;
      console.log(await gaiaClient.getBalance(gaiaUserAddress, 'stake'));
      for (const denom of denomsMirror.map((denom) => denom.neutronDenom)) {
        await unbondingMirrorClient.withdraw(
          neutronUserAddress,
          {
            receiver: gaiaUserAddress,
          },
          1.5,
          undefined,
          [{ denom: denom, amount: '1' }],
        );
      }
      await waitFor(
        async () =>
          (await gaiaClient.getBalance(gaiaUserAddress, 'stake')).amount !==
          balanceBefore,
        60_000,
      );
      console.log(await gaiaClient.getBalance(gaiaUserAddress, 'stake'));
    });
  });
});
