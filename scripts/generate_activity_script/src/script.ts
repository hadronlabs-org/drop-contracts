import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { SigningStargateClient } from "@cosmjs/stargate";
import { AccountData, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice, Coin } from "@cosmjs/stargate";
import { Client as DropCoreClient } from "../../../integration_tests/src/generated/contractLib/dropCore";
import { Client as DropWithdrawalManager } from "../../../integration_tests/src/generated/contractLib/dropWithdrawalManager";
import { Client as DropWithdrawalVoucher } from "../../../integration_tests/src/generated/contractLib/dropWithdrawalVoucher";

const CORE_CONTRACT: string = process.env.CORE_CONTRACT;

const MNEMONIC: string = process.env.MNEMONIC;
const BASE_DENOM: string = process.env.BASE_DENOM;
const FACTORY_DENOM: string = process.env.FACTORY_DENOM;
const NEUTRON_NODE_ADDRESS: string = process.env.NEUTRON_NODE_ADDRESS;

const MAX_BOND: number = Number(process.env.MAX_BOND);
const MAX_UNBOND: number = Number(process.env.MAX_UNBOND);

const BOND_PROB: number = Number(process.env.BOND_PROB);
const UNBOND_PROB: number = Number(process.env.UNBOND_PROB);
const WITHDRAW_PROB: number = Number(process.env.WITHDRAW_PROB);

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

enum MODE {
  BOND = "BOND",
  UNBOND = "UNBOND",
  WITHDRAW = "SEND_NFT",
}

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
): Promise<Action> {
  const { transactionHash } = await dropInstance.bond(address, {}, "auto", "", [
    {
      amount: fund.amount,
      denom: fund.denom,
    },
  ]);
  return {
    mode: MODE.BOND,
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
): Promise<Action | null> {
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
      mode: MODE.BOND,
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
      mode: MODE.BOND,
      txHash: null,
      reason: `Nothing to bond, ${BASE_DENOM} balance is lower then min(${min}) (this value either exchange rate or config.lsm_min_bond_amount)`,
    };
  }
  if (min > MAX_BOND) {
    return {
      mode: MODE.BOND,
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
  const random_amount: number = Math.floor(Math.random() * (max - min) + min);
  try {
    const res = await bond(dropInstance, address, {
      amount: String(random_amount),
      denom: BASE_DENOM,
    });
    const { code, hash } = await neutronWallet.clientCW.getTx(res.txHash);
    if (code !== 0) {
      return {
        mode: MODE.BOND,
        txHash: hash,
        reason: "Check up given hash",
      };
    }
    return res;
  } catch (e) {
    return {
      mode: MODE.BOND,
      txHash: null,
      reason: e.message,
    };
  }
}

async function unbond(
  dropInstance: DropCoreClient,
  address: string,
  fund: Coin
): Promise<Action> {
  const { transactionHash } = await dropInstance.unbond(address, "auto", "", [
    {
      amount: fund.amount,
      denom: fund.denom,
    },
  ]);
  return {
    mode: MODE.UNBOND,
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
): Promise<Action | null> {
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
      mode: MODE.UNBOND,
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
        mode: MODE.UNBOND,
        txHash: hash,
        reason: "Check up given hash",
      };
    }
    return res;
  } catch (e) {
    return {
      mode: MODE.UNBOND,
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
): Promise<Action> {
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
    mode: MODE.WITHDRAW,
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
): Promise<Action | null> {
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
      mode: MODE.WITHDRAW,
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
        mode: MODE.WITHDRAW,
        txHash: hash,
        reason: "Check up given hash",
      };
    }
    return res;
  } catch (e) {
    return {
      mode: MODE.WITHDRAW,
      txHash: null,
      reason: e.message,
    };
  }
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
      gasPrice: GasPrice.fromString("0.75untrn"),
    }
  );
  neutronWallet.mainAccounts = await neutronWallet.mainWallet.getAccounts();

  const core_contract = new DropCoreClient(
    neutronWallet.clientCW,
    CORE_CONTRACT
  );
  const logs: Array<Action> = [];
  if (Math.random() <= WITHDRAW_PROB) {
    const res = await withdrawRandomNFT(neutronWallet, core_contract);
    if (res !== null) {
      logs.push(res);
    }
  }
  if (Math.random() <= UNBOND_PROB) {
    const res = await unbondRandomAmount(neutronWallet, core_contract);
    if (res !== null) {
      logs.push(res);
    }
  }
  if (Math.random() <= BOND_PROB) {
    const res = await bondRandomAmount(neutronWallet, core_contract);
    if (res !== null) {
      logs.push(res);
    }
  }
  console.log(
    JSON.stringify({
      neutron: {
        address: neutronWallet.mainAccounts[0].address,
        logs: logs,
      },
    })
  );
}

main();
