import cosmopark, { CosmoparkConfig } from '@neutron-org/cosmopark';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { StargateClient } from '@cosmjs/stargate';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { waitFor } from './helpers/waitFor';
import { sleep } from './helpers/sleep';

const keys = [
  'master',
  'hermes',
  'ibcrelayer',
  'demowallet1',
  'demo1',
  'demo2',
  'demo3',
] as const;

const TIMEOUT = 10_000;

const networkConfigs = {
  lsm: {
    binary: 'liquidstakingd',
    chain_id: 'testlsm',
    denom: 'stake',
    image: 'lsm',
    prefix: 'cosmos',
    trace: true,
    validators: 2,
    validators_balance: '1000000000',
    genesis_opts: {
      'app_state.slashing.params.downtime_jail_duration': '10s',
      'app_state.slashing.params.signed_blocks_window': '10',
      'app_state.staking.params.validator_bond_factor': '10',
      'app_state.interchainaccounts.host_genesis_state.params.allow_messages': [
        '*',
      ],
    },
    config_opts: {
      'rpc.laddr': 'tcp://0.0.0.0:26657',
    },
    app_opts: {
      'api.enable': true,
      'api.address': 'tcp://0.0.0.0:1317',
      'api.swagger': true,
      'grpc.enable': true,
      'grpc.address': '0.0.0.0:9090',
      'minimum-gas-prices': '0stake',
      'rosetta.enable': true,
    },
    upload: ['./artifacts/scripts/init-lsm.sh'],
    post_start: [`/opt/init-lsm.sh > /opt/init-lsm.log 2>&1`],
  },
  gaia: {
    binary: 'gaiad',
    chain_id: 'testgaia',
    denom: 'stake',
    image: 'gaia',
    prefix: 'cosmos',
    trace: true,
    validators: 2,
    validators_balance: '1000000000',
    genesis_opts: {
      'app_state.slashing.params.downtime_jail_duration': '10s',
      'app_state.slashing.params.signed_blocks_window': '10',
      'app_state.staking.params.validator_bond_factor': '10',
      'app_state.interchainaccounts.host_genesis_state.params.allow_messages': [
        '*',
      ],
    },
    config_opts: {
      'rpc.laddr': 'tcp://0.0.0.0:26657',
    },
    app_opts: {
      'api.enable': true,
      'api.address': 'tcp://0.0.0.0:1317',
      'api.swagger': true,
      'grpc.enable': true,
      'grpc.address': '0.0.0.0:9090',
      'minimum-gas-prices': '0stake',
      'rosetta.enable': true,
    },
    upload: ['./artifacts/scripts/init-gaia.sh'],
    post_start: [`/opt/init-gaia.sh > /opt/init-gaia.log 2>&1`],
  },
  neutron: {
    binary: 'neutrond',
    chain_id: 'ntrntest',
    denom: 'untrn',
    image: 'neutron-node',
    prefix: 'neutron',
    loglevel: 'debug',
    trace: true,
    type: 'ics',
    upload: [
      './artifacts/contracts',
      './artifacts/contracts_thirdparty',
      './artifacts/scripts/init-neutrond.sh',
    ],
    post_init: ['CHAINID=ntrntest CHAIN_DIR=/opt /opt/init-neutrond.sh'],
    genesis_opts: {
      'app_state.crisis.constant_fee.denom': 'untrn',
    },
    config_opts: {
      'consensus.timeout_commit': '1s',
      'consensus.timeout_propose': '1s',
    },
    app_opts: {
      'api.enable': 'true',
      'api.address': 'tcp://0.0.0.0:1317',
      'api.swagger': 'true',
      'grpc.enable': 'true',
      'grpc.address': '0.0.0.0:9090',
      'minimum-gas-prices': '0.0025untrn',
      'rosetta.enable': 'true',
      'telemetry.prometheus-retention-time': 1000,
    },
  },
};

const relayersConfig = {
  hermes: {
    balance: '1000000000',
    binary: 'hermes',
    config: {
      'chains.0.trusting_period': '14days',
      'chains.0.unbonding_period': '480h0m0s',
      'chains.1.gas_multiplier': 1.2,
      'chains.0.gas_multiplier': 1.2,
    },
    image: 'hermes',
    log_level: 'trace',
    type: 'hermes',
  },
  neutron: {
    balance: '1000000000',
    binary: 'neutron-query-relayer',
    image: 'neutron-org/neutron-query-relayer',
    log_level: 'debug',
    type: 'neutron',
  },
};

type Keys = (typeof keys)[number];

