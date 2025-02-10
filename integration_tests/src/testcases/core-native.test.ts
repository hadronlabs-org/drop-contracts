import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import {
  DropCore,
  DropFactory,
  DropPuppeteerNative,
  DropStrategy,
  DropWithdrawalManager,
  DropWithdrawalVoucher,
  DropRewardsManager,
  DropSplitter,
  DropToken,
  DropNativeSyncBondProvider,
  DropValRef,
  DropValidatorsSet,
  DropNeutronDistributionMock,
} from 'drop-ts-client';
import {
  QueryClient,
  StakingExtension,
  BankExtension,
  setupStakingExtension,
  setupBankExtension,
} from '@cosmjs/stargate';
import { join } from 'path';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { Tendermint34Client } from '@cosmjs/tendermint-rpc';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import { instrumentCoreClass } from '../helpers/knot';
import {
  ResponseHookMsg,
  UnbondBatch,
} from 'drop-ts-client/lib/contractLib/dropCore';
import { sleep } from '../helpers/sleep';
import { checkExchangeRate } from '../helpers/exchangeRate';
import { stringToPath } from '@cosmjs/crypto';

const DropTokenClass = DropToken.Client;
const DropFactoryClass = DropFactory.Client;
const DropCoreClass = DropCore.Client;
const DropPuppeteerNativeClass = DropPuppeteerNative.Client;
const DropStrategyClass = DropStrategy.Client;
const DropWithdrawalVoucherClass = DropWithdrawalVoucher.Client;
const DropWithdrawalManagerClass = DropWithdrawalManager.Client;
const DropRewardsManagerClass = DropRewardsManager.Client;
const DropSplitterClass = DropSplitter.Client;
const DropNativeSyncBondProviderClass = DropNativeSyncBondProvider.Client;
const DropValRefClass = DropValRef.Client;
const DropValidatorsSetClass = DropValidatorsSet.Client;
const DropNeutronDistributionMockClass = DropNeutronDistributionMock.Client;

const UNBONDING_TIME = 360;

