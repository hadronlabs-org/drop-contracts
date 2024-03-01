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
  IndexedTx,
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
const LidoAutoWithdrawerClass = LidoAutoWithdrawer.Client;

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
    autoWithdrawerContractClient?: InstanceType<typeof LidoAutoWithdrawerClass>;
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
        channel: 'channel-0',
        bond_limit: '100000',
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
    const { neutronUserAddress, factoryContractClient } = context;
    const res = await factoryContractClient.updateConfig(neutronUserAddress, {
      puppeteer_fees: {
        timeout_fee: '20000',
        ack_fee: '10000',
        recv_fee: '0',
        register_fee: '1000000',
      },
    });
    expect(res.transactionHash).toHaveLength(64);
  });
  it('update by factory admin execute', async () => {
    const { neutronUserAddress, factoryContractClient: contractClient } =
      context;
    const res = await contractClient.adminExecute(
      neutronUserAddress,
      {
        addr: context.puppeteerContractClient.contractAddress,
        msg: Buffer.from(
          JSON.stringify({
            set_fees: {
              timeout_fee: '10000',
              ack_fee: '10000',
              recv_fee: '0',
              register_fee: '1000000',
            },
          }),
        ).toString('base64'),
      },
      1.5,
    );
    expect(res.transactionHash).toHaveLength(64);
    const fees: any = await context.puppeteerContractClient.queryExtention({
      msg: { fees: {} },
    });
    expect(fees).toEqual({
      recv_fee: [{ denom: 'untrn', amount: '0' }],
      ack_fee: [{ denom: 'untrn', amount: '10000' }],
      timeout_fee: [{ denom: 'untrn', amount: '10000' }],
      register_fee: { denom: 'untrn', amount: '1000000' },
    });
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
  it('register balance ICQ', async () => {
    const { factoryContractClient, neutronUserAddress } = context;
    const res = await factoryContractClient.proxy(
      neutronUserAddress,
      {
        validator_set: {
          update_validators: { validators: [] },
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

  it('bond failed as over limit', async () => {
    const { coreContractClient, neutronUserAddress, neutronIBCDenom } = context;
    await expect(
      coreContractClient.bond(neutronUserAddress, {}, 1.6, undefined, [
        {
          amount: '500000',
          denom: neutronIBCDenom,
        },
      ]),
    ).rejects.toThrowError(/Bond limit exceeded/);
  });

  it('update limit', async () => {
    const { factoryContractClient, neutronUserAddress } = context;
    const res = await factoryContractClient.adminExecute(neutronUserAddress, {
      addr: context.coreContractClient.contractAddress,
      msg: Buffer.from(
        JSON.stringify({
          update_config: {
            new_config: {
              bond_limit: '0',
            },
          },
        }),
      ).toString('base64'),
    });
    expect(res.transactionHash).toHaveLength(64);
    const config = await context.coreContractClient.queryConfig();
    expect(config.bond_limit).toBe(null);
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
  it('verify bonded amount', async () => {
    const { coreContractClient } = context;
    const bonded = await coreContractClient.queryTotalBonded();
    expect(bonded).toEqual('500000');
  });
  it('reset bonded amount', async () => {
    const { coreContractClient, neutronUserAddress } = context;
    const res = await context.factoryContractClient.adminExecute(
      neutronUserAddress,
      {
        addr: context.coreContractClient.contractAddress,
        msg: Buffer.from(
          JSON.stringify({
            reset_bonded_amount: {},
          }),
        ).toString('base64'),
      },
    );
    expect(res.transactionHash).toHaveLength(64);
    const bonded = await coreContractClient.queryTotalBonded();
    expect(bonded).toEqual('0');
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

  it('delegate tokens on gaia side', async () => {
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.master_mnemonic,
      {
        prefix: 'cosmosvaloper',
        hdPaths: [stringToPath("m/44'/118'/1'/0/0") as any],
      },
    );
    context.validatorAddress = (await wallet.getAccounts())[0].address;
    const res = await context.park.executeInNetwork(
      'gaia',
      `gaiad tx staking delegate ${context.validatorAddress} 1000000stake --from ${context.gaiaUserAddress} --yes --chain-id testgaia --home=/opt --keyring-backend=test --output json`,
    );
    expect(res.exitCode).toBe(0);
    const out = JSON.parse(res.out);
    expect(out.code).toBe(0);
    expect(out.txhash).toHaveLength(64);
    let tx: IndexedTx | null = null;
    await waitFor(async () => {
      tx = await context.gaiaClient.getTx(out.txhash);
      return tx !== null;
    });
    expect(tx.height).toBeGreaterThan(0);
    expect(tx.code).toBe(0);
  });
  it('tokenize share on gaia side', async () => {
    const res = await context.park.executeInNetwork(
      'gaia',
      `gaiad tx staking tokenize-share ${context.validatorAddress} 600000stake ${context.gaiaUserAddress} --from ${context.gaiaUserAddress} --yes --chain-id testgaia --home=/opt --keyring-backend=test --gas auto --gas-adjustment 2 --output json`,
    );
    expect(res.exitCode).toBe(0);
    const out = JSON.parse(res.out);
    expect(out.code).toBe(0);
    expect(out.txhash).toHaveLength(64);
    let tx: IndexedTx | null = null;
    await waitFor(async () => {
      tx = await context.gaiaClient.getTx(out.txhash);
      return tx !== null;
    });
    expect(tx.height).toBeGreaterThan(0);
    expect(tx.code).toBe(0);
    const balances = await context.gaiaQueryClient.bank.allBalances(
      context.gaiaUserAddress,
    );
    expect(
      balances.find((a) => a.denom == `${context.validatorAddress}/1`),
    ).toEqual({
      denom: `${context.validatorAddress}/1`,
      amount: '600000',
    });
  });
  it('transfer tokenized share to neutron', async () => {
    const res = await context.park.executeInNetwork(
      'gaia',
      `gaiad tx ibc-transfer transfer transfer channel-0 ${context.neutronUserAddress} 600000${context.validatorAddress}/1 --from ${context.gaiaUserAddress}  --yes --chain-id testgaia --home=/opt --keyring-backend=test --gas auto --gas-adjustment 2 --output json`,
    );
    expect(res.exitCode).toBe(0);
    const out = JSON.parse(res.out);
    expect(out.code).toBe(0);
    expect(out.txhash).toHaveLength(64);
    let tx: IndexedTx | null = null;
    await waitFor(async () => {
      tx = await context.gaiaClient.getTx(out.txhash);
      return tx !== null;
    });
    expect(tx.height).toBeGreaterThan(0);
    expect(tx.code).toBe(0);
  });
  it('wait for neutron to receive tokenized share', async () => {
    const { neutronClient, neutronUserAddress } = context;
    let balances;
    await waitFor(async () => {
      balances =
        await neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
          neutronUserAddress,
        );
      return balances.data.balances.length > 3;
    });
    const shareOnNeutron = balances.data.balances.find(
      (b) => b.amount === '600000',
    );
    expect(shareOnNeutron).toBeDefined();
    expect(shareOnNeutron?.amount).toBe('600000');
    context.tokenizedDenomOnNeutron = shareOnNeutron?.denom;
  });
  it('bond tokenized share from unregistered validator', async () => {
    const { coreContractClient, neutronUserAddress } = context;
    const res = coreContractClient.bond(
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
    await expect(res).rejects.toThrowError(/Invalid denom/);
  });
  it('register validator', async () => {
    const { factoryContractClient, neutronUserAddress, validatorAddress } =
      context;
    const res = await factoryContractClient.proxy(neutronUserAddress, {
      validator_set: {
        update_validator: {
          validator: {
            valoper_address: validatorAddress,
            weight: 1,
          },
        },
      },
    });
    expect(res).toBeTruthy();
  });
  it('bond tokenized share from registered validator', async () => {
    const { coreContractClient, neutronUserAddress } = context;
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
  });

  it('unbond', async () => {
    const { coreContractClient, neutronUserAddress, ldDenom } = context;

    let res = await coreContractClient.unbond(
      neutronUserAddress,
      1.6,
      undefined,
      [
        {
          amount: Math.floor(300_000 / context.exchangeRate).toString(),
          denom: ldDenom,
        },
      ],
    );
    expect(res.transactionHash).toHaveLength(64);
    const amount = Math.floor(100_000 / context.exchangeRate).toString();
    res = await coreContractClient.unbond(neutronUserAddress, 1.6, undefined, [
      {
        amount,
        denom: ldDenom,
      },
    ]);
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
      total_amount: '400000',
      expected_amount: '400000',
      unbond_items: [
        {
          amount: '300000',
          expected_amount: '300000',
          sender: neutronUserAddress,
        },
        {
          amount: '100000',
          expected_amount: '100000',
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
    expect(vouchers.tokens.length).toBe(2);
    expect(vouchers.tokens[0]).toBe(`0_${neutronUserAddress}_1`);
    let tokenId = vouchers.tokens[0];
    let voucher = await withdrawalVoucherContractClient.queryNftInfo({
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
          {
            display_type: null,
            trait_type: 'expected_amount',
            value: '300000',
          },
          {
            display_type: null,
            trait_type: 'exchange_rate',
            value: '1',
          },
        ],
        batch_id: '0',
        description: 'Withdrawal voucher',
        expected_amount: '300000',
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
        amount: '100000',
        attributes: [
          {
            display_type: null,
            trait_type: 'unbond_batch_id',
            value: '0',
          },
          {
            display_type: null,
            trait_type: 'received_amount',
            value: '100000',
          },
          {
            display_type: null,
            trait_type: 'expected_amount',
            value: '100000',
          },
          {
            display_type: null,
            trait_type: 'exchange_rate',
            value: '1',
          },
        ],
        batch_id: '0',
        description: 'Withdrawal voucher',
        expected_amount: '100000',
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
        msg: Buffer.from(
          JSON.stringify({
            withdraw: {},
          }),
        ).toString('base64'),
      }),
    ).rejects.toThrowError(/is not unbonded yet/);
  });

  it('update batch status', async () => {
    const { coreContractClient, neutronUserAddress } = context;
    await coreContractClient.fakeProcessBatch(neutronUserAddress, {
      batch_id: '0',
      unbonded_amount: '200000',
    });
  });

  it('validate unbonding batch', async () => {
    const { coreContractClient, neutronUserAddress } = context;
    const batch = await coreContractClient.queryUnbondBatch({
      batch_id: '0',
    });
    expect(batch).toBeTruthy();
    expect(batch).toEqual<UnbondBatch>({
      slashing_effect: '0.5',
      status: 'unbonded',
      created: expect.any(Number),
      expected_release: 0,
      total_amount: '400000',
      expected_amount: '400000',
      unbond_items: [
        {
          amount: '300000',
          expected_amount: '300000',
          sender: neutronUserAddress,
        },
        {
          amount: '100000',
          expected_amount: '100000',
          sender: neutronUserAddress,
        },
      ],
      unbonded_amount: '200000',
      withdrawed_amount: null,
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
      [{ amount: '200000', denom: neutronIBCDenom }],
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
      msg: Buffer.from(
        JSON.stringify({
          withdraw: {},
        }),
      ).toString('base64'),
    });
    expect(res.transactionHash).toHaveLength(64);
    const balance = await neutronClient.CosmosBankV1Beta1.query.queryBalance(
      neutronUserAddress,
      { denom: neutronIBCDenom },
    );
    expect(parseInt(balance.data.balance.amount) - balanceBefore).toBe(150000);
  });
  it('withdraw to custom receiver', async () => {
    const {
      withdrawalVoucherContractClient: voucherContractClient,
      neutronUserAddress,
      neutronSecondUserAddress,
      neutronClient,
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
    const balance = await neutronClient.CosmosBankV1Beta1.query.queryBalance(
      neutronSecondUserAddress,
      { denom: neutronIBCDenom },
    );
    expect(parseInt(balance.data.balance.amount)).toBe(50000);
  });
});
