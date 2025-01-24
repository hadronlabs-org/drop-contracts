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
import { instrumentCoreClass } from '../helpers/knot';
import { UnbondBatch } from 'drop-ts-client/lib/contractLib/dropCore';
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
          denom: 'untrn',
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
        const nonStakedBalance =
          await context.nativeBondProviderContractClient.queryNonStakedBalance();
        expect(nonStakedBalance).toEqual('0');
        expect(nativeBondState).toEqual({
          status: 'idle',
          transaction: null,
        });
        await checkExchangeRate(context);
      });

      it('verify delegations', async () => {
        const res = await context.park.executeInNetwork(
          'neutronv2',
          `neutrond q staking delegations ${context.puppeteerContractClient.contractAddress} --output json`,
        );
        const { delegation_responses } = JSON.parse(res.out);
        expect(delegation_responses).toHaveLength(2);
        expect(delegation_responses[0].balance.amount).toEqual('200000');
        expect(delegation_responses[1].balance.amount).toEqual('200000');
      });
    });
  });
});
