import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { SigningStargateClient } from "@cosmjs/stargate";
import {
  AccountData,
  DirectSecp256k1HdWallet,
  GeneratedType,
  Registry,
} from "@cosmjs/proto-signing";
import { GasPrice, Coin } from "@cosmjs/stargate";
import { Client as DropCoreClient } from "../../../integration_tests/src/generated/contractLib/dropCore";
import { Client as DropWithdrawalManager } from "../../../integration_tests/src/generated/contractLib/dropWithdrawalManager";
import { Client as DropWithdrawalVoucher } from "../../../integration_tests/src/generated/contractLib/dropWithdrawalVoucher";

import { MsgTokenizeShares } from "@messages";
import { MsgSend } from "cosmjs-types/cosmos/bank/v1beta1/tx";
import { MsgDelegate } from "cosmjs-types/cosmos/staking/v1beta1/tx";
import { MsgTransfer } from "cosmjs-types/ibc/applications/transfer/v1/tx";

import { sleep } from "../../../integration_tests/src/helpers/sleep";

const CORE_CONTRACT: string = process.env.CORE_CONTRACT;

const MNEMONIC: string = process.env.MNEMONIC;
const BASE_DENOM: string = process.env.BASE_DENOM;
const FACTORY_DENOM: string = process.env.FACTORY_DENOM;
const NEUTRON_NODE_ADDRESS: string = process.env.NEUTRON_NODE_ADDRESS;
const TARGET_NODE_ADDRESS: string = process.env.TARGET_NODE_ADDRESS;

const TARGET_CHAIN_PREFIX: string = process.env.TARGET_CHAIN_PREFIX;
const TARGET_DENOM: string = process.env.TARGET_DENOM;

const MAX_BOND: number = Number(process.env.MAX_BOND);
const MAX_UNBOND: number = Number(process.env.MAX_UNBOND);
const MAX_LSM_PROCESS: number = Number(process.env.MAX_LSM_PROCESS);

const IBC_CHANNEL_TO: string = process.env.IBC_CHANNEL_TO;
const IBC_CHANNEL_FROM: string = process.env.IBC_CHANNEL_FROM;

const BOND_PROB: number = Number(process.env.BOND_PROB);
const UNBOND_PROB: number = Number(process.env.UNBOND_PROB);
const WITHDRAW_PROB: number = Number(process.env.WITHDRAW_PROB);
const PROCESS_LSM_PROB: number = Number(process.env.PROCESS_LSM_PROB);

/*
 * Each of given probabilities should be in interval [0, 1]
 * According to given probabilities relevant actions will be choosed with appropriate probability
 */
if ((0 <= BOND_PROB && BOND_PROB <= 1) === false) {
  console.error(`0 <= BOND_PROB(${BOND_PROB}) <= 1 != true`);
  process.exit(1);
}
if ((0 <= UNBOND_PROB && UNBOND_PROB <= 1) === false) {
  console.error(`0 <= UNBOND_PROB(${UNBOND_PROB}) <= 1 != true`);
  process.exit(1);
}
if ((0 <= WITHDRAW_PROB && WITHDRAW_PROB <= 1) === false) {
  console.error(`0 <= WITHDRAW_PROB(${WITHDRAW_PROB}) <= 1 != true`);
  process.exit(1);
}

function isInstance<T extends object>(value: string, type: T): type is T {
  return Object.values(type).includes(value);
}

enum NeutronAction {
  BOND = "BOND",
  UNBOND = "UNBOND",
  WITHDRAW = "SEND_NFT",
  PROCESS_LSM_SHARES_IBC_TO = "PROCESS_LSM_SHARES.IBC_TO",
  PROCESS_LSM_SHARES_BOND = "PROCESS_LSM_SHARES.BOND",
}

enum TargetAction {
  PROCESS_LSM_SHARES_IBC_FROM = "PROCESS_LSM_SHARES.IBC_FROM",
  PROCESS_LSM_SHARES_DELEGATE = "PROCESS_LSM_SHARES.DELEGATE",
  PROCESS_LSM_SHARES_TOKENIZE_SHARES = "PROCESS_LSM_SHARES.TOKENIZE_SHARES",
}

type MODE = NeutronAction | TargetAction;

type Wallet = {
  mainWallet?: DirectSecp256k1HdWallet;
  clientCW?: SigningCosmWasmClient;
  clientSG?: SigningStargateClient;
  mainAccounts?: readonly AccountData[];
};

type ActionLog = {
  mode: MODE;
  txHash: string;
};

