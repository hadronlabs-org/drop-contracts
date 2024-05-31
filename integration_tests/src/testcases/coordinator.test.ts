import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import {
  DropCore,
  DropFactory,
  DropPump,
  DropPuppeteer,
  DropStaker,
} from 'drop-ts-client';
import {
  QueryClient,
  StakingExtension,
  BankExtension,
  setupStakingExtension,
  setupBankExtension,
  SigningStargateClient,
} from '@cosmjs/stargate';
import { join } from 'path';
import { Tendermint34Client } from '@cosmjs/tendermint-rpc';
import {
  SigningCosmWasmClient,
  instantiate2Address,
} from '@cosmjs/cosmwasm-stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { AccountData, DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { awaitBlocks, generateWallets, setupPark } from '../testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { waitFor } from '../helpers/waitFor';
import { ContractSalt } from '../helpers/salt';
import { sha256, stringToPath } from '@cosmjs/crypto';
import { MsgTransfer } from 'cosmjs-types/ibc/applications/transfer/v1/tx';

const DropPumpClass = DropPump.Client;
const DropFactoryClass = DropFactory.Client;
const DropPuppeteerClass = DropPuppeteer.Client;
const DropCoreClass = DropCore.Client;
const DropStakerClass = DropStaker.Client;

