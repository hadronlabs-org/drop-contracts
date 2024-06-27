import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { IndexedTx, SigningStargateClient } from "@cosmjs/stargate";
import {
  AccountData,
  DirectSecp256k1HdWallet,
  GeneratedType,
  Registry,
} from "@cosmjs/proto-signing";
import { GasPrice, Coin } from "@cosmjs/stargate";
import { Client as DropCoreClient } from "../../../integration_tests/src/generated/contractLib/dropCore";
import { Client as DropValidatorsSet } from "../../../integration_tests/src/generated/contractLib/dropValidatorsSet";
import { Client as DropWithdrawalManager } from "../../../integration_tests/src/generated/contractLib/dropWithdrawalManager";
import { Client as DropWithdrawalVoucher } from "../../../integration_tests/src/generated/contractLib/dropWithdrawalVoucher";

import { MsgTokenizeShares } from "@messages";
import { MsgSend } from "cosmjs-types/cosmos/bank/v1beta1/tx";
import { MsgDelegate } from "cosmjs-types/cosmos/staking/v1beta1/tx";
import { MsgTransfer } from "cosmjs-types/ibc/applications/transfer/v1/tx";

import { sleep } from "../../../integration_tests/src/helpers/sleep";
import { waitForTx } from "../../../integration_tests/src/helpers/waitForTx";

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

const BOND_PROB: number = parseFloat(process.env.BOND_PROB) || 0;
const UNBOND_PROB: number = parseFloat(process.env.UNBOND_PROB) || 0;
const WITHDRAW_PROB: number = parseFloat(process.env.WITHDRAW_PROB) || 0;
const PROCESS_LSM_PROB: number = parseFloat(process.env.PROCESS_LSM_PROB) || 0;

/*
 * Each of given probabilities should be in interval [0, 1]
 * According to given probabilities relevant actions will be choosed with appropriate probability
 */