type ErrorLog = {
  mode: MODE;
  txHash: string | null;
  reason: string;
};

type Action = ActionLog | ErrorLog;

async function bond(
  dropInstance: DropCoreClient,
  address: string,
  fund: Coin
): Promise<ActionLog> {
  const { transactionHash } = await dropInstance.bond(address, {}, "auto", "", [
    {
      amount: fund.amount,
      denom: fund.denom,
    },
  ]);
  return {
    mode: NeutronAction.BOND,
    txHash: transactionHash,
  };
}

/* Function returns Action if transaction succeeded and null if any error or problem occurred
 * We don't really need to know type of error here since it isn't crucial for us,
 * We'll just try another method in core
 */
async function bondRandomAmount(
  neutronWallet: Wallet,
  dropInstance: DropCoreClient
): Promise<Action> {
  const address: string = neutronWallet.mainAccounts[0].address;

  /* If here is nothing to bond on our balance, then just return null
   * Other random method will be tried to call then
   */
  const IBCDenomBalance: Coin = await neutronWallet.clientCW.getBalance(
    address,
    BASE_DENOM
  );
  if (Number(IBCDenomBalance.amount) === 0) {
    return {
      mode: NeutronAction.BOND,
      txHash: null,
      reason: `Nothing to bond, ${BASE_DENOM} balance is 0`,
    };
  }

  const config = await dropInstance.queryConfig();
  const exchangeRate: number = Math.floor(
    Number(await dropInstance.queryExchangeRate())
  );
  const minExchangeRate: number = exchangeRate + 1;

  /* Minimum amount of funds that we can send to core contract while bonding
   * Is either lsm_min_bond_amount (which is typically set in 1) or current exchange_rate parameter
   * Here we're choosing the biggest value of these two to avoid further errors
   */
  const min: number =
    Number(config.lsm_min_bond_amount) < minExchangeRate
      ? minExchangeRate
      : Number(config.lsm_min_bond_amount);
  if (min > Number(IBCDenomBalance.amount)) {
    return {
      mode: NeutronAction.BOND,
      txHash: null,
      reason: `Nothing to bond, ${BASE_DENOM} balance is lower then min(${min}) (this value either exchange rate or config.lsm_min_bond_amount)`,
    };
  }
  if (min > MAX_BOND) {
    return {
      mode: NeutronAction.BOND,
      txHash: null,
      reason: `MAX_BOND lower then min(${min}) (this value either exchange rate or config.lsm_min_bond_amount)`,
    };
  }

  /* Maximum amount of funds that we can send to core contract while bonding
   * Is either our current balance in case if it's lower then MAX_BOND parameter
   * or MAX_BOND otherwise.
   */
  const max: number =
    Number(IBCDenomBalance.amount) < MAX_BOND
      ? Number(IBCDenomBalance.amount)
      : MAX_BOND;

  /* If any error occured when executing method then just ignore
   * It's content and return null, script will try to call another method
   */
  const randomAmount: number = Math.floor(Math.random() * (max - min) + min);
  try {
    const res = await bond(dropInstance, address, {
      amount: String(randomAmount),
      denom: BASE_DENOM,
    });
    const { code, hash } = await neutronWallet.clientCW.getTx(res.txHash);
    if (code !== 0) {
      return {
        mode: NeutronAction.BOND,
        txHash: hash,
        reason: "Check up given hash",
      };
    }
    return res;
  } catch (e) {
    return {
      mode: NeutronAction.BOND,
      txHash: null,
      reason: e.message,
    };
  }
}

async function unbond(
  dropInstance: DropCoreClient,
  address: string,
  fund: Coin
): Promise<ActionLog> {
  const { transactionHash } = await dropInstance.unbond(address, "auto", "", [
    {
      amount: fund.amount,
      denom: fund.denom,
    },
  ]);
  return {
    mode: NeutronAction.UNBOND,
    txHash: transactionHash,
  };
}

/* Function returns Action if transaction succeeded and null if any error or problem occurred
 * We don't really need to know type of error here since it isn't crucial for us,
 * We'll just try another method in core
 */