const awaitFirstBlock = (rpc: string): Promise<void> =>
  waitFor(async () => {
    try {
      const client = await StargateClient.connect(rpc);
      const block = await client.getBlock();
      if (block.header.height > 1) {
        return true;
      }
    } catch (e) {
      return false;
    }
  }, 20_000);

export const awaitBlocks = async (
  rpc: string,
  blocks: number,
): Promise<void> => {
  const start = Date.now();
  const client = await StargateClient.connect(rpc);
  const initBlock = await client.getBlock();
  // eslint-disable-next-line no-constant-condition
  while (true) {
    try {
      const block = await client.getBlock();
      if (block.header.height - initBlock.header.height >= blocks) {
        break;
      }
      if (Date.now() - start > TIMEOUT) {
        throw new Error('Timeout waiting for the specific block');
      }
    } catch (e) {
      //noop
    }
    await sleep(1000);
  }
};

const awaitNeutronChannels = (rest: string, rpc: string): Promise<void> =>
  waitFor(async () => {
    try {
      const client = new NeutronClient({
        apiURL: `http://${rest}`,
        rpcURL: `http://${rpc}`,
        prefix: 'neutron',
      });
      const res = await client.IbcCoreChannelV1.query.queryChannels();
      if (
        res.data.channels.length > 0 &&
        res.data.channels[0].counterparty.channel_id !== ''
      ) {
        return true;
      }
    } catch (e) {
      console.log('Failed to await neutron channels', e);
      return false;
    }
  }, 100_000);

export const generateWallets = (): Promise<Record<Keys, string>> =>
  keys.reduce(
    async (acc, key) => {
      const accObj = await acc;
      const wallet = await DirectSecp256k1HdWallet.generate(12, {
        prefix: 'neutron',
      });
      accObj[key] = wallet.mnemonic;
      return accObj;
    },
    Promise.resolve({} as Record<Keys, string>),
  );

export const setupPark = async (
  context = 'lido',
  networks: string[] = [],
  needRelayers = false,
): Promise<cosmopark> => {
  const wallets = await generateWallets();
  const config: CosmoparkConfig = {
    context,
    networks: {},
    master_mnemonic: wallets.master,
    loglevel: 'info',
    wallets: {
      demowallet1: {
        mnemonic: wallets.demowallet1,
        balance: '1000000000',
      },
      demo1: { mnemonic: wallets.demo1, balance: '1000000000' },
      demo2: { mnemonic: wallets.demo2, balance: '1000000000' },
      demo3: { mnemonic: wallets.demo3, balance: '1000000000' },
    },
  };
  for (const network of networks) {
    config.networks[network] = networkConfigs[network];
  }
  if (needRelayers) {
    config.relayers = [
      {
        ...relayersConfig.hermes,
        networks,
        connections: [networks],
        mnemonic: wallets.hermes,
      } as any,
      {
        ...relayersConfig.neutron,
        networks,
        mnemonic: wallets.ibcrelayer,
      },
    ];
  }
  const instance = await cosmopark.create(config);
  await Promise.all(
    Object.entries(instance.ports).map(([network, ports]) =>
      awaitFirstBlock(`127.0.0.1:${ports.rpc}`).catch((e) => {
        console.log(`Failed to await first block for ${network}: ${e}`);
        throw e;
      }),
    ),
  );
  console.log('Awaited first blocks');
  if (needRelayers) {
    await awaitNeutronChannels(
      `127.0.0.1:${instance.ports['neutron'].rest}`,
      `127.0.0.1:${instance.ports['neutron'].rpc}`,
    ).catch((e) => {
      console.log(`Failed to await neutron channels: ${e}`);
      throw e;
    });
  }
  return instance;
};

export const setupSingle = async (
  context = 'lido',
  network: string = 'neutron',
): Promise<cosmopark> => {
  const wallets = await generateWallets();
  const config: CosmoparkConfig = {
    context,
    networks: {},
    master_mnemonic: wallets.master,
    loglevel: 'info',
    wallets: {
      demowallet1: {
        mnemonic: wallets.demowallet1,
        balance: '1000000000',
      },
      demo1: { mnemonic: wallets.demo1, balance: '1000000000' },
      demo2: { mnemonic: wallets.demo2, balance: '1000000000' },
      demo3: { mnemonic: wallets.demo3, balance: '1000000000' },
    },
  };
  config.networks[network] = networkConfigs[network];
  const instance = await cosmopark.create(config);
  await Promise.all(
    Object.entries(instance.ports).map(([network, ports]) =>
      awaitFirstBlock(`127.0.0.1:${ports.rpc}`).catch((e) => {
        console.log(`Failed to await first block for ${network}: ${e}`);
        throw e;
      }),
    ),
  );
  console.log('Awaited first blocks');
  return instance;
};