describe('Coordinator', () => {
  let pumpContractBinary: Uint8Array;
  let factoryContractBinary: Uint8Array;

  const context: {
    park?: Cosmopark;
    pumpContractAddress?: string;
    factoryContractAddress?: string;
    withdrawalManagerContractAddress?: string;
    wallet?: DirectSecp256k1HdWallet;
    gaiaWallet?: DirectSecp256k1HdWallet;
    pumpContractClient?: InstanceType<typeof DropPumpClass>;
    factoryContractClient?: InstanceType<typeof DropFactoryClass>;
    stakerContractClient?: InstanceType<typeof DropStakerClass>;
    puppeteerContractClient?: InstanceType<typeof DropPuppeteerClass>;
    coreContractClient?: InstanceType<typeof DropCoreClass>;
    account?: AccountData;
    pumpIcaAddress?: string;
    puppeteerIcaAddress?: string;
    stakerIcaAddress?: string;
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
      staker?: number;
      puppeteer?: number;
      validatorsSet?: number;
      distribution?: number;
      rewardsManager?: number;
      pump?: number;
      factory?: number;
    };
    coreCoreId?: number;
    tokenCodeId?: number;
    exchangeRate?: number;
    tokenContractAddress?: string;
    neutronIBCDenom?: string;
    ldDenom?: string;
  } = { codeIds: {} };

  beforeAll(async (t) => {
    const wallets = await generateWallets();
    context.wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      wallets.demowallet1,
      {
        prefix: 'neutron',
      },
    );
    context.account = (await context.wallet.getAccounts())[0];

    pumpContractBinary = fs.readFileSync(
      join(__dirname, '../../../artifacts/drop_pump.wasm'),
    );
    const pumpChecksum = sha256(pumpContractBinary);
    context.pumpContractAddress = instantiate2Address(
      pumpChecksum,
      context.account.address,
      new Uint8Array([ContractSalt]),
      'neutron',
    );

    factoryContractBinary = fs.readFileSync(
      join(__dirname, '../../../artifacts/drop_factory.wasm'),
    );
    const factoryCodeChecksum = sha256(factoryContractBinary);
    context.factoryContractAddress = instantiate2Address(
      factoryCodeChecksum,
      context.account.address,
      new Uint8Array([ContractSalt]),
      'neutron',
    );

    context.park = await setupPark(
      t,
      ['neutron', 'gaia'],
      {},
      {
        hermes: true,
        coordinator: {
          environment: {
            FACTORY_CONTRACT_ADDRESS: context.factoryContractAddress,
            COORDINATOR_CHECKS_PERIOD: '10',
            PUMP_CONTRACT_ADDRESS: context.pumpContractAddress,
            PUMP_MIN_BALANCE: '10',
          },
        },
      },
      wallets,
    );

    context.gaiaWallet = await DirectSecp256k1HdWallet.fromMnemonic(
      context.park.config.wallets.demowallet1.mnemonic,
      {
        prefix: 'cosmos',
      },
    );
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

  afterAll(async () => {
    // await context.park.stop();
  });

  it('upload contracts', async () => {
    const { client, account } = context;
    {
      const res = await client.upload(account.address, pumpContractBinary, 1.5);
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.pump = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        factoryContractBinary,
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.factory = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(join(__dirname, '../../../artifacts/drop_core.wasm')),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.core = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(join(__dirname, '../../../artifacts/drop_token.wasm')),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.token = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_withdrawal_voucher.wasm'),
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
          join(__dirname, '../../../artifacts/drop_withdrawal_manager.wasm'),
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
          join(__dirname, '../../../artifacts/drop_strategy.wasm'),
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
          join(__dirname, '../../../artifacts/drop_distribution.wasm'),
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
          join(__dirname, '../../../artifacts/drop_validators_set.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.validatorsSet = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(join(__dirname, '../../../artifacts/drop_staker.wasm')),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.staker = res.codeId;
    }
    {
      const res = await client.upload(
        account.address,
        fs.readFileSync(
          join(__dirname, '../../../artifacts/drop_puppeteer.wasm'),
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
          join(__dirname, '../../../artifacts/drop_rewards_manager.wasm'),
        ),
        1.5,
      );
      expect(res.codeId).toBeGreaterThan(0);
      context.codeIds.rewardsManager = res.codeId;
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

  it('instantiate', async () => {
    const { client, account } = context;
    const instantiateRes = await DropFactory.Client.instantiate2(
      client,
      account.address,
      context.codeIds.factory,
      ContractSalt,
      {
        sdk_version: process.env.SDK_VERSION || '0.46.0',
        code_ids: {
          core_code_id: context.codeIds.core,
          token_code_id: context.codeIds.token,
          withdrawal_voucher_code_id: context.codeIds.withdrawalVoucher,
          withdrawal_manager_code_id: context.codeIds.withdrawalManager,
          strategy_code_id: context.codeIds.strategy,
          staker_code_id: context.codeIds.staker,
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
          idle_min_interval: 5,
          puppeteer_timeout: 60,
          unbond_batch_switch_time: 60,
          unbonding_safe_period: 10,
          unbonding_period: 360,
          lsm_redeem_threshold: 2,
          lsm_min_bond_amount: '1000',
          lsm_redeem_max_interval: 60_000,
          bond_limit: '100000',
          min_stake_amount: '2',
          icq_update_delay: 5,
        },
        staker_params: {
          min_stake_amount: '10000',
          min_ibc_transfer: '10000',
        },
      },
      'drop-staking-factory',
      'auto',
      [],
    );

    expect(instantiateRes.contractAddress).toEqual(
      context.factoryContractAddress,
    );

    context.factoryContractClient = new DropFactoryClass(
      client,
      context.factoryContractAddress,
    );

    const res = await context.factoryContractClient.queryState();
    expect(res).toBeTruthy();

    context.puppeteerContractClient = new DropPuppeteerClass(
      context.client,
      res.puppeteer_contract,
    );

    context.coreContractClient = new DropCoreClass(
      context.client,
      res.core_contract,
    );

    context.stakerContractClient = new DropStaker.Client(
      context.client,
      res.staker_contract,
    );

    context.withdrawalManagerContractAddress = res.withdrawal_manager_contract;
  });

  it('instantiate pump contract', async () => {
    const {
      client,
      account,
      neutronUserAddress,
      withdrawalManagerContractAddress,
    } = context;
    {
      const instantiateRes = await DropPump.Client.instantiate2(
        client,
        account.address,
        context.codeIds.pump,
        ContractSalt,
        {
          connection_id: 'connection-0',
          dest_address: withdrawalManagerContractAddress,
          dest_channel: 'channel-0',
          dest_port: 'transfer',
          local_denom: 'untrn',
          refundee: neutronUserAddress,
          timeout: {
            local: 60,
            remote: 60,
          },
          owner: account.address,
        },
        'drop-staking-pump',
        1.5,
        [],
      );
      expect(instantiateRes.contractAddress).toEqual(
        context.pumpContractAddress,
      );

      context.pumpContractClient = new DropPumpClass(
        client,
        context.pumpContractAddress,
      );
    }
  });

  it('register staker ICA', async () => {
    const { stakerContractClient, neutronUserAddress } = context;
    const res = await stakerContractClient.registerICA(
      neutronUserAddress,
      1.5,
      undefined,
      [{ amount: '1000000', denom: 'untrn' }],
    );
    expect(res.transactionHash).toHaveLength(64);
    let ica = '';
    await waitFor(async () => {
      const res = await stakerContractClient.queryIca();
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
    context.stakerIcaAddress = ica;
  });

  it('register pump ICA', async () => {
    const { pumpContractClient: contractClient, neutronUserAddress } = context;
    const res = await contractClient.registerICA(
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
    expect(res).toBeTruthy();
    expect(res.transactionHash).toHaveLength(64);
    let ica = '';
    await waitFor(async () => {
      const res = await contractClient.queryIca();
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
    context.pumpIcaAddress = ica;

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
    context.puppeteerIcaAddress = ica;
  });

  it('set puppeteer ICA to the staker', async () => {
    const res = await context.factoryContractClient.adminExecute(
      context.neutronUserAddress,
      {
        msgs: [
          {
            wasm: {
              execute: {
                contract_addr: context.stakerContractClient.contractAddress,
                msg: Buffer.from(
                  JSON.stringify({
                    update_config: {
                      new_config: {
                        puppeteer_ica: context.puppeteerIcaAddress,
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
      undefined,
      [],
    );
    expect(res.transactionHash).toHaveLength(64);
  });

  it('grant staker to delegate funds from puppeteer ICA', async () => {
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
                    grant_delegate: {
                      grantee: context.stakerIcaAddress,
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

  it('verify grant', async () => {
    const res = await context.park.executeInNetwork(
      'gaia',
      `${context.park.config.networks['gaia'].binary} query authz grants-by-grantee ${context.stakerIcaAddress} --output json`,
    );
    const out = JSON.parse(res.out);
    expect(out.grants).toHaveLength(1);
    const grant = out.grants[0];
    expect(grant.granter).toEqual(context.puppeteerIcaAddress);
    expect(grant.grantee).toEqual(context.stakerIcaAddress);
  });

  it('send some funds to ICA', async () => {
    const { gaiaClient, gaiaUserAddress, pumpIcaAddress } = context;
    const res = await gaiaClient.sendTokens(
      gaiaUserAddress,
      pumpIcaAddress,
      [
        {
          amount: '1000000',
          denom: 'stake',
        },
      ],
      1.5,
    );
    expect(res.transactionHash).toHaveLength(64);
  });
  it('verify funds are received', async () => {
    const { neutronClient, withdrawalManagerContractAddress } = context;
    let ibcBalance = 0;
    await waitFor(async () => {
      const res = await neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
        withdrawalManagerContractAddress,
      );

      ibcBalance = parseInt(
        res.data.balances.find((b) => b.denom.startsWith('ibc/'))?.amount ||
          '0',
      );
      return res.data.balances.length >= 1;
    }, 100000);
    expect(ibcBalance).toEqual(1000000);
  });
  it('check balance on pump', async () => {
    const { neutronClient, pumpContractAddress: contractAddress } = context;
    await waitFor(async () => {
      const res =
        await neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
          contractAddress,
        );
      return res.data.balances.length > 0;
    });
    const res =
      await neutronClient.CosmosBankV1Beta1.query.queryAllBalances(
        contractAddress,
      );
    expect(res.data.balances).toEqual([
      {
        amount: '19000',
        denom: 'untrn',
      },
    ]);
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

  describe('Core state machine', () => {
    it('get machine state', async () => {
      const state = await context.coreContractClient.queryContractState();
      expect(state).toEqual('idle');
    });

    it('bond coins', async () => {
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
            amount: '20000',
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

    describe('state machine cycle', () => {
      const ica: { balance?: number } = {};

      it('get ICA balance', async () => {
        const { gaiaClient } = context;
        const res = await gaiaClient.getBalance(
          context.puppeteerIcaAddress,
          'stake',
        );
        ica.balance = parseInt(res.amount);
        expect(ica.balance).toEqual(0);
      });

      it.skip('tick', async () => {
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
      it('get ICA increased balance', async () => {
        const { gaiaClient } = context;
        let res;
        await waitFor(async () => {
          try {
            res = await gaiaClient.getBalanceStaked(
              context.puppeteerIcaAddress,
            );
          } catch (e) {
            //
          }
          return res && parseInt(res.amount) !== 0;
        }, 500_000);

        expect(res.amount).toEqual('20000');
      });
      it.skip('wait for balances to come', async () => {
        let res;
        await waitFor(async () => {
          try {
            res = await context.puppeteerContractClient.queryExtension({
              msg: {
                balances: {},
              },
            });
          } catch (e) {
            //
          }
          return res && res[0].coins.length !== 0;
        }, 500_000);
      });

      it.skip('state should be staking', async () => {
        await waitFor(async () => {
          let state;
          try {
            state = await context.coreContractClient.queryContractState();
          } catch (e) {
            //
          }
          return state === 'staking';
        }, 500_000);

        const state = await context.coreContractClient.queryContractState();
        expect(state).toEqual('staking');
      });
    });
  });
});