async function unbondRandomAmount(
  neutronWallet: Wallet,
  dropInstance: DropCoreClient
): Promise<Action> {
  const address: string = neutronWallet.mainAccounts[0].address;
  /* If here is nothing to bond on our balance, then just return null
   * Other random method will be tried to call then
   */
  const factoryBalance: Coin = await neutronWallet.clientCW.getBalance(
    address,
    FACTORY_DENOM
  );
  if (Number(factoryBalance.amount) === 0) {
    return {
      mode: NeutronAction.UNBOND,
      txHash: null,
      reason: `Nothing to unbond, ${FACTORY_DENOM} balance is 0`,
    };
  }

  const max: number =
    Number(factoryBalance.amount) < MAX_UNBOND
      ? Number(factoryBalance.amount)
      : MAX_UNBOND;

  /* If any error occured when executing method then just ignore
   * It's content and return null, script will try to call another method
   */
  const randomAmount: number = Math.floor(Math.random() * Number(max) + 1);
  try {
    const res = await unbond(dropInstance, address, {
      amount: String(randomAmount),
      denom: FACTORY_DENOM,
    });
    const { code, hash } = await neutronWallet.clientCW.getTx(res.txHash);
    if (code !== 0) {
      return {
        mode: NeutronAction.UNBOND,
        txHash: hash,
        reason: "Check up given hash",
      };
    }
    return res;
  } catch (e) {
    return {
      mode: NeutronAction.UNBOND,
      txHash: null,
      reason: e.message,
    };
  }
}

async function sendNFT(
  withdrawalVoucher: DropWithdrawalVoucher,
  withdrawalManager: DropWithdrawalManager,
  address: string,
  NFTID: string
): Promise<ActionLog> {
  const { transactionHash } = await withdrawalVoucher.sendNft(
    address,
    {
      contract: withdrawalManager.contractAddress,
      token_id: NFTID,
      msg: "eyJ3aXRoZHJhdyI6e319",
    },
    "auto",
    "",
    []
  );
  /* To withdraw unbonded amount we need to send nft to the withdrawal_manager contract
   * To do that we need to call send method on withdrawal_voucher contract (which is NFT contract underhood)
   * Field msg here is encoded base64 json object { "withdraw": {} }
   */
  return {
    mode: NeutronAction.WITHDRAW,
    txHash: transactionHash,
  };
}

/* Function returns Action if transaction succeeded and null if any error or problem occurred
 * We don't really need to know type of error here since it isn't crucial for us,
 * We'll just try another method in core
 */
async function withdrawRandomNFT(
  neutronWallet: Wallet,
  dropInstance: DropCoreClient
): Promise<Action> {
  const address: string = neutronWallet.mainAccounts[0].address;
  /* Get both withdrawal_manager and withdrawal_voucher wrappers based on querying config method
   * We need them to execute send_nft method on withdrawal_voucher with withdrawal_manager as the recepient
   * To withdraw our unbonded tokens we need to work with withdrawal_voucher' send_nft method
   */
  const config = await dropInstance.queryConfig();
  const withdrawalManager: DropWithdrawalManager = new DropWithdrawalManager(
    neutronWallet.clientCW,
    config.withdrawal_manager_contract
  );
  const withdrawalVoucher: DropWithdrawalVoucher = new DropWithdrawalVoucher(
    neutronWallet.clientCW,
    config.withdrawal_voucher_contract
  );

  /* Get all NFT tokens in 1-st loop possessed by given address
   * Get details for each gotten NFT in 2-nd loop
   * Check batch status for each batch_id, filter who hasn't withdrawed yet in 3-rd loop
   */
  let ownedNFTs = [];
  while (true) {
    const startAfter =
      ownedNFTs.length > 0 ? ownedNFTs[ownedNFTs.length - 1] : undefined;
    const { tokens } = await withdrawalVoucher.queryTokens({
      owner: address,
      ...(startAfter && { start_after: startAfter }),
    });
    if (tokens.length === 0) {
      break;
    }
    ownedNFTs = ownedNFTs.concat(tokens);
  }
  const ownedNFTsDetailed = [];
  for (const NFT of ownedNFTs) {
    ownedNFTsDetailed.push({
      nft_id: NFT,
      details: await withdrawalVoucher.queryNftInfo({
        token_id: NFT,
      }),
    });
  }
  const withdrawnNFTs = [];
  for (const NFTDetailed of ownedNFTsDetailed) {
    const { status } = await dropInstance.queryUnbondBatch({
      batch_id: NFTDetailed.details.extension.batch_id,
    });
    if (status === "withdrawn") {
      withdrawnNFTs.push(NFTDetailed.nft_id);
    }
  }

  /* Nothing to withdraw
   * Return null and try another method in contract
   */
  if (withdrawnNFTs.length === 0) {
    return {
      mode: NeutronAction.WITHDRAW,
      txHash: null,
      reason: "Nothing to withdraw",
    };
  }

  /* Pick up random NFT from given NFT list and try to withdraw it
   * If any error occured when executing method then just ignore
   * It's content and return null, script will try to call another method
   */
  const NFTID: string =
    withdrawnNFTs[Math.floor(Math.random() * withdrawnNFTs.length)];
  try {
    const res = await sendNFT(
      withdrawalVoucher,
      withdrawalManager,
      address,
      NFTID
    );
    const { code, hash } = await neutronWallet.clientCW.getTx(res.txHash);
    if (code !== 0) {
      return {
        mode: NeutronAction.WITHDRAW,
        txHash: hash,
        reason: "Check up given hash",
      };
    }
    return res;
  } catch (e) {
    return {
      mode: NeutronAction.WITHDRAW,
      txHash: null,
      reason: e.message,
    };
  }
}