describe('Core', () => {
  const context: {
    park?: Cosmopark;
    wallet?: DirectSecp256k1HdWallet;
    distributionMockClient?: InstanceType<
      typeof DropNeutronDistributionMockClass
    >;
    factoryContractClient?: InstanceType<typeof DropFactoryClass>;
    coreContractClient?: InstanceType<typeof DropCoreClass>;
    strategyContractClient?: InstanceType<typeof DropStrategyClass>;
    puppeteerContractClient?: InstanceType<typeof DropPuppeteerNativeClass>;
    splitterContractClient?: InstanceType<typeof DropSplitterClass>;
    tokenContractClient?: InstanceType<typeof DropTokenClass>;
    withdrawalVoucherContractClient?: InstanceType<
      typeof DropWithdrawalVoucherClass
    >;
    withdrawalManagerContractClient?: InstanceType<
      typeof DropWithdrawalManagerClass
    >;
    rewardsManagerContractClient?: InstanceType<typeof DropRewardsManagerClass>;
    nativeBondProviderContractClient?: InstanceType<
      typeof DropNativeSyncBondProviderClass
    >;
    valRefClient?: InstanceType<typeof DropValRefClass>;
    validatorsSetClient?: InstanceType<typeof DropValidatorsSetClass>;
    account?: AccountData;
    rewardsPumpIcaAddress?: string;
    client?: SigningCosmWasmClient;
    queryClient?: QueryClient & StakingExtension & BankExtension;
    neutronClient?: InstanceType<typeof NeutronClient>;
    neutronRPCEndpoint?: string;
    neutronUserAddress?: string;
    validatorAddress?: string;
    secondValidatorAddress?: string;
    codeIds: {
      core?: number;
      token?: number;
      withdrawalVoucher?: number;
      withdrawalManager?: number;
      redemptionRateAdapter?: number;
      strategy?: number;
      puppeteer?: number;
      validatorsSet?: number;
      distribution?: number;
      rewardsManager?: number;
      splitter?: number;
      pump?: number;
      nativeBondProvider?: number;
      valRef?: number;
      distributionModuleMock?: number;
    };
    exchangeRate?: number;
    ldDenom?: string;
  } = { codeIds: {} };

  beforeAll(async (t) => {
    context.park = await setupPark(t, ['neutronv2'], {
      neutronv2: {
        genesis_opts: {
          'app_state.staking.params.unbonding_time': `${UNBONDING_TIME}s`,
          'app_state.staking.params.bond_denom': `untrn`,
        },
      },
    });

    context.neutronClient = new NeutronClient({
      apiURL: `http://127.0.0.1:${context.park.ports.neutronv2.rest}`,
      rpcURL: `127.0.0.1:${context.park.ports.neutronv2.rpc}`,
      prefix: 'neutron',
    });

    context.wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demowallet1.mnemonic,
      {
        prefix: 'neutron',
      },
    );

    context.account = (await context.wallet.getAccounts())[0];

    context.neutronRPCEndpoint = `http://127.0.0.1:${context.park.ports.neutronv2.rpc}`;
    context.client = await SigningCosmWasmClient.connectWithSigner(
      context.neutronRPCEndpoint,
      context.wallet,
      {
        gasPrice: GasPrice.fromString('0.025untrn'),
      },
    );
    const tmClient = await Tendermint34Client.connect(
      `http://127.0.0.1:${context.park.ports.neutronv2.rpc}`,
    );
    context.queryClient = QueryClient.withExtensions(
      tmClient,
      setupStakingExtension,
      setupBankExtension,
    );
    context.neutronUserAddress = (
      await context.wallet.getAccounts()
    )[0].address;

    {
      const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
        context.park.config.master_mnemonic,
        {
          prefix: 'neutronvaloper',
          hdPaths: [stringToPath("m/44'/118'/1'/0/0") as any],
        },
      );
      context.validatorAddress = (await wallet.getAccounts())[0].address;
    }
    {
      const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
        context.park.config.master_mnemonic,
        {
          prefix: 'neutronvaloper',
          hdPaths: [stringToPath("m/44'/118'/2'/0/0") as any],
        },
      );
      context.secondValidatorAddress = (await wallet.getAccounts())[0].address;
    }
  });

  afterAll(async () => {
    await context.park.stop();
  });

  it('instantiate', async () => {
    const { client, account } = context;
    context.codeIds = {};

    {
      const buffer = fs.readFileSync(
        join(
          __dirname,
          '../../../artifacts/drop_neutron_distribution_mock.wasm',
        ),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.distributionModuleMock = res.codeId;
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
    }
    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../../artifacts/drop_puppeteer_native.wasm'),
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
    }
    {
      const buffer = fs.readFileSync(
        join(__dirname, '../../../artifacts/drop_redemption_rate_adapter.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.redemptionRateAdapter = res.codeId;
    }
    {
      const buffer = fs.readFileSync(
        join(
          __dirname,
          '../../../artifacts/drop_native_sync_bond_provider.wasm',
        ),
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
        join(__dirname, '../../../artifacts/drop_val_ref.wasm'),
      );

      const res = await client.upload(
        account.address,
        new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.valRef = res.codeId;
    }

    let instantiateRes = await DropNeutronDistributionMock.Client.instantiate(
      client,
      account.address,
      context.codeIds.distributionModuleMock,
      {},
      'distribution-module-mock',
      'auto',
      [],
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.distributionMockClient = new DropNeutronDistributionMock.Client(
      client,
      instantiateRes.contractAddress,
    );

    const buffer = fs.readFileSync(
      join(__dirname, '../../../artifacts/drop_factory.wasm'),
    );

    const res = await client.upload(
      account.address,
      new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    instantiateRes = await DropFactory.Client.instantiate(
      client,
      account.address,
      res.codeId,
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
          puppeteer_code_id: context.codeIds.puppeteer,
          rewards_manager_code_id: context.codeIds.rewardsManager,
          splitter_code_id: context.codeIds.splitter,
          rewards_pump_code_id: context.codeIds.pump,
          native_bond_provider_code_id: context.codeIds.nativeBondProvider,
        },
        remote_opts: {
          connection_id: 'connection-0',
          denom: 'stake',
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
        base_denom: 'untrn',
        core_params: {
          idle_min_interval: 120,
          unbond_batch_switch_time: 60,
          unbonding_safe_period: 10,
          unbonding_period: 360,
          bond_limit: null,
          icq_update_delay: 5,
        },
        factory: {
          native: {
            distribution_module_contract:
              context.distributionMockClient.contractAddress,
          },
        },
      },
      'drop-staking-factory',
      'auto',
      [
        {
          denom: 'untrn',
          amount: '10000000',
        },
      ],
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
    context.tokenContractClient = new DropToken.Client(
      context.client,
      res.token_contract,
    );
    context.puppeteerContractClient = new DropPuppeteerNative.Client(
      context.client,
      res.puppeteer_contract,
    );
    context.splitterContractClient = new DropSplitter.Client(
      context.client,
      res.splitter_contract,
    );
    context.nativeBondProviderContractClient =
      new DropNativeSyncBondProvider.Client(
        context.client,
        res.native_bond_provider_contract,
      );
    context.validatorsSetClient = new DropValidatorsSet.Client(
      context.client,
      res.validators_set_contract,
    );
  });

  it('set up withdrawal manager address', async () => {
    const res = await context.factoryContractClient.updateConfig(
      context.neutronUserAddress,
      {
        core: {
          pump_ica_address:
            context.rewardsManagerContractClient.contractAddress,
        },
      },
    );
    expect(res.transactionHash).toHaveLength(64);
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
                      rewards_withdraw_address:
                        context.splitterContractClient.contractAddress,
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
  });
  it('query exchange rate', async () => {
    const { coreContractClient } = context;
    context.exchangeRate = parseFloat(
      await coreContractClient.queryExchangeRate(),
    );
    expect(context.exchangeRate).toEqual(1);
    await checkExchangeRate(context);
  });
  it('deploy val ref contract', async () => {
    const res = await DropValRef.Client.instantiate(
      context.client,
      context.account.address,
      context.codeIds.valRef,
      {
        owner: context.account.address,
        core_address: context.coreContractClient.contractAddress,
        validators_set_address: context.validatorsSetClient.contractAddress,
      },
      'drop-val-ref',
      1.5,
    );
    expect(res.contractAddress).toHaveLength(66);
    context.valRefClient = new DropValRef.Client(
      context.client,
      res.contractAddress,
    );

    await context.factoryContractClient.adminExecute(
      context.account.address,
      {
        msgs: [
          {
            wasm: {
              execute: {
                contract_addr: context.validatorsSetClient.contractAddress,
                msg: Buffer.from(
                  JSON.stringify({
                    update_config: {
                      new_config: {
                        val_ref_contract: context.valRefClient.contractAddress,
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
    );
  });
  it('register first validator in val_ref', async () => {
    await context.valRefClient.setRefs(context.account.address, {
      refs: [
        {
          ref: 'valX001',
          validator_address: context.validatorAddress,
        },
      ],
    });
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
                weight: 2,
                on_top: '80000',
              },
              {
                valoper_address: secondValidatorAddress,
                weight: 3,
                on_top: '20000',
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
  it('register core bond hook', async () => {
    await context.factoryContractClient.adminExecute(
      context.account.address,
      {
        msgs: [
          {
            wasm: {
              execute: {
                contract_addr: context.coreContractClient.contractAddress,
                msg: Buffer.from(
                  JSON.stringify({
                    set_bond_hooks: {
                      hooks: [context.valRefClient.contractAddress],
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

  it('bond', async () => {
    const { coreContractClient, neutronClient, neutronUserAddress } = context;
    const res = await coreContractClient.bond(
      neutronUserAddress,
      {},
      1.6,
      undefined,
      [
        {
          amount: '400000',
          denom: 'untrn',
        },
      ],
    );
    expect(res.transactionHash).toHaveLength(64);
    const contractAttributes = res.events.find(
      (e) => e.type === 'wasm-crates.io:drop-staking__drop-core-execute-bond',
    ).attributes;

    const attributesList = contractAttributes.map((e) => e.key);
    expect(attributesList).toContain('used_bond_provider');

    const usedBondProvider = contractAttributes.find(
      (e) => e.key === 'used_bond_provider',
    );
    expect(usedBondProvider.value).toEqual(
      context.nativeBondProviderContractClient.contractAddress,
    );

    const balances =
      await neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
        neutronUserAddress,
      );
    const ldBalance = balances.data.balances.find((one) =>
      one.denom.startsWith('factory'),
    );
    expect(ldBalance).toEqual({
      denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
      amount: String(Math.floor(400_000 / context.exchangeRate)),
    });
    context.ldDenom = ldBalance?.denom;
    await checkExchangeRate(context);
  });
  it('verify bonded amount', async () => {
    const { coreContractClient } = context;
    const bonded = await coreContractClient.queryTotalBonded();
    expect(bonded).toEqual('400000');
  });

  it('unbond', async () => {
    const { coreContractClient, neutronUserAddress, ldDenom } = context;
    const res = await coreContractClient.unbond(
      neutronUserAddress,
      1.6,
      undefined,
      [
        {
          amount: Math.floor(400_000 / context.exchangeRate).toString(),
          denom: ldDenom,
        },
      ],
    );
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
      total_dasset_amount_to_withdraw: '400000',
      expected_native_asset_amount: '0',
      total_unbond_items: 1,
      unbonded_amount: null,
      withdrawn_amount: null,
    });
  });

  describe('state machine', () => {
    describe('prepare', () => {
      it('get machine state', async () => {
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('idle');
      });
    });
    describe('first cycle', () => {
      it('first tick did nothing and stays in idle', async () => {
        const res = await context.coreContractClient.tick(
          context.neutronUserAddress,
          1.5,
          undefined,
          [],
        );
        expect(res.transactionHash).toHaveLength(64);

        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('idle');
      });
      it('tick', async () => {
        const res = await context.coreContractClient.tick(
          context.neutronUserAddress,
          1.5,
          undefined,
          [],
        );
        expect(res.transactionHash).toHaveLength(64);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('peripheral');
        const nativeBondState =
          await context.nativeBondProviderContractClient.queryTxState();
        expect(nativeBondState).toEqual({
          status: 'waiting_for_ack',
          transaction: {
            i_b_c_transfer: {
              amount: '1000000',
              denom: 'untrn',
              real_amount: '1000000',
              reason: 'delegate',
              recipient: puppeteerIcaAddress,
            },
          },
        });
        await checkExchangeRate(context);
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
        expect(response).toBeTruthy();
        expect<ResponseHookMsg>(response).toHaveProperty('success');
      });
      it('next tick should go to idle', async () => {
        const { neutronUserAddress } = context;

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
        expect(state).toEqual('idle');
      });
      it('next tick should call delegation method on the bond provider', async () => {
        const { neutronUserAddress } = context;

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
        expect(state).toEqual('peripheral');
      });
      it('wait delegations', async () => {
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
              amount: String((800000 / 5) * 2 + 180000),
            },
            share_ratio: '1',
          },
          {
            delegator: puppeteerIcaAddress,
            validator: secondValidatorAddress,
            amount: {
              denom: context.park.config.networks.gaia.denom,
              amount: String((800000 / 5) * 3 + 20000),
            },
            share_ratio: '1',
          },
        ];
        expectedDelegations.sort((a, b) =>
          a.validator.localeCompare(b.validator),
        );
        expect(delegations).toEqual(expectedDelegations);
      });
      it('tick goes to idle', async () => {
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
        await checkExchangeRate(context);
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
      it('tick goes to claiming', async () => {
        const { neutronUserAddress } = context;

        const res = await context.coreContractClient.tick(
          neutronUserAddress,
          2,
          undefined,
          [],
        );
        expect(res.transactionHash).toHaveLength(64);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('claiming');
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
      it('tick goes to unbonding', async () => {
        const { neutronUserAddress } = context;

        const res = await context.coreContractClient.tick(
          neutronUserAddress,
          2,
          undefined,
          [],
        );
        expect(res.transactionHash).toHaveLength(64);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('unbonding');
        await checkExchangeRate(context);
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
        const { neutronUserAddress } = context;

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
      it('next tick goes to idle', async () => {
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
        await checkExchangeRate(context);
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
      it('tick to idle', async () => {
        const { coreContractClient, neutronUserAddress } = context;
        await coreContractClient.tick(neutronUserAddress, 1.5, undefined, []);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('idle');
        await checkExchangeRate(context);
      });
      it('tick to claiming', async () => {
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
        const { coreContractClient, neutronUserAddress } = context;

        await coreContractClient.tick(neutronUserAddress, 1.5, undefined, []);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('idle');
      });
      it('validate unbonding batch', async () => {
        const batch = await context.coreContractClient.queryUnbondBatch({
          batch_id: '0',
        });
        expect(batch).toEqual<UnbondBatch>({
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
        await waitFor(async () => {
          const balances =
            await context.neutronv2Client.CosmosBankV1Beta1.query.queryAllBalances(
              context.withdrawalManagerContractClient.contractAddress,
            );
          return balances.data.balances.length > 0;
        }, 40_000);
        await checkExchangeRate(context);
      });
      it('withdraw', async () => {
        const {
          withdrawalVoucherContractClient: voucherContractClient,
          neutronUserAddress,
          neutronClient,
        } = context;
        const balanceBefore = parseInt(
          (
            await neutronClient.CosmosBankV1Beta1.query.queryBalance(
              neutronUserAddress,
              { denom: 'untrn' },
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
            { denom: 'untrn' },
          );
        expect(parseInt(balance.data.balance.amount) - balanceBefore).toBe(
          200000,
        );
        await checkExchangeRate(context);
      });
      it('withdraw to custom receiver', async () => {
        const {
          withdrawalVoucherContractClient: voucherContractClient,
          neutronUserAddress,
          junoSecondUserAddress: neutronSecondUserAddress,
          junoClient: neutronClient,
        } = context;
        const balanceBefore = parseInt(
          (
            await neutronClient.CosmosBankV1Beta1.query.queryBalance(
              neutronSecondUserAddress,
              { denom: 'untrn' },
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
            { denom: 'untrn' },
          );
        expect(parseInt(balance.data.balance.amount)).toBe(300000);
      });
    });

    describe('fifth cycle (unbond before delegation)', () => {
      describe('prepare', () => {
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
      });

      it('tick to claiming', async () => {
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
            return false;
          }
          if (!response || !('success' in response)) return false;
          return true;
        }, 30_000);
      });
      it('tick to idle', async () => {
        const { coreContractClient, neutronUserAddress } = context;

        await coreContractClient.tick(neutronUserAddress, 1.5, undefined, []);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('idle');
        await checkExchangeRate(context);
      });
      it('increase idle interval', async () => {
        const { factoryContractClient, neutronUserAddress } = context;
        const res = await factoryContractClient.updateConfig(
          neutronUserAddress,
          {
            core: {
              idle_min_interval: 120,
            },
          },
        );
        expect(res.transactionHash).toHaveLength(64);
      });
      it('bond and unbond ibc coins', async () => {
        const { coreContractClient, ldDenom, neutronUserAddress } = context;
        let res = await coreContractClient.bond(
          neutronUserAddress,
          {},
          1.6,
          undefined,
          [
            {
              amount: '1000000',
              denom: 'untrn',
            },
          ],
        );

        expect(res.transactionHash).toHaveLength(64);

        await coreContractClient.tick(neutronUserAddress, 1.5, undefined, [
          {
            amount: '1000000',
            denom: 'untrn',
          },
        ]);

        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('peripheral');

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
        await checkExchangeRate(context);
      });
    });
    describe('sixth stake rewards', () => {
      const rewardsPumpIcaBalance = 0;
      it('pump rewards', async () => {
        await waitFor(async () => {
          const balances =
            await context.neutronv2Client.CosmosBankV1Beta1.query.queryAllBalances(
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
              denom: 'untrn',
            },
          ],
          1.5,
        );
        expect(res.transactionHash).toHaveLength(64);
      });
      it('split it', async () => {
        const nativeBondProviderBalanceBefore = (
          await context.neutronv2Client.CosmosBankV1Beta1.query.queryBalance(
            context.nativeBondProviderContractClient.contractAddress,
            { denom: 'untrn' },
          )
        ).data.balance.amount;
        expect(parseInt(nativeBondProviderBalanceBefore, 10)).toEqual(0);

        const res = await context.splitterContractClient.distribute(
          context.neutronUserAddress,
          1.5,
          undefined,
        );
        expect(res.transactionHash).toHaveLength(64);
        const nativeBondProviderBalanceAfter = (
          await context.neutronv2Client.CosmosBankV1Beta1.query.queryBalance(
            context.nativeBondProviderContractClient.contractAddress,
            { denom: 'untrn' },
          )
        ).data.balance.amount;

        expect(parseInt(nativeBondProviderBalanceAfter, 10)).toEqual(10000);
      });
      it('puppteer account state after bond provider ibc transfer', async () => {
        await waitFor(async () => {
          const res =
            await context.nativeBondProviderContractClient.queryTxState();
          return res.status === 'idle';
        }, 80_000);
      });
    });
  });
});
