import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import {
  LidoAutoWithdrawer,
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
import { setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import {
  ResponseHookMsg,
  ResponseHookSuccessMsg,
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
const LidoAutoWithdrawerClass = LidoAutoWithdrawer.Client;

describe('Auto withdrawer', () => {
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
    autoWithdrawerContractClient?: InstanceType<typeof LidoAutoWithdrawerClass>;
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
    context.park = await setupPark(
      'autowithdrawer',
      ['neutron', 'gaia'],
      true,
      true,
      true,
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
        unbond_batch_switch_time: 240,
        unbonding_safe_period: 10,
        unbonding_period: 360,
        channel: 'channel-0',
        lsm_redeem_threshold: 10,
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
    context.ldDenom = `factory/${context.tokenContractAddress}/lido`;
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
  it('query exchange rate', async () => {
    const { coreContractClient } = context;
    context.exchangeRate = parseFloat(
      await coreContractClient.queryExchangeRate(),
    );
    expect(context.exchangeRate).toEqual(1);
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
        join(__dirname, '../../../artifacts/lido_auto_withdrawer.wasm'),
      ),
      1.5,
    );
    expect(res.codeId).toBeGreaterThan(0);
    const instantiateRes = await LidoAutoWithdrawer.Client.instantiate(
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
      'Lido-auto-withdrawer',
      [],
      'auto',
    );
    expect(instantiateRes.contractAddress).toHaveLength(66);
    context.autoWithdrawerContractClient = new LidoAutoWithdrawer.Client(
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
          'Lido-staking-pump',
          [],
          1.5,
        );
        expect(res.contractAddress).toHaveLength(66);
        context.pumpContractClient = new LidoPump.Client(
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
              pump_address: ica,
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
            return res[0].coins.length == 2;
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
    });

    describe('fourth cycle', () => {
      let previousResponse: ResponseHookSuccessMsg;

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
        if (batchInfo.expected_release > currentTime) {
          const diffMs = (batchInfo.expected_release - currentTime + 1) * 1000;
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
              (await context.puppeteerContractClient.queryExtention({
                msg: {
                  balances: {},
                },
              })) as any
            )[2] / 1e9,
          );
          return icaTs > batchInfo.expected_release;
        }, 50_000);
      });
      it('tick', async () => {
        const { coreContractClient, neutronUserAddress } = context;
        previousResponse = (
          (await coreContractClient.queryLastPuppeteerResponse()) as {
            success: ResponseHookSuccessMsg;
          }
        ).success;
        await coreContractClient.tick(neutronUserAddress, 1.5, undefined, []);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('claiming');
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
          return (
            (response as { success: ResponseHookSuccessMsg }).success
              .request_id > previousResponse.request_id
          );
        }, 30_000);
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
      it('tick', async () => {
        const { coreContractClient, neutronUserAddress } = context;
        await coreContractClient.tick(neutronUserAddress, 1.5, undefined, []);
        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('staking');
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
        });
      });
      it('withdraw', async () => {
        const {
          neutronUserAddress,
          neutronClient,
          neutronIBCDenom,
          autoWithdrawerContractClient,
        } = context;

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

        const balance =
          await neutronClient.CosmosBankV1Beta1.query.queryBalance(
            neutronUserAddress,
            { denom: neutronIBCDenom },
          );
        expect(parseInt(balance.data.balance.amount) - balanceBefore).toBe(
          20000,
        );

        const bondings = await autoWithdrawerContractClient.queryBondings({
          user: neutronUserAddress,
        });
        expect(bondings).toEqual({
          bondings: [],
          next_page_key: null,
        });
      });
    });
  });
});