async function IBCToTransfer(
  clientSG: SigningStargateClient,
  address_from: string,
  address_to: string,
  channel: string,
  port: string,
  amount: Coin
): Promise<Action> {
  const transactionHash = await clientSG.signAndBroadcastSync(
    address_from,
    [
      {
        typeUrl: "/ibc.applications.transfer.v1.MsgTransfer",
        value: {
          sourcePort: port,
          sourceChannel: channel,
          token: {
            denom: amount.denom,
            amount: amount.amount,
          },
          sender: address_from,
          receiver: address_to,
          timeoutHeight: "0",
          timeoutTimestamp: String(Date.now() + 60 * 10e3),
        },
      },
    ],
    {
      gas: "400000",
      amount: [
        {
          denom: "untrn",
          amount: "4000",
        },
      ],
    },
    ""
  );
  return {
    txHash: transactionHash,
    mode: NeutronAction.PROCESS_LSM_SHARES_IBC_TO,
  };
}

async function randomIBCToTransfer(
  dropCore: DropCoreClient,
  neutronWallet: Wallet,
  address_from: string,
  address_to: string
): Promise<Action> {
  const baseDenomBalance: Coin = await neutronWallet.clientCW.getBalance(
    address_from,
    BASE_DENOM
  );
  if (Number(baseDenomBalance.amount) === 0) {
    return {
      mode: NeutronAction.PROCESS_LSM_SHARES_IBC_TO,
      txHash: null,
      reason: `Nothing to transfer via IBC, ${BASE_DENOM} balance is 0`,
    };
  }

  const exchangeRate: number = Math.floor(
    Number(await dropCore.queryExchangeRate())
  );
  const minExchangeRate: number = exchangeRate + 1;
  const config = await dropCore.queryConfig();

  const min: number =
    Number(config.lsm_min_bond_amount) < minExchangeRate
      ? minExchangeRate
      : Number(config.lsm_min_bond_amount);
  if (min > Number(baseDenomBalance.amount)) {
    return {
      mode: NeutronAction.PROCESS_LSM_SHARES_IBC_TO,
      txHash: null,
      reason: `Nothing to send via IBC, ${BASE_DENOM} balance is lower then min(${min}) (this value either exchange rate or config.lsm_min_bond_amount)`,
    };
  }
  if (min > MAX_LSM_PROCESS) {
    return {
      mode: NeutronAction.PROCESS_LSM_SHARES_IBC_TO,
      txHash: null,
      reason: `MAX_LSM_PROCESS lower then min(${min}) (this value either exchange rate or config.lsm_min_bond_amount)`,
    };
  }

  const max: number =
    Number(baseDenomBalance.amount) < MAX_LSM_PROCESS
      ? Number(baseDenomBalance.amount)
      : MAX_LSM_PROCESS;

  const randomAmount: number = Math.floor(Math.random() * (max - min) + min);

  try {
    const res = await IBCToTransfer(
      neutronWallet.clientSG,
      address_from,
      address_to,
      IBC_CHANNEL_TO,
      "transfer",
      {
        denom: BASE_DENOM,
        amount: String(randomAmount),
      }
    );
    await sleep(5000);
    const { code, hash } = await neutronWallet.clientCW.getTx(res.txHash);
    if (code !== 0) {
      return {
        mode: NeutronAction.PROCESS_LSM_SHARES_IBC_TO,
        txHash: hash,
        reason: "Check up given hash",
      };
    }
    return res;
  } catch (e) {
    return {
      mode: NeutronAction.PROCESS_LSM_SHARES_IBC_TO,
      txHash: null,
      reason: e.message,
    };
  }
}