if (!(0 <= BOND_PROB && BOND_PROB <= 1)) {
  throw `BOND_PROB should be in interval 0 <= BOND_PROB(${BOND_PROB}) <= 1`;
}
if (!(0 <= UNBOND_PROB && UNBOND_PROB <= 1)) {
  throw `UNBOND_PROB should be in interval 0 <= UNBOND_PROB(${UNBOND_PROB}) <= 1`;
}
if (!(0 <= WITHDRAW_PROB && WITHDRAW_PROB <= 1)) {
  throw `WITHDRAW_PROB should be in interval 0 <= WITHDRAW_PROB(${WITHDRAW_PROB}) <= 1`;
}
if (!(0 <= PROCESS_LSM_PROB && PROCESS_LSM_PROB <= 1)) {
  throw `PROCESS_LSM_PROB should be in interval 0 <= PROCESS_LSM_PROB(${PROCESS_LSM_PROB}) <= 1`;
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

/* Function returns ActionLog if transaction succeeded and throws an error if any problem occurred
 */
async function bondRandomAmount(
  neutronWallet: Wallet,
  dropInstance: DropCoreClient
): Promise<ActionLog> {
  const address: string = neutronWallet.mainAccounts[0].address;

  /* If here is nothing to bond on our balance, then just throw an error
   */
  const IBCDenomBalance: Coin = await neutronWallet.clientCW.getBalance(
    address,
    BASE_DENOM
  );
  if (Number(IBCDenomBalance.amount) === 0) {
    throw `Nothing to bond, ${BASE_DENOM} balance is 0`;
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
  const minBond: number = Math.max(
    Number(config.lsm_min_bond_amount),
    minExchangeRate
  );
  if (minBond > Number(IBCDenomBalance.amount)) {
    throw `Nothing to bond, ${BASE_DENOM} balance is lower then min(${minBond}) (this value either exchange rate or config.lsm_min_bond_amount)`;
  }
  if (minBond > MAX_BOND) {
    throw `MAX_BOND lower then min(${minBond}) (this value either exchange rate or config.lsm_min_bond_amount)`;
  }

  /* Maximum amount of funds that we can send to core contract while bonding
   * Is either our current balance in case if it's lower then MAX_BOND parameter
   * or MAX_BOND otherwise.
   */
  const maxBond: number = Math.min(Number(IBCDenomBalance.amount), MAX_BOND);
  const randomAmount: number = Math.floor(
    Math.random() * (maxBond - minBond) + minBond
  );
  const res = await bond(dropInstance, address, {
    amount: String(randomAmount),
    denom: BASE_DENOM,
  });
  const { code, hash } = await neutronWallet.clientCW.getTx(res.txHash);
  if (code !== 0) {
    throw `Check up given hash${hash}`;
  }
  return res;
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

/* Function returns ActionLog if transaction succeeded and throws an error if any problem occurred
 */
async function unbondRandomAmount(
  neutronWallet: Wallet,
  dropInstance: DropCoreClient
): Promise<ActionLog> {
  const address: string = neutronWallet.mainAccounts[0].address;
  /* If here is nothing to bond on our balance, then just throw an error
   */
  const factoryBalance: Coin = await neutronWallet.clientCW.getBalance(
    address,
    FACTORY_DENOM
  );
  if (Number(factoryBalance.amount) === 0) {
    throw `Nothing to unbond, ${FACTORY_DENOM} balance is 0`;
  }

  const maxUnbond: number = Math.min(Number(factoryBalance.amount), MAX_UNBOND);
  const randomAmount: number = Math.floor(
    Math.random() * Number(maxUnbond) + 1
  );
  const res = await unbond(dropInstance, address, {
    amount: String(randomAmount),
    denom: FACTORY_DENOM,
  });
  const { code, hash } = await neutronWallet.clientCW.getTx(res.txHash);
  if (code !== 0) {
    throw `Check up given hash ${hash}`;
  }
  return res;
}

async function sendNFT(
  withdrawalVoucher: DropWithdrawalVoucher,
  withdrawalManager: DropWithdrawalManager,
  address: string,
  NFTID: string
): Promise<ActionLog> {
  /* To withdraw unbonded amount we need to send nft to the withdrawal_manager contract
   * To do that we need to call send method on withdrawal_voucher contract (which is NFT contract underhood)
   * Field msg here is encoded base64 json object { "withdraw": {} }
   */
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
  return {
    mode: NeutronAction.WITHDRAW,
    txHash: transactionHash,
  };
}

/* Function returns Action if transaction succeeded and throws an error if any problem occurred
 */
async function withdrawRandomNFT(
  neutronWallet: Wallet,
  dropInstance: DropCoreClient
): Promise<ActionLog> {
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
   * Throw an exception
   */
  if (withdrawnNFTs.length === 0) {
    throw "Nothing to withdraw";
  }

  /* Pick up random NFT from given NFT list and try to withdraw it
   * If any error occured when executing method then just ignore
   * It's content and return null, script will try to call another method
   */
  const NFTID: string =
    withdrawnNFTs[Math.floor(Math.random() * withdrawnNFTs.length)];

  const res = await sendNFT(
    withdrawalVoucher,
    withdrawalManager,
    address,
    NFTID
  );
  const { code, hash } = await neutronWallet.clientCW.getTx(res.txHash);
  if (code !== 0) {
    throw `Check up given hash ${hash}`;
  }
  return res;
}

/* Function dedicated to do IBC transfer
 * This function used by two other functions in this script:
 *  - IBCToTransfer
 *  - IBCFromTransfer
 * We've to use signAndBroadcast instead of sendIbcTokens since second is deprecated
 */
async function IBCTransfer(
  clientSG: SigningStargateClient,
  addressFrom: string,
  addressTo: string,
  channel: string,
  port: string,
  amount: Coin,
  fee: Coin
): Promise<string> {
  const { transactionHash } = await clientSG.signAndBroadcast(
    addressFrom,
    [
      {
        typeUrl: "/ibc.applications.transfer.v1.MsgTransfer",
        value: {
          sourcePort: port,
          sourceChannel: channel,
          token: amount,
          sender: addressFrom,
          receiver: addressTo,
          timeoutHeight: "0",
          timeoutTimestamp: String(
            (Math.floor(Date.now() / 1000) + 10 * 60) * 1e9
          ),
        },
      },
    ],
    {
      gas: "200000",
      amount: [fee],
    },
    ""
  );
  await waitForTx(clientSG, transactionHash);
  return transactionHash;
}

/* Function dedicated to do IBC transfer from Neutron to Remote Chain
 * Function doesn't handle any exceptions from the inner call
 */
async function IBCToTransfer(
  clientSG: SigningStargateClient,
  addressFrom: string,
  addressTo: string,
  channel: string,
  port: string,
  amount: Coin
) {
  const txHash = await IBCTransfer(
    clientSG,
    addressFrom,
    addressTo,
    channel,
    port,
    amount,
    {
      denom: "untrn",
      amount: "2000",
    }
  );
  return {
    mode: NeutronAction.PROCESS_LSM_SHARES_IBC_TO,
    txHash: txHash,
  };
}

/* Function dedicated to choose the random amount of tokens that we want to send in processLSMShares function
 * MAX_LSM_PROCESS from .env is the maximum amount of tokens that this function can choose to send
 * If current balance lower than MAX_LSM_PROCESS, then MAX_LSM_PROCESS is current balance
 */
async function randomIBCToTransfer(
  dropCore: DropCoreClient,
  neutronWallet: Wallet,
  addressFrom: string,
  addressTo: string
): Promise<ActionLog> {
  const baseDenomBalance: Coin = await neutronWallet.clientCW.getBalance(
    addressFrom,
    BASE_DENOM
  );
  /* If here is nothing to send on our balance, then just throw an error
   */
  if (Number(baseDenomBalance.amount) === 0) {
    throw `Nothing to transfer via IBC, ${BASE_DENOM} balance is 0`;
  }

  const exchangeRate: number = Math.floor(
    Number(await dropCore.queryExchangeRate())
  );
  const minExchangeRate: number = exchangeRate + 1;
  const config = await dropCore.queryConfig();

  /* min is a minimum amount of tokens that this function can randomly choose from interval
   * It's either a config.lsm_min_bond_amount or minExchangeRate depends on
   * What is bigger (the biggest here is a chosen minimum)
   */
  const minIBCSend: number = Math.max(
    Number(config.lsm_min_bond_amount),
    minExchangeRate
  );
  if (minIBCSend > Number(baseDenomBalance.amount)) {
    throw `Nothing to send via IBC, ${BASE_DENOM} balance is lower then min(${minIBCSend}) (this value either exchange rate or config.lsm_min_bond_amount)`;
  }
  if (minIBCSend > MAX_LSM_PROCESS) {
    throw `MAX_LSM_PROCESS lower then min(${minIBCSend}) (this value either exchange rate or config.lsm_min_bond_amount)`;
  }

  /* max is a maximum amount of tokens that this function can randomly choose from interval
   * It's either a MAX_LSM_PROCESS or current BASE_DENOM balance depends on
   * What is less (the smaller here is a chosen maximum)
   */
  const maxIBCSend: number = Math.min(
    Number(baseDenomBalance.amount),
    MAX_LSM_PROCESS
  );
  const randomAmount: number = Math.floor(
    Math.random() * (maxIBCSend - minIBCSend) + minIBCSend
  );

  /* By default in our case it's always "transfer" port
   */
  const res = await IBCToTransfer(
    neutronWallet.clientSG,
    addressFrom,
    addressTo,
    IBC_CHANNEL_TO,
    "transfer",
    {
      denom: BASE_DENOM,
      amount: String(randomAmount),
    }
  );
  let txDetails: IndexedTx = await neutronWallet.clientCW.getTx(res.txHash);
  while (txDetails === null) {
    await sleep(5000);
    txDetails = await neutronWallet.clientCW.getTx(res.txHash);
  }

  /* Check for the error code in given transaction
   * If it's not a zero then throw an error and hint that the transaction should be checked
   * BTW, since neutron nodes're collecting the transaction details here we can check for the eny errors
   * On the remote chain it either could be possible or not (because protobuf structs may differ).
   * It's a reason why we check it here and don't check in IBCFromTransfer function
   */
  const { code, hash } = txDetails;
  if (code !== 0) {
    `Check up given hash ${hash}`;
  }
  return res;
}

/* Function dedicated to delegate tokens
 * Original function SigningStargateClient.delegateTokens does not provide txHash in return
 * It's a reason why we're forced to use custom message type and broadcast it
 */
async function delegateTokens(
  clientSG: SigningStargateClient,
  addressFrom: string,
  randomValidator: string,
  amount: Coin
): Promise<ActionLog> {
  const { transactionHash } = await clientSG.delegateTokens(
    addressFrom,
    randomValidator,
    amount,
    {
      gas: "300000",
      amount: [
        {
          denom: TARGET_DENOM,
          amount: "3000",
        },
      ],
    },
    ""
  );
  return {
    mode: TargetAction.PROCESS_LSM_SHARES_DELEGATE,
    txHash: transactionHash,
  };
}

/* Function dedicated to tokenize deligated tokens into shares
 * There is no such custom message in cosmjs library so we generated this type from protobuf by our own
 * And provided it into clientSG registry in main function.
 * protoc call:
 * protoc --plugin="protoc-gen-ts=./node_modules/.bin/protoc-gen-ts_proto" --ts_opt=esModuleInterop=true messages.proto --ts_out=./ts-proto --proto_path=...
 */
async function tokenizeShares(
  clientSG: SigningStargateClient,
  validatorAddress: string,
  addressFrom: string,
  amount: Coin
): Promise<ActionLog> {
  const { transactionHash } = await clientSG.signAndBroadcast(
    addressFrom,
    [
      {
        typeUrl: "/cosmos.staking.v1beta1.MsgTokenizeShares",
        value: {
          delegatorAddress: addressFrom,
          validatorAddress: validatorAddress,
          amount: amount,
          tokenizedShareOwner: addressFrom,
        },
      },
    ],
    {
      gas: "400000",
      amount: [
        {
          denom: TARGET_DENOM,
          amount: "4000",
        },
      ],
    },
    ""
  );
  return {
    mode: TargetAction.PROCESS_LSM_SHARES_TOKENIZE_SHARES,
    txHash: transactionHash,
  };
}

/* Function dedicated to do IBC transfer of tokenized shares
 * Back to Neutron from Remote Chain
 */
async function IBCFromTransfer(
  clientSG: SigningStargateClient,
  addressFrom: string,
  addressTo: string,
  channel: string,
  port: string,
  amount: Coin
) {
  const txHash = await IBCTransfer(
    clientSG,
    addressFrom,
    addressTo,
    channel,
    port,
    amount,
    {
      denom: TARGET_DENOM,
      amount: "2000",
    }
  );
  return {
    mode: TargetAction.PROCESS_LSM_SHARES_IBC_FROM,
    txHash: txHash,
  };
}

/* Function dedicated to reveal which tokenized share ID is the latest on our account
 * To get it we're iterating over all tokens with cosmosvaloper in their name and getting ID after / character
 * Comparing them by value and then returning the latest full not-splitted denom
 * If there is no denom with cosmosvaloper in it's name then return null
 * SigningStargateClient.getAllBalances includes the pagination with next_key's
 */
async function lastTokenizeShareDenom(
  targetWallet: Wallet
): Promise<string | null> {
  const allBalances = await targetWallet.clientSG.getAllBalances(
    targetWallet.mainAccounts[0].address
  );
  const filteredBalances = allBalances.filter((balance) =>
    balance.denom.includes("cosmosvaloper")
  );
  if (filteredBalances.length === 0) {
    return null;
  }
  if (filteredBalances.length === 1) {
    return filteredBalances[0].denom;
  }
  return filteredBalances
    .map((balance) => {
      return {
        denom: balance.denom,
        token_share_id: Number(balance.denom.split("/")[1]),
      };
    })
    .reduce((maxId, current) => {
      return maxId.token_share_id > current.token_share_id ? maxId : current;
    }).denom;
}

/* Function dedicated to process LSM shares transactions. It's the longest function in this script
 * This function includes 5 different actions:
 *  - IBC transfer from Neutron to Remote Chain
 *  - Delegation of this amount to validator from whitelist
 *  - Tokenization of delegation from previous step into Shares
 *  - IBC transfer back to Neutron from Remote Chain
 *  - Bond Tokenized Shares
 * Each of these actions processed separately including all specifics
 * If one of these transactions falled on certain step then:
 *  - Nothing happens in future and already executed transactions won't be somehow reverted
 *  - This function sends back the Array<ActionLog> with all successfully executed transactions back to main
 */
async function processLSMShares(
  neutronWallet: Wallet,
  targetWallet: Wallet,
  dropCore: DropCoreClient
): Promise<Array<ActionLog>> {
  /* === IBC transfer from Neutron to Remote Chain Action === */
  /* To understand how many tokens have been sent we need to
   * Memorize how many tokens we've had before IBC transfer
   * Such solution gives us an opportunity not to touch randomIBCToTransfer function return type
   */
  const targetDenomBalanceBefore = await targetWallet.clientCW.getBalance(
    targetWallet.mainAccounts[0].address,
    TARGET_DENOM
  );
  const randomIBCToTransferAction: ActionLog = await randomIBCToTransfer(
    dropCore,
    neutronWallet,
    neutronWallet.mainAccounts[0].address,
    targetWallet.mainAccounts[0].address
  );

  /* Wait untill current balance won't be changed
   * Once changed break the loop
   */
  let targetDenomBalanceAfter: Coin;
  while (true) {
    targetDenomBalanceAfter = await targetWallet.clientCW.getBalance(
      targetWallet.mainAccounts[0].address,
      TARGET_DENOM
    );
    if (
      Number(targetDenomBalanceBefore.amount) ===
      Number(targetDenomBalanceAfter.amount)
    ) {
      await sleep(2000);
    } else {
      break;
    }
  }

  /* Calculate the difference - it's the transfered amount
   * That we're searching for
   */
  const transferedAmount =
    Number(targetDenomBalanceAfter.amount) -
    Number(targetDenomBalanceBefore.amount);

  const config = await dropCore.queryConfig();
  const dropValidatorsSet: DropValidatorsSet = new DropValidatorsSet(
    neutronWallet.clientCW,
    config.validators_set_contract
  );
  const whitelistedValidators: Array<string> = (
    await dropValidatorsSet.queryValidators()
  ).map((validator) => validator.valoper_address);
  const randomValidator: string =
    whitelistedValidators[
      Math.floor(Math.random() * whitelistedValidators.length)
    ];

  /* === Delegation of this amount to validator from whitelist === */
  const delegateTokensAction: ActionLog = await delegateTokens(
    targetWallet.clientSG,
    targetWallet.mainAccounts[0].address,
    randomValidator,
    {
      denom: TARGET_DENOM,
      amount: String(transferedAmount),
    }
  );

  /* === Tokenization of delegation from previous step into Shares === */
  const tokenizeSharesAction: ActionLog = await tokenizeShares(
    targetWallet.clientSG,
    randomValidator,
    targetWallet.mainAccounts[0].address,
    {
      denom: TARGET_DENOM,
      amount: String(transferedAmount),
    }
  );

  /* Latest tokenized share is our denom that we're looking for
   * We need it to do IBC send back to Neutron from remote chain
   */
  let lastLSMAfterTokenizeSharesAction =
    await lastTokenizeShareDenom(targetWallet);

  /* In order to reveal what's the latest IBC denom on Neutron chain is
   * We're using the same method as in previous IBC send
   * Before IBCFromTransfer we're memorizing the current denom list
   * After the execution we'll compare neutronDenomsBeforeIBCFromSend and neutronDenomsAfterIBCFromSend arrays length
   * To get the new IBC denom on Neutron
   */
  const neutronDenomsBeforeIBCFromSend: Array<string> = (
    await neutronWallet.clientSG.getAllBalances(
      neutronWallet.mainAccounts[0].address
    )
  ).map((coin) => coin.denom);

  /* === IBC transfer back to Neutron from Remote Chain === */
  const IBCFromTransferAction: ActionLog = await IBCFromTransfer(
    targetWallet.clientSG,
    targetWallet.mainAccounts[0].address,
    neutronWallet.mainAccounts[0].address,
    IBC_CHANNEL_FROM,
    "transfer",
    {
      denom: lastLSMAfterTokenizeSharesAction,
      amount: String(transferedAmount),
    }
  );

  /* Iterate over the new denoms and wait until
   * New denom'll appear in the list
   */
  let neutronDenomsAfterIBCFromSend: Array<string> = [];
  while (true) {
    const neutronCoinsAfterIBCFromSend =
      await neutronWallet.clientSG.getAllBalances(
        neutronWallet.mainAccounts[0].address
      );
    if (
      neutronCoinsAfterIBCFromSend.length ===
      neutronDenomsBeforeIBCFromSend.length
    ) {
      await sleep(2000);
    } else {
      neutronDenomsAfterIBCFromSend = neutronCoinsAfterIBCFromSend.map(
        (coin) => coin.denom
      );
      break;
    }
  }

  /* Get this denom by array's difference
   */
  const newDenom: string = neutronDenomsAfterIBCFromSend.filter(
    (denom) => !neutronDenomsBeforeIBCFromSend.includes(denom)
  )[0];

  /* === Bond Tokenized Shares === */
  let bondAction: ActionLog = await bond(
    dropCore,
    neutronWallet.mainAccounts[0].address,
    {
      denom: newDenom,
      amount: String(transferedAmount),
    }
  );
  /* It's the Neutron and we definetely know that transactions are indexed here
   * So then we're querying status code of the bond executin and depends on the code
   * Return either ActionLog or throw an exception
   * Also, not forgetting to set bondAction.mode into NeutronAction.PROCESS_LSM_SHARES_BOND
   * Since it's just NeutronAction.BOND by default
   */
  const { code, hash } = await neutronWallet.clientCW.getTx(bondAction.txHash);
  if (code !== 0) {
    throw `Check up given hash ${hash}`;
  }
  bondAction.mode = NeutronAction.PROCESS_LSM_SHARES_BOND;

  return [
    randomIBCToTransferAction,
    delegateTokensAction,
    tokenizeSharesAction,
    IBCFromTransferAction,
    bondAction,
  ];
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
      /* Because of SigningStargateClient has no IBC transfer method by default,
       * We have to use custom registry here
       */
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
    TARGET_NODE_ADDRESS,
    targetWallet.mainWallet,
    {
      /* Because of:
       *  - SigningStargateClient has no IBC transfer method by default
       *  - SigningStargateClient has no MsgTokenizeShares method by default
       * We've to use custom registry with additional message types
       * That we need in LSM Share processing
       */
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
  const actions: Array<() => Promise<Array<ActionLog>>> = [
    async (): Promise<Array<ActionLog>> => {
      if (Math.random() <= PROCESS_LSM_PROB) {
        return await processLSMShares(
          neutronWallet,
          targetWallet,
          coreСontract
        );
      }
      return [];
    },
    async (): Promise<Array<ActionLog>> => {
      if (Math.random() <= WITHDRAW_PROB) {
        return [await withdrawRandomNFT(neutronWallet, coreСontract)];
      }
      return [];
    },
    async (): Promise<Array<ActionLog>> => {
      if (Math.random() <= UNBOND_PROB) {
        return [await unbondRandomAmount(neutronWallet, coreСontract)];
      }
      return [];
    },
    async (): Promise<Array<ActionLog>> => {
      if (Math.random() <= BOND_PROB) {
        return [await bondRandomAmount(neutronWallet, coreСontract)];
      }
      return [];
    },
  ];

  const neutronLogs: Array<ActionLog> = [];
  const targetLogs: Array<ActionLog> = [];
  while (actions.length !== 0) {
    const randomIndex = Math.floor(Math.random() * actions.length);
    const logs = await actions[randomIndex]();
    /* Depending on what the mode of given log we sort it out into
     * Either neutronLogs or targetLogs
     */
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
