import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import {
  LidoCore,
  LidoFactory,
  LidoPump,
  LidoPuppeteer,
  LidoStrategy,
  LidoWithdrawalManager,
  LidoWithdrawalVoucher,
} from '../generated/contractLib';
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
  Coin,
  DirectSecp256k1HdWallet,
} from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { awaitBlocks, setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import {
  ResponseHookMsg,
  UnbondBatch,
} from '../generated/contractLib/lidoCore';
import { stringToPath } from '@cosmjs/crypto';
import { sleep } from '../helpers/sleep';

const LidoFactoryClass = LidoFactory.Client;
const LidoCoreClass = LidoCore.Client;
const LidoPumpClass = LidoPump.Client;
const LidoPuppeteerClass = LidoPuppeteer.Client;
const LidoStrategyClass = LidoStrategy.Client;
const LidoWithdrawalVoucherClass = LidoWithdrawalVoucher.Client;
const LidoWithdrawalManagerClass = LidoWithdrawalManager.Client;

describe('Core', () => {
  const context: {
    park?: Cosmopark;
    contractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    gaiaWallet2?: DirectSecp256k1HdWallet;
    factoryContractClient?: InstanceType<typeof LidoFactoryClass>;
    coreContractClient?: InstanceType<typeof LidoCoreClass>;
    strategyContractClient?: InstanceType<typeof LidoStrategyClass>;
    pumpContractClient?: InstanceType<typeof LidoPumpClass>;
    puppeteerContractClient?: InstanceType<typeof LidoPuppeteerClass>;
    withdrawalVoucherContractClient?: InstanceType<
      typeof LidoWithdrawalVoucherClass
    >;
    withdrawalManagerContractClient?: InstanceType<
      typeof LidoWithdrawalManagerClass
    >;
    account?: AccountData;
    icaAddress?: string;
    client?: SigningCosmWasmClient;
    gaiaClient?: SigningStargateClient;
    gaiaUserAddress?: string;
    gaiaUserAddress2?: string;
    gaiaQueryClient?: QueryClient & StakingExtension & BankExtension;
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
      validatorsSet?: number;
      distribution?: number;
      rewardsManager?: number;
    };
    exchangeRate?: number;
    tokenContractAddress?: string;
    neutronIBCDenom?: string;
    ldDenom?: string;
  } = { codeIds: {} };

  beforeAll(async () => {
    context.park = await setupPark('corefsm', ['neutron', 'gaia'], true, true);
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
    context.client = await SigningCosmWasmClient.connectWithSigner(
      `http://127.0.0.1:${context.park.ports.neutron.rpc}`,
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

  it('instantiate', async () => {
    const { client, account } = context;
    context.codeIds = {};

    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(join(__dirname, '../../../artifacts/lido_core.wasm')),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.core = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(join(__dirname, '../../../artifacts/lido_token.wasm')),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.token = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/lido_withdrawal_voucher.wasm'),
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
          join(__dirname, '../../../artifacts/lido_withdrawal_manager.wasm'),
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
          join(__dirname, '../../../artifacts/lido_strategy.wasm'),
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
          join(__dirname, '../../../artifacts/lido_distribution.wasm'),
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
          join(__dirname, '../../../artifacts/lido_validators_set.wasm'),
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
          join(__dirname, '../../../artifacts/lido_puppeteer.wasm'),
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
          join(__dirname, '../../../artifacts/lido_rewards_manager.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.rewardsManager = res.codeId;
    }

    const res = await client.upload(
      account.address,
      fs.readFileSync(join(__dirname, '../../../artifacts/lido_factory.wasm')),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await LidoFactory.Client.instantiate(
      client,
      account.address,
      res.codeId,
      {
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
        },
        remote_opts: {
          connection_id: 'connection-0',
          transfer_channel_id: 'channel-0',
          port_id: 'transfer',
          denom: 'stake',
          update_period: 2,
        },
        salt: 'salt',
        subdenom: 'lido',
        token_metadata: {
          description: 'Lido token',
          display: 'lido',
          exponent: 6,
          name: 'Lido liquid staking token',
          symbol: 'LIDO',
          uri: null,
          uri_hash: null,
        },
      },
      'Lido-staking-factory',
      [],
      'auto',
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.contractAddress = instantiateRes.contractAddress;
    context.factoryContractClient = new LidoFactory.Client(
      client,
      context.contractAddress,
    );
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
  });

  it('transfer tokens to neutron', async () => {
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
    });
    expect(context.neutronIBCDenom).toBeTruthy();
  });

  it('init', async () => {
    const { factoryContractClient: contractClient } = context;
    const res = await contractClient.init(context.neutronUserAddress, {
      base_denom: context.neutronIBCDenom,
      core_params: {
        idle_min_interval: 60,
        puppeteer_timeout: 60,
        unbond_batch_switch_time: 6000,
        unbonding_safe_period: 10,
        unbonding_period: 60,
        channel: 'channel-0',
      },
    });
    expect(res.transactionHash).toHaveLength(64);
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
      'LIDO-staking-token',
    );
    const coreContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.core_contract,
      );
    expect(coreContractInfo.data.contract_info.label).toBe('LIDO-staking-core');
    const withdrawalVoucherContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.withdrawal_voucher_contract,
      );
    expect(withdrawalVoucherContractInfo.data.contract_info.label).toBe(
      'LIDO-staking-withdrawal-voucher',
    );
    const withdrawalManagerContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.withdrawal_manager_contract,
      );
    expect(withdrawalManagerContractInfo.data.contract_info.label).toBe(
      'LIDO-staking-withdrawal-manager',
    );
    const puppeteerContractInfo =
      await neutronClient.CosmwasmWasmV1.query.queryContractInfo(
        res.puppeteer_contract,
      );
    expect(puppeteerContractInfo.data.contract_info.label).toBe(
      'LIDO-staking-puppeteer',
    );
    context.coreContractClient = new LidoCore.Client(
      context.client,
      res.core_contract,
    );
    context.withdrawalVoucherContractClient = new LidoWithdrawalVoucher.Client(
      context.client,
      res.withdrawal_voucher_contract,
    );
    context.withdrawalManagerContractClient = new LidoWithdrawalManager.Client(
      context.client,
      res.withdrawal_manager_contract,
    );
    context.strategyContractClient = new LidoStrategy.Client(
      context.client,
      res.strategy_contract,
    );
    context.tokenContractAddress = res.token_contract;
    context.puppeteerContractClient = new LidoPuppeteer.Client(
      context.client,
      res.puppeteer_contract,
    );
  });

  it('set fees for puppeteer', async () => {
    const { neutronUserAddress, factoryContractClient: contractClient } =
      context;
    const res = await contractClient.updateConfig(neutronUserAddress, {
      puppeteer_fees: {
        timeout_fee: '10000',
        ack_fee: '10000',
        recv_fee: '0',
        register_fee: '1000000',
      },
    });
    expect(res.transactionHash).toHaveLength(64);
  });

  it('register ICA', async () => {
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
    expect(
      balances.data.balances.find((one) => one.denom.startsWith('factory')),
    ).toEqual({
      denom: `factory/${context.tokenContractAddress}/lido`,
      amount: String(Math.floor(500_000 / context.exchangeRate)),
    });
  });

  it('bond with receiver', async () => {
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
          amount: '500000',
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
      denom: `factory/${context.tokenContractAddress}/lido`,
      amount: String(Math.floor(500_000 / context.exchangeRate)),
    });
    context.ldDenom = ldBalance?.denom;
  });

  it('unbond', async () => {
    const { coreContractClient, neutronUserAddress, ldDenom } = context;
    const res = await coreContractClient.unbond(
      neutronUserAddress,
      1.6,
      undefined,
      [
        {
          amount: Math.floor(500_000 / context.exchangeRate).toString(),
          denom: ldDenom,
        },
      ],
    );
    expect(res.transactionHash).toHaveLength(64);
  });

  it('validate unbonding batch', async () => {
    const { coreContractClient, neutronUserAddress } = context;
    const batch = await coreContractClient.queryUnbondBatch({
      batch_id: '0',
    });
    expect(batch).toBeTruthy();
    expect(batch).toEqual<UnbondBatch>({
      slashing_effect: null,
      created: expect.any(Number),
      expected_release: 0,
      status: 'new',
      total_amount: '500000',
      expected_amount: '500000',
      unbond_items: [
        {
          amount: '500000',
          expected_amount: '500000',
          sender: neutronUserAddress,
        },
      ],
      unbonded_amount: null,
      withdrawed_amount: null,
    });
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
          fs.readFileSync(join(__dirname, '../../../artifacts/lido_pump.wasm')),
          1.5,
        );
        expect(resUpload.codeId).toBeGreaterThan(0);
        const { codeId } = resUpload;
        const res = await LidoPump.Client.instantiate(
          client,
          neutronUserAddress,
          codeId,
          {
            connection_id: 'connection-0',
            ibc_fees: {
              timeout_fee: '10000',
              ack_fee: '10000',
              recv_fee: '0',
              register_fee: '1000000',
            },
            local_denom: 'stake',
            timeout: {
              local: 60,
              remote: 60,
            },
          },
          'Lido-staking-pump',
          [],
          1.5,
        );
        expect(res.contractAddress).toHaveLength(66);
        context.pumpContractClient = new LidoPump.Client(
          client,
          res.contractAddress,
        );
        const resFactory = await context.factoryContractClient.updateConfig(
          neutronUserAddress,
          {
            core: {
              pump_address: context.pumpContractClient.contractAddress,
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
      it('tick', async () => {
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
        expect(state).toEqual('transfering');
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
        ).rejects.toThrowError(/Puppeteer response is not received/);
      });
      it('state of fsm is transfering', async () => {
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('transfering');
      });
      it('wait for response from puppeteer', async () => {
        let response;
        await waitFor(async () => {
          try {
            response =
              await context.coreContractClient.queryLastPuppeteerResponse();
          } catch (e) {
            //
          }
          return !!response;
        }, 100_000);
      });
      it('get ICA increased balance', async () => {
        const { gaiaClient } = context;
        const res = await gaiaClient.getBalance(context.icaAddress, 'stake');
        const balance = parseInt(res.amount);
        expect(balance - 1000000).toEqual(ica.balance);
        ica.balance = balance;
      });
      it('wait for balances to come', async () => {
        let res;
        await waitFor(async () => {
          try {
            res = await context.puppeteerContractClient.queryExtention({
              msg: {
                balances: {},
              },
            });
          } catch (e) {
            //
          }
          return res && res[0].coins.length !== 0;
        }, 100_000);
      });
      it('second tick goes to staking', async () => {
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
        expect(state).toEqual('staking');
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
        ).rejects.toThrowError(/Puppeteer response is not received/);
      });
      it('wait for response from puppeteer', async () => {
        let response;
        await waitFor(async () => {
          try {
            response =
              await context.coreContractClient.queryLastPuppeteerResponse();
          } catch (e) {
            //
          }
          return !!response;
        }, 100_000);
      });
      it('query strategy contract to see delegations', async () => {
        await waitFor(async () => {
          try {
            await context.strategyContractClient.queryCalcWithdraw({
              withdraw: '500000',
            });
            return true;
          } catch (e) {
            return false;
          }
        }, 100_000);
      });
      it('third tick goes to unbonding', async () => {
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
        expect(state).toEqual('unbonding');
      });
      it('third tick is failed bc no response from puppeteer yet', async () => {
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
      it('query unbonding batch', async () => {
        const batch = await context.coreContractClient.queryUnbondBatch({
          batch_id: '0',
        });
        expect(batch).toBeTruthy();
        expect(batch).toEqual<UnbondBatch>({
          slashing_effect: null,
          status: 'unbond_requested',
          created: expect.any(Number),
          expected_release: 0,
          total_amount: '500000',
          expected_amount: '500000',
          unbond_items: [
            {
              amount: '500000',
              expected_amount: '500000',
              sender: context.neutronUserAddress,
            },
          ],
          unbonded_amount: null,
          withdrawed_amount: null,
        });
      });
      it('wait for response from puppeteer', async () => {
        let response;
        await waitFor(async () => {
          try {
            response =
              await context.coreContractClient.queryLastPuppeteerResponse();
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
      });
      it('verify that unbonding batch is in unbonding state', async () => {
        const batch = await context.coreContractClient.queryUnbondBatch({
          batch_id: '0',
        });
        expect(batch).toBeTruthy();
        expect(batch).toEqual<UnbondBatch>({
          slashing_effect: null,
          status: 'unbonding',
          created: expect.any(Number),
          expected_release: expect.any(Number),
          total_amount: '500000',
          expected_amount: '500000',
          unbond_items: [
            {
              amount: '500000',
              expected_amount: '500000',
              sender: context.neutronUserAddress,
            },
          ],
          unbonded_amount: null,
          withdrawed_amount: null,
        });
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
      });
      it('wait for response from puppeteer', async () => {
        let response;
        await waitFor(async () => {
          try {
            response =
              await context.coreContractClient.queryLastPuppeteerResponse();
          } catch (e) {
            //
          }
          return !!response;
        }, 100_000);
      });
      it('get ICA balance', async () => {
        const { gaiaClient } = context;
        const res = await gaiaClient.getBalance(context.icaAddress, 'stake');
        const newBalance = parseInt(res.amount);
        expect(newBalance).toBeGreaterThan(balance);
      });
      it('wait for balance to update', async () => {
        const [, currentHeight] =
          (await context.puppeteerContractClient.queryExtention({
            msg: {
              balances: {},
            },
          })) as any;
        await waitFor(async () => {
          const [, nowHeight] =
            (await context.puppeteerContractClient.queryExtention({
              msg: {
                balances: {},
              },
            })) as any;
          return nowHeight !== currentHeight;
        }, 30_000);
      });
      it('next tick goes to staking', async () => {
        const { neutronUserAddress } = context;
        const res = await context.coreContractClient.tick(
          neutronUserAddress,
          1.5,
          undefined,
          [],
        );
        expect(res.transactionHash).toHaveLength(64);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('staking');
      });
      it('wait for response from puppeteer', async () => {
        let response;
        await waitFor(async () => {
          try {
            response =
              await context.coreContractClient.queryLastPuppeteerResponse();
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
      });
    });
    describe('third cycle', () => {
      let remoteNonNativeDenoms: string[] = [];
      it('generate two new tokenfactory tokens and send them to the remote zone', async () => {
        const { neutronUserAddress } = context;
        await context.park.executeInNetwork(
          'neutron',
          `neutrond tx tokenfactory create-denom test1 --from ${neutronUserAddress} --yes --chain-id ntrntest  --gas auto --gas-adjustment 1.6 --fees 10000untrn --home=/opt --keyring-backend=test --output json`,
        );
        await sleep(5_000);
        await context.park.executeInNetwork(
          'neutron',
          `neutrond tx tokenfactory create-denom test2 --from ${neutronUserAddress} --yes --chain-id ntrntest  --gas auto --gas-adjustment 1.6 --fees 10000untrn --home=/opt --keyring-backend=test --output json`,
        );
        await sleep(5_000);
        const denoms =
          await context.neutronClient.OsmosisTokenfactoryV1Beta1.query.queryDenomsFromCreator(
            neutronUserAddress,
          );
        expect(denoms.data.denoms.length).toEqual(2);
        await context.park.executeInNetwork(
          'neutron',
          `neutrond tx tokenfactory mint 1000000${denoms.data.denoms[0]} --from ${neutronUserAddress} --yes --chain-id ntrntest  --gas auto --gas-adjustment 1.6 --fees 10000untrn --home=/opt --keyring-backend=test --output json`,
        );
        await sleep(5_000);
        await context.park.executeInNetwork(
          'neutron',
          `neutrond tx tokenfactory mint 1000000${denoms.data.denoms[1]} --from ${neutronUserAddress} --yes --chain-id ntrntest  --gas auto --gas-adjustment 1.6 --fees 10000untrn --home=/opt --keyring-backend=test --output json`,
        );
        await sleep(5_000);
        const balances =
          await context.neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
            neutronUserAddress,
          );
        const tokenFactoryDenoms = balances.data.balances.filter((b) =>
          b.denom.startsWith('factory/'),
        );
        await context.park.executeInNetwork(
          'neutron',
          `neutrond tx ibc-transfer transfer transfer channel-0 ${context.icaAddress} 66666${tokenFactoryDenoms[0].denom} --from ${neutronUserAddress} --yes --chain-id ntrntest  --gas auto --gas-adjustment 1.6 --fees 10000untrn --home=/opt --keyring-backend=test --output json`,
        );
        await sleep(5_000);
        await context.park.executeInNetwork(
          'neutron',
          `neutrond tx ibc-transfer transfer transfer channel-0 ${context.icaAddress} 2222${tokenFactoryDenoms[1].denom} --from ${neutronUserAddress} --yes --chain-id ntrntest  --gas auto --gas-adjustment 1.6 --fees 10000untrn --home=/opt --keyring-backend=test --output json`,
        );
        await sleep(5_000);
      });
      it('wait for balances to come', async () => {
        let res: readonly Coin[] = [];
        await waitFor(async () => {
          res = await context.gaiaClient.getAllBalances(context.icaAddress);
          return (
            res.some((b) => b.amount === '66666') &&
            res.some((b) => b.amount === '2222')
          );
        }, 30_000);
        remoteNonNativeDenoms = [
          res.find((b) => b.amount === '66666').denom,
          res.find((b) => b.amount === '2222').denom,
        ];
      });
      it('setup non-native receivers', async () => {
        const { factoryContractClient, neutronUserAddress } = context;
        const res = await factoryContractClient.proxy(
          neutronUserAddress,
          {
            core: {
              update_non_native_rewards_receivers: {
                items: remoteNonNativeDenoms.map((denom) => ({
                  denom,
                  address: context.gaiaUserAddress,
                  min_amount: '10000',
                  fee: '0.1',
                  fee_address: context.gaiaUserAddress2,
                })),
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
      it('wait for non-native balances to come', async () => {
        await waitFor(async () => {
          try {
            const res: any =
              await context.puppeteerContractClient.queryExtention({
                msg: {
                  non_native_rewards_balances: {},
                },
              });
            return res.length == 2;
          } catch (e) {
            //
          }
        });
      });
      it('tick', async () => {
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
      it('wait for the response from puppeteer', async () => {
        let response: ResponseHookMsg;
        await waitFor(async () => {
          try {
            response =
              await context.coreContractClient.queryLastPuppeteerResponse();
          } catch (e) {
            //
          }
          return !!response;
        }, 30_000);
        expect(response).toBeTruthy();
        expect<ResponseHookMsg>(response).toHaveProperty('success');
      });
      it('check balances', async () => {
        const { gaiaClient } = context;
        const receiverBalance = await gaiaClient.getBalance(
          context.gaiaUserAddress,
          remoteNonNativeDenoms[0],
        );
        expect(receiverBalance.amount).toEqual('60000');
        const feeBalance = await gaiaClient.getBalance(
          context.gaiaUserAddress2,
          remoteNonNativeDenoms[0],
        );
        expect(feeBalance.amount).toEqual('6666');
        // this one is still on ICA as amount is below min_amount
        const icaBalance = await gaiaClient.getBalance(
          context.icaAddress,
          remoteNonNativeDenoms[1],
        );
        expect(icaBalance.amount).toEqual('2222');
      });
      it('wait for balances to update', async () => {
        await waitFor(async () => {
          const res: any = await context.puppeteerContractClient.queryExtention(
            {
              msg: {
                non_native_rewards_balances: {},
              },
            },
          );
          return res[0].coins.length === 1;
        });
      }, 30_000);
      it('tick should fail', async () => {
        const { neutronUserAddress } = context;
        await expect(
          context.coreContractClient.tick(
            neutronUserAddress,
            1.5,
            undefined,
            [],
          ),
        ).rejects.toThrowError(/Idle min interval is not reached/);
      });
    });
  });
});