async function processLSMShares(
  neutronWallet: Wallet,
  targetWallet: Wallet,
  dropCore: DropCoreClient
): Promise<Array<Action>> {
  const logRandomIBCToTransfer = await randomIBCToTransfer(
    dropCore,
    neutronWallet,
    neutronWallet.mainAccounts[0].address,
    targetWallet.mainAccounts[0].address
  );
  return [logRandomIBCToTransfer];
}

async function main() {
  const neutronWallet: Wallet = {};
  neutronWallet.mainWallet = await DirectSecp256k1HdWallet.fromMnemonic(
    MNEMONIC,
    {
      prefix: "neutron",
    }
  );
  neutronWallet.clientCW = await SigningCosmWasmClient.connectWithSigner(
    NEUTRON_NODE_ADDRESS,
    neutronWallet.mainWallet,
    {
      gasPrice: GasPrice.fromString("0.75untrn"),
    }
  );
  neutronWallet.clientSG = await SigningStargateClient.connectWithSigner(
    NEUTRON_NODE_ADDRESS,
    neutronWallet.mainWallet,
    {
      registry: new Registry(
        new Map<string, GeneratedType>([
          ["/ibc.applications.transfer.v1.MsgTransfer", MsgTransfer],
        ])
      ),
      gasPrice: GasPrice.fromString("0.75untrn"),
    }
  );
  neutronWallet.mainAccounts = await neutronWallet.mainWallet.getAccounts();

  const targetWallet: Wallet = {};
  targetWallet.mainWallet = await DirectSecp256k1HdWallet.fromMnemonic(
    MNEMONIC,
    {
      prefix: TARGET_CHAIN_PREFIX,
    }
  );
  targetWallet.clientCW = await SigningCosmWasmClient.connectWithSigner(
    TARGET_NODE_ADDRESS,
    targetWallet.mainWallet,
    {
      gasPrice: GasPrice.fromString(`0.75${TARGET_DENOM}`),
    }
  );
  targetWallet.clientSG = await SigningStargateClient.connectWithSigner(
    NEUTRON_NODE_ADDRESS,
    neutronWallet.mainWallet,
    {
      registry: new Registry(
        new Map<string, GeneratedType>([
          ["/cosmos.bank.v1beta1.MsgSend", MsgSend],
          ["/ibc.applications.transfer.v1.MsgTransfer", MsgTransfer],
          ["/cosmos.staking.v1beta1.MsgTokenizeShares", MsgTokenizeShares],
          ["/cosmos.staking.v1beta1.MsgDelegate", MsgDelegate],
        ])
      ),
      gasPrice: GasPrice.fromString(`0.75${TARGET_DENOM}`),
    }
  );
  targetWallet.mainAccounts = await targetWallet.mainWallet.getAccounts();

  const coreСontract = new DropCoreClient(
    neutronWallet.clientCW,
    CORE_CONTRACT
  );

  const actions: Array<() => Promise<Array<Action>>> = [
    async (): Promise<Array<Action>> => {
      if (Math.random() <= PROCESS_LSM_PROB) {
        return await processLSMShares(
          neutronWallet,
          targetWallet,
          coreСontract
        );
      }
      return [];
    },
    async (): Promise<Array<Action>> => {
      if (Math.random() <= WITHDRAW_PROB) {
        return [await withdrawRandomNFT(neutronWallet, coreСontract)];
      }
      return [];
    },
    async (): Promise<Array<Action>> => {
      if (Math.random() <= UNBOND_PROB) {
        return [await unbondRandomAmount(neutronWallet, coreСontract)];
      }
      return [];
    },
    async (): Promise<Array<Action>> => {
      if (Math.random() <= BOND_PROB) {
        return [await bondRandomAmount(neutronWallet, coreСontract)];
      }
      return [];
    },
  ];

  const neutronLogs: Array<Action> = [];
  const targetLogs: Array<Action> = [];
  while (actions.length !== 0) {
    const randomIndex = Math.floor(Math.random() * actions.length);
    const logs = await actions[randomIndex]();
    for (const log of logs) {
      if (isInstance(log.mode, TargetAction)) {
        targetLogs.push(log);
      } else {
        neutronLogs.push(log);
      }
    }
    actions.splice(randomIndex, 1);
  }

  console.log(
    JSON.stringify({
      neutron: {
        address: neutronWallet.mainAccounts[0].address,
        logs: neutronLogs,
      },
      target: {
        address: targetWallet.mainAccounts[0].address,
        logs: targetLogs,
      },
    })
  );
}

main();
