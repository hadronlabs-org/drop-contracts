import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import {
  DropCore,
  DropFactoryNative,
  DropPump,
  DropPuppeteerNative,
  DropStrategy,
  DropWithdrawalManager,
  DropWithdrawalVoucher,
  DropRewardsManager,
  DropSplitter,
  DropToken,
  DropNativeSimpleBondProvider,
  DropValRef,
  DropValidatorsSet,
} from 'drop-ts-client';
import {
  QueryClient,
  StakingExtension,
  BankExtension,
  setupStakingExtension,
  setupBankExtension,
} from '@cosmjs/stargate';
import { join } from 'path';
import { Tendermint34Client } from '@cosmjs/tendermint-rpc';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { awaitBlocks, setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import {
  ResponseHookMsg,
  UnbondBatch,
} from 'drop-ts-client/lib/contractLib/dropCore';
import { sleep } from '../helpers/sleep';
import { waitForTx } from '../helpers/waitForTx';
import { waitForPuppeteerICQ } from '../helpers/waitForPuppeteerICQ';
import { instrumentCoreClass } from '../helpers/knot';
import { checkExchangeRate } from '../helpers/exchangeRate';

const DropTokenClass = DropToken.Client;
const DropFactoryNativeClass = DropFactoryNative.Client;
const DropCoreClass = DropCore.Client;
const DropPumpClass = DropPump.Client;
const DropPuppeteerNativeClass = DropPuppeteerNative.Client;
const DropStrategyClass = DropStrategy.Client;
const DropWithdrawalVoucherClass = DropWithdrawalVoucher.Client;
const DropWithdrawalManagerClass = DropWithdrawalManager.Client;
const DropRewardsManagerClass = DropRewardsManager.Client;
const DropRewardsPumpClass = DropPump.Client;
const DropSplitterClass = DropSplitter.Client;
const DropNativeSimpleBondProviderClass = DropNativeSimpleBondProvider.Client;
const DropValRefClass = DropValRef.Client;
const DropValidatorsSetClass = DropValidatorsSet.Client;

const UNBONDING_TIME = 360;

describe('Core', () => {
  const context: {
    park?: Cosmopark;
    wallet?: DirectSecp256k1HdWallet;
    factoryContractClient?: InstanceType<typeof DropFactoryNativeClass>;
    coreContractClient?: InstanceType<typeof DropCoreClass>;
    strategyContractClient?: InstanceType<typeof DropStrategyClass>;
    pumpContractClient?: InstanceType<typeof DropPumpClass>;
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
    rewardsPumpContractClient?: InstanceType<typeof DropRewardsPumpClass>;
    nativeBondProviderContractClient?: InstanceType<
      typeof DropNativeSimpleBondProviderClass
    >;
    valRefClient?: InstanceType<typeof DropValRefClass>;
    validatorsSetClient?: InstanceType<typeof DropValidatorsSetClass>;
    account?: AccountData;
    puppeteerIcaAddress?: string;
    rewardsPumpIcaAddress?: string;
    client?: SigningCosmWasmClient;
    queryClient?: QueryClient & StakingExtension & BankExtension;
    junoRPCEndpoint?: string;
    junoUserAddress?: string;
    junoSecondUserAddress?: string;
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
    };
    exchangeRate?: number;
  } = { codeIds: {} };

  beforeAll(async (t) => {
    context.park = await setupPark(t, ['juno'], {
      juno: {
        genesis_opts: {
          'app_state.staking.params.unbonding_time': `${UNBONDING_TIME}s`,
        },
      },
    });

    context.wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demowallet1.mnemonic,
      {
        prefix: 'juno',
      },
    );

    context.account = (await context.wallet.getAccounts())[0];

    context.junoRPCEndpoint = `http://127.0.0.1:${context.park.ports.juno.rpc}`;
    context.client = await SigningCosmWasmClient.connectWithSigner(
      context.junoRPCEndpoint,
      context.wallet,
      {
        gasPrice: GasPrice.fromString('0.025untrn'),
      },
    );
    const tmClient = await Tendermint34Client.connect(
      `http://127.0.0.1:${context.park.ports.juno.rpc}`,
    );
    context.queryClient = QueryClient.withExtensions(
      tmClient,
      setupStakingExtension,
      setupBankExtension,
    );
    const secondWallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demo2.mnemonic,
      {
        prefix: 'juno',
      },
    );
    context.junoSecondUserAddress = (
      await secondWallet.getAccounts()
    )[0].address;
  });

  afterAll(async () => {
    await context.park.stop();
  });

  it('instantiate', async () => {
    const { client, account } = context;
    context.codeIds = {};

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
          '../../../artifacts/drop_native_simple_bond_provider.wasm',
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

    const buffer = fs.readFileSync(
      join(__dirname, '../../../artifacts/drop_factory_native.wasm'),
    );

    const res = await client.upload(
      account.address,
      new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await DropFactoryNative.Client.instantiate(
      client,
      account.address,
      res.codeId,
      {
        local_denom: 'ujunox',
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
          transfer_channel_id: 'channel-0',
          reverse_transfer_channel_id: 'channel-0',
          port_id: 'transfer',
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
        base_denom: 'ujunox',
        core_params: {
          idle_min_interval: 120,
          unbond_batch_switch_time: 60,
          unbonding_safe_period: 10,
          unbonding_period: 360,
          bond_limit: '0s',
          icq_update_delay: 5,
        },
        native_bond_params: {
          min_stake_amount: '10000',
          min_ibc_transfer: '10000',
        },
      },
      'drop-staking-factory',
      'auto',
      [],
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.factoryContractClient = new DropFactoryNative.Client(
      client,
      instantiateRes.contractAddress,
    );
  });

  it('query factory state', async () => {
    const { factoryContractClient: contractClient, client: neutronClient } =
      context;
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
    context.puppeteerContractClient = new DropPuppeteerNative.Client(
      context.client,
      res.puppeteer_contract,
    );
    context.splitterContractClient = new DropSplitter.Client(
      context.client,
      res.splitter_contract,
    );
    context.nativeBondProviderContractClient =
      new DropNativeSimpleBondProvider.Client(
        context.client,
        res.native_bond_provider_contract,
      );
    context.validatorsSetClient = new DropValidatorsSet.Client(
      context.client,
      res.validators_set_contract,
    );
  });

  it('set up rewards receiver', async () => {
    const { junoUserAddress: neutronUserAddress } = context;
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
      junoUserAddress: neutronUserAddress,
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
      context.junoUserAddress,
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
      junoClient: neutronClient,
      junoUserAddress: neutronUserAddress,
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

    await awaitBlocks(`http://127.0.0.1:${context.park.ports.gaia.rpc}`, 1);
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
  it('verify bonded amount', async () => {
    const { coreContractClient } = context;
    const bonded = await coreContractClient.queryTotalBonded();
    expect(bonded).toEqual('500000');
  });
  it('reset bonded amount', async () => {
    const { coreContractClient, junoUserAddress: neutronUserAddress } = context;
    const res = await context.factoryContractClient.adminExecute(
      neutronUserAddress,
      {
        msgs: [
          {
            wasm: {
              execute: {
                contract_addr: context.coreContractClient.contractAddress,
                msg: Buffer.from(
                  JSON.stringify({
                    reset_bonded_amount: {},
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
    expect(res.transactionHash).toHaveLength(64);
    const bonded = await coreContractClient.queryTotalBonded();
    expect(bonded).toEqual('0');
  });
  it('bond with receiver', async () => {
    const {
      coreContractClient,
      junoClient: neutronClient,
      junoUserAddress: neutronUserAddress,
      neutronIBCDenom,
      junoSecondUserAddress: neutronSecondUserAddress,
    } = context;
    const res = await coreContractClient.bond(
      neutronUserAddress,
      { receiver: neutronSecondUserAddress },
      1.6,
      undefined,
      [
        {
          amount: '400000',
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
    expect(ldBalance).toEqual({
      denom: `factory/${context.tokenContractClient.contractAddress}/drop`,
      amount: String(Math.floor(400_000 / context.exchangeRate)),
    });
    context.ldDenom = ldBalance?.denom;
    await checkExchangeRate(context);
  });

  it('unbond', async () => {
    const {
      coreContractClient,
      junoUserAddress: neutronUserAddress,
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
        const { gaiaClient } = context;
        const res = await gaiaClient.getBalance(
          context.puppeteerIcaAddress,
          context.park.config.networks.gaia.denom,
        );
        ica.balance = parseInt(res.amount);
        expect(ica.balance).toEqual(0);
      });
      it('deploy pump', async () => {
        const {
          client,
          account,
          junoUserAddress: neutronUserAddress,
        } = context;

        const buffer = fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_pump.wasm'),
        );
        const resUpload = await client.upload(
          account.address,
          new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.byteLength),
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
      it('first tick did nothing and stays in idle', async () => {
        const {
          gaiaClient,
          junoUserAddress: neutronUserAddress,
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
        expect(state).toEqual('idle');
      });
      it('tick', async () => {
        const {
          gaiaClient,
          junoUserAddress: neutronUserAddress,
          coreContractClient,
          puppeteerContractClient,
          neutronIBCDenom,
          puppeteerIcaAddress,
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
        expect(state).toEqual('peripheral');
        const nativeBondState =
          await context.nativeBondProviderContractClient.queryTxState();
        expect(nativeBondState).toEqual({
          status: 'waiting_for_ack',
          transaction: {
            i_b_c_transfer: {
              amount: '1000000',
              denom: neutronIBCDenom,
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
        const {
          gaiaClient,
          junoUserAddress: neutronUserAddress,
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
        expect(state).toEqual('idle');
      });
      it('next tick should call delegation method on the bond provider', async () => {
        const {
          gaiaClient,
          junoUserAddress: neutronUserAddress,
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
        const {
          junoUserAddress: neutronUserAddress,
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
      it('decrease idle interval', async () => {
        const { factoryContractClient, junoUserAddress: neutronUserAddress } =
          context;
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
        const {
          junoUserAddress: neutronUserAddress,
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
        const { junoUserAddress: neutronUserAddress } = context;
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
        const {
          junoUserAddress: neutronUserAddress,
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
        const {
          junoUserAddress: neutronUserAddress,
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
      const balance = 0;
      it('idle tick', async () => {
        const {
          junoUserAddress: neutronUserAddress,
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
          junoUserAddress: neutronUserAddress,
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
    describe('third cycle (LSM-shares)', () => {
      let lsmDenoms: string[] = [];
      let oldBalanceDenoms: string[] = [];
      describe('prepare', () => {
        it('remove native bond provider from the core', async () => {
          const res = await context.factoryContractClient.adminExecute(
            context.junoUserAddress,
            {
              msgs: [
                {
                  wasm: {
                    execute: {
                      contract_addr: context.coreContractClient.contractAddress,
                      msg: Buffer.from(
                        JSON.stringify({
                          remove_bond_provider: {
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

        it('register lsm share bond provider in the core', async () => {
          const res = await context.factoryContractClient.adminExecute(
            context.junoUserAddress,
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
                              context.lsmShareBondProviderContractClient
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

        describe('create LSM shares and send them to neutron', () => {
          it('get balances', async () => {
            const oldBalances =
              await context.junoClient.CosmosBankV1Beta1.query.queryAllBalances(
                context.junoUserAddress,
              );
            oldBalanceDenoms = oldBalances.data.balances.map((b) => b.denom);
          });
          it('update idle interval', async () => {
            const {
              factoryContractClient,
              junoUserAddress: neutronUserAddress,
            } = context;
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
                `gaiad tx ibc-transfer transfer transfer channel-0 ${context.junoUserAddress} 60000${context.validatorAddress}/2 --from ${context.gaiaUserAddress}  --yes --chain-id testgaia --home=/opt --keyring-backend=test --gas auto --gas-adjustment 2 --output json`,
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
                `gaiad tx ibc-transfer transfer transfer channel-0 ${context.junoUserAddress} 60000${context.secondValidatorAddress}/3 --from ${context.gaiaUserAddress}  --yes --chain-id testgaia --home=/opt --keyring-backend=test --gas auto --gas-adjustment 2 --output json`,
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
                await context.junoClient.CosmosBankV1Beta1.query.queryAllBalances(
                  context.junoUserAddress,
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
            const { coreContractClient, junoUserAddress: neutronUserAddress } =
              context;
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
            await checkExchangeRate(context);
          }
          {
            const { coreContractClient, junoUserAddress: neutronUserAddress } =
              context;
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
            await checkExchangeRate(context);
          }
        });
        it('verify pending lsm shares', async () => {
          const pending =
            await context.lsmShareBondProviderContractClient.queryPendingLSMShares();
          expect(pending).toHaveLength(2);
        });
      });
      describe('transfering', () => {
        it('tick', async () => {
          const {
            junoUserAddress: neutronUserAddress,
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
          expect(state).toEqual('peripheral');
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
          }, 100_000);
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
        it('one lsm share is gone from the contract balance', async () => {
          const balances =
            await context.junoClient.CosmosBankV1Beta1.query.queryAllBalances(
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
                await context.lsmShareBondProviderContractClient.queryPendingLSMShares();
              pending = res;
            } catch (e) {
              //
            }
            return !!pending && pending.length === 1;
          }, 60_000);
        });
        it('tick to idle', async () => {
          const { junoUserAddress: neutronUserAddress } = context;
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
        it('tick to peripheral', async () => {
          const { junoUserAddress: neutronUserAddress } = context;
          const res = await context.coreContractClient.tick(
            neutronUserAddress,
            1.5,
            undefined,
            [],
          );
          expect(res.transactionHash).toHaveLength(64);
          const state = await context.coreContractClient.queryContractState();
          expect(state).toEqual('peripheral');
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
          }, 100_000);
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
        it('second lsm share is gone from the contract balance', async () => {
          const balances =
            await context.junoClient.CosmosBankV1Beta1.query.queryAllBalances(
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
                await context.lsmShareBondProviderContractClient.queryPendingLSMShares();
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
            await context.lsmShareBondProviderContractClient.queryLSMSharesToRedeem();
          expect(pending).toHaveLength(2);
        });
        it('tick to idle', async () => {
          const {
            gaiaClient,
            junoUserAddress: neutronUserAddress,
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
        it('tick to peripheral', async () => {
          const { junoUserAddress: neutronUserAddress } = context;
          const res = await context.coreContractClient.tick(
            neutronUserAddress,
            1.5,
            undefined,
            [],
          );
          expect(res.transactionHash).toHaveLength(64);
          const state = await context.coreContractClient.queryContractState();
          expect(state).toEqual('peripheral');
          await checkExchangeRate(context);
        });
        it('imeediately tick again fails', async () => {
          const { junoUserAddress: neutronUserAddress } = context;
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
              await context.lsmShareBondProviderContractClient.queryLSMSharesToRedeem();
            return pending.length === 0;
          }, 30_000);
        });
        it('wait for delegations to come', async () => {
          const { remote_height: currentHeight } =
            (await context.puppeteerContractClient.queryExtension({
              msg: {
                delegations: {},
              },
            })) as any;
          await waitFor(async () => {
            const { remote_height: nowHeight } =
              (await context.puppeteerContractClient.queryExtension({
                msg: {
                  delegations: {},
                },
              })) as any;
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
          await checkExchangeRate(context);
        });
      });
    });

    describe('forth cycle', () => {
      it('validate NFT', async () => {
        const {
          withdrawalVoucherContractClient,
          junoUserAddress: neutronUserAddress,
        } = context;
        const vouchers = await withdrawalVoucherContractClient.queryTokens({
          owner: context.junoUserAddress,
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
        const { coreContractClient, junoUserAddress: neutronUserAddress } =
          context;
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
        await checkExchangeRate(context);
      });
      it('try to withdraw from paused manager', async () => {
        const {
          withdrawalVoucherContractClient,
          junoUserAddress: neutronUserAddress,
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
        const {
          withdrawalVoucherContractClient,
          junoUserAddress: neutronUserAddress,
        } = context;
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
        const { factoryContractClient, junoUserAddress: neutronUserAddress } =
          context;
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
        const { coreContractClient, junoUserAddress: neutronUserAddress } =
          context;
        await coreContractClient.tick(neutronUserAddress, 1.5, undefined, []);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('idle');
        await checkExchangeRate(context);
      });
      it('tick to claiming', async () => {
        const {
          coreContractClient,
          junoUserAddress: neutronUserAddress,
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
        const {
          gaiaClient,
          coreContractClient,
          junoUserAddress: neutronUserAddress,
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
        await checkExchangeRate(context);
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
          junoUserAddress: neutronUserAddress,
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
        const { pumpContractClient, junoUserAddress: neutronUserAddress } =
          context;
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
            await context.junoClient.CosmosBankV1Beta1.query.queryAllBalances(
              context.withdrawalManagerContractClient.contractAddress,
            );
          return balances.data.balances.length > 0;
        }, 40_000);
        await checkExchangeRate(context);
      });
      it('withdraw', async () => {
        const {
          withdrawalVoucherContractClient: voucherContractClient,
          junoUserAddress: neutronUserAddress,
          junoClient: neutronClient,
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
        await checkExchangeRate(context);
      });
      it('withdraw to custom receiver', async () => {
        const {
          withdrawalVoucherContractClient: voucherContractClient,
          junoUserAddress: neutronUserAddress,
          junoSecondUserAddress: neutronSecondUserAddress,
          junoClient: neutronClient,
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
      describe('prepare', () => {
        it('remove lsm share bond provider from the core', async () => {
          const res = await context.factoryContractClient.adminExecute(
            context.junoUserAddress,
            {
              msgs: [
                {
                  wasm: {
                    execute: {
                      contract_addr: context.coreContractClient.contractAddress,
                      msg: Buffer.from(
                        JSON.stringify({
                          remove_bond_provider: {
                            bond_provider_address:
                              context.lsmShareBondProviderContractClient
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

        it('register native bond provider in the core', async () => {
          const res = await context.factoryContractClient.adminExecute(
            context.junoUserAddress,
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
        const {
          coreContractClient,
          junoUserAddress: neutronUserAddress,
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
        const {
          gaiaClient,
          coreContractClient,
          junoUserAddress: neutronUserAddress,
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
      it('increase idle interval', async () => {
        const { factoryContractClient, junoUserAddress: neutronUserAddress } =
          context;
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
        const {
          coreContractClient,
          ldDenom,
          junoUserAddress: neutronUserAddress,
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

        await coreContractClient.tick(neutronUserAddress, 1.5, undefined, [
          {
            amount: '1000000',
            denom: 'untrn',
          },
        ]);

        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('peripheral');

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
        await checkExchangeRate(context);
      });
    });
    describe('sixth stake rewards', () => {
      let rewardsPumpIcaBalance = 0;
      it('pump rewards', async () => {
        const {
          rewardsPumpContractClient,
          junoUserAddress: neutronUserAddress,
          gaiaClient,
        } = context;
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
            await context.junoClient.CosmosBankV1Beta1.query.queryAllBalances(
              context.splitterContractClient.contractAddress,
            );
          return balances.data.balances.length > 0;
        }, 60_000);
      });
      it('top up splitter', async () => {
        const res = await context.client.sendTokens(
          context.junoUserAddress,
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
        const nativeBondProviderBalanceBefore = (
          await context.junoClient.CosmosBankV1Beta1.query.queryBalance(
            context.nativeBondProviderContractClient.contractAddress,
            { denom: context.neutronIBCDenom },
          )
        ).data.balance.amount;
        expect(parseInt(nativeBondProviderBalanceBefore, 10)).toEqual(0);

        const res = await context.splitterContractClient.distribute(
          context.junoUserAddress,
          1.5,
          undefined,
        );
        expect(res.transactionHash).toHaveLength(64);
        const nativeBondProviderBalanceAfter = (
          await context.junoClient.CosmosBankV1Beta1.query.queryBalance(
            context.nativeBondProviderContractClient.contractAddress,
            { denom: context.neutronIBCDenom },
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
        const balances = await context.gaiaClient.getAllBalances(
          context.puppeteerIcaAddress,
        );
        expect(balances).toEqual([
          {
            amount: '1000000',
            denom: context.park.config.networks.gaia.denom,
          },
        ]);
      });
    });
  });

  it('update validators set and check kv queries id', async () => {
    const {
      junoUserAddress: neutronUserAddress,
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

    const queryIdsNew = await context.puppeteerContractClient.queryKVQueryIds();
    expect(queryIdsNew).toEqual([[2, 'delegations_and_balance']]);
  });
});
