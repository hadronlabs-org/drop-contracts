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
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { awaitBlocks, setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import { UnbondBatch } from '../generated/contractLib/lidoCore';
import { stringToPath } from '@cosmjs/crypto';

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
    context.park = await setupPark('core', ['neutron', 'gaia'], true);
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
        idle_min_interval: 1,
        puppeteer_timeout: 60,
        unbond_batch_switch_time: 6000,
        unbonding_safe_period: 10,
        unbonding_period: 60,
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
      const res = await puppeteerContractClient.queryState();
      ica = res.ica;
      return !!res.ica;
    }, 50_000);
    expect(ica).toHaveLength(65);
    expect(ica.startsWith('cosmos')).toBeTruthy();
    context.icaAddress = ica;
  });
  it('register balance ICQ', async () => {
    const { puppeteerContractClient, neutronUserAddress } = context;
    const res = await puppeteerContractClient.registerBalanceQuery(
      neutronUserAddress,
      { denom: 'stake' },
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
    expect(context.exchangeRate).toBeGreaterThan(1);
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
      total_amount: '495049',
      expected_amount: '499999',
      unbond_items: [
        {
          amount: '495049',
          expected_amount: '499999',
          sender: neutronUserAddress,
        },
      ],
      unbonded_amount: null,
      withdrawed_amount: null,
    });
  });

  it('validate NFT', async () => {
    const { withdrawalVoucherContractClient, neutronUserAddress } = context;
    const vouchers = await withdrawalVoucherContractClient.queryTokens({
      owner: context.neutronUserAddress,
    });
    expect(vouchers.tokens.length).toBe(1);
    expect(vouchers.tokens[0]).toBe(`0_${neutronUserAddress}_1`);
    const tokenId = vouchers.tokens[0];
    const voucher = await withdrawalVoucherContractClient.queryNftInfo({
      token_id: tokenId,
    });
    expect(voucher).toBeTruthy();
    expect(voucher).toMatchObject({
      extension: {
        amount: '495049',
        attributes: [
          {
            display_type: null,
            trait_type: 'unbond_batch_id',
            value: '0',
          },
          {
            display_type: null,
            trait_type: 'received_amount',
            value: '495049',
          },
          {
            display_type: null,
            trait_type: 'expected_amount',
            value: '499999',
          },
          {
            display_type: null,
            trait_type: 'exchange_rate',
            value: '1.01',
          },
        ],
        batch_id: '0',
        description: 'Withdrawal voucher',
        expected_amount: '499999',
        name: 'LDV voucher',
      },
      token_uri: null,
    });
  });

  it('try to withdraw before unbonded', async () => {
    const { withdrawalVoucherContractClient, neutronUserAddress } = context;
    const tokenId = `0_${neutronUserAddress}_1`;
    await expect(
      withdrawalVoucherContractClient.sendNft(neutronUserAddress, {
        token_id: tokenId,
        contract: context.withdrawalManagerContractClient.contractAddress,
        msg: Buffer.from('{}').toString('base64'),
      }),
    ).rejects.toThrowError(/is not unbonded yet/);
  });

  it('update batch status', async () => {
    const { coreContractClient, neutronUserAddress } = context;
    await coreContractClient.fakeProcessBatch(neutronUserAddress, {
      batch_id: '0',
      unbonded_amount: '499999',
    });
  });

  it('validate unbonding batch', async () => {
    const { coreContractClient, neutronUserAddress } = context;
    const batch = await coreContractClient.queryUnbondBatch({
      batch_id: '0',
    });
    expect(batch).toBeTruthy();
    expect(batch).toEqual<UnbondBatch>({
      slashing_effect: '1',
      status: 'unbonded',
      created: expect.any(Number),
      expected_release: 0,
      total_amount: '495049',
      expected_amount: '499999',
      unbond_items: [
        {
          amount: '495049',
          expected_amount: '499999',
          sender: neutronUserAddress,
        },
      ],
      unbonded_amount: '499999',
      withdrawed_amount: null,
    });
  });

  it('withdraw win non funded withdrawal manager', async () => {
    const {
      withdrawalVoucherContractClient: voucherContractClient,
      neutronUserAddress,
    } = context;
    const tokenId = `0_${neutronUserAddress}_1`;
    await expect(
      voucherContractClient.sendNft(neutronUserAddress, {
        token_id: tokenId,
        contract: context.withdrawalManagerContractClient.contractAddress,
        msg: Buffer.from('{}').toString('base64'),
      }),
    ).rejects.toThrowError(/spendable balance {2}is smaller/);
  });

  it('fund withdrawal manager', async () => {
    const {
      withdrawalManagerContractClient,
      neutronUserAddress,
      neutronIBCDenom,
    } = context;
    const res = await context.client.sendTokens(
      neutronUserAddress,
      withdrawalManagerContractClient.contractAddress,
      [{ amount: '500000', denom: neutronIBCDenom }],
      1.6,
      undefined,
    );
    expect(res.code).toEqual(0);
  });

  it('withdraw', async () => {
    const {
      withdrawalVoucherContractClient: voucherContractClient,
      neutronUserAddress,
      neutronClient,
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
      msg: Buffer.from('{}').toString('base64'),
    });
    expect(res.transactionHash).toHaveLength(64);
    const balance = await neutronClient.CosmosBankV1Beta1.query.queryBalance(
      neutronUserAddress,
      { denom: neutronIBCDenom },
    );
    expect(parseInt(balance.data.balance.amount) - balanceBefore).toBe(499999);
  });
});
